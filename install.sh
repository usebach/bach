#!/bin/sh
# Installs the bach command on macOS or Linux, no Rust needed:
#
#   curl -fsSL https://raw.githubusercontent.com/usebach/bach/main/install.sh | sh
#
# Downloads the latest release for this system (or $BACH_VERSION: "0.2.0"
# for exactly that one, "0.2" or "0" for the newest of that line, "latest"
# for the newest), checks its SHA-256 and puts `bach` in ~/.local/bin (or
# $BACH_INSTALL_DIR). `bach build` fetches the Windows setup runtime it
# builds on by itself, the first time it needs it.
#
# It never needs administrator rights, and never asks for them, unless you
# ask for a system-wide install: BACH_SCOPE=system (or --system) puts bach in
# /usr/local/bin for every user, using sudo only for that one copy, and only
# if that folder isn't yours to write to. If sudo isn't there or is refused,
# bach is installed for you alone instead.
#
#   curl -fsSL https://raw.githubusercontent.com/usebach/bach/main/install.sh | sh -s -- --system
set -eu

repo="usebach/bach"
scope="${BACH_SCOPE:-user}"
for arg in "$@"; do
  case "$arg" in
    --system) scope=system ;;
    --user) scope=user ;;
    *) echo "Unknown option $arg. Use --system or --user." >&2; exit 1 ;;
  esac
done
case "$scope" in
  user | system) ;;
  *) echo "BACH_SCOPE must be user or system, not $scope." >&2; exit 1 ;;
esac
user_dir="$HOME/.local/bin"
if [ "$scope" = system ]; then default_dir="/usr/local/bin"; else default_dir="$user_dir"; fi
dir="${BACH_INSTALL_DIR:-$default_dir}"

case "$(uname -s)" in
  Linux) os=linux ;;
  Darwin) os=macos ;;
  *) echo "On Windows, use install.ps1 instead." >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64 | amd64) arch=x64 ;;
  arm64 | aarch64) arch=arm64 ;;
  *) echo "There's no prebuilt bach for $(uname -m); build it with: cargo install --git https://github.com/$repo bach-cli" >&2; exit 1 ;;
esac

wanted="${BACH_VERSION:-}"
wanted="${wanted#v}"
[ "$wanted" = "latest" ] && wanted=""
# A token (as in GitHub Actions) keeps the API's rate limit out of the way.
gh_api() {
  if [ -n "${GITHUB_TOKEN:-}" ]; then
    curl -fsSL -H "Authorization: Bearer $GITHUB_TOKEN" "$1"
  else
    curl -fsSL "$1"
  fi
}

case "$wanted" in
  *[!0-9.]* | .* | *. | *..*) partial="" ;;
  [0-9]* ) dots=$(printf '%s' "$wanted" | tr -cd '.' | wc -c)
           if [ "$dots" -le 1 ]; then partial=1; else partial=""; fi ;;
  *) partial="" ;;
esac

if [ -n "$partial" ]; then
  # "0" or "0.2": the newest release of that line. 0.2 is 0.2.0, 0.2.1, ...
  # and never 0.20; a pre-release is only taken when the line has no other.
  list=$(gh_api "https://api.github.com/repos/$repo/releases?per_page=100") || {
    echo "Couldn't reach GitHub to find Bach $wanted; check your connection." >&2; exit 1; }
  # One "tag prerelease" pair per release, from GitHub's pretty-printed JSON.
  pairs=$(printf '%s\n' "$list" | awk '
    /^    "tag_name":/ { gsub(/[",]/, "", $2); tag = $2 }
    /^    "draft":/ { draft = ($2 ~ /true/) }
    /^    "prerelease":/ { if (!draft) print tag, ($2 ~ /true/ ? "pre" : "stable") }')
  escaped=$(printf '%s' "$wanted" | sed 's/\./\\./g')
  line=$(printf '%s\n' "$pairs" | grep -E "^v$escaped(\.|[ ])" || true)
  tag=$(printf '%s\n' "$line" | grep ' stable$' | cut -d ' ' -f 1 | sort -V | tail -n 1)
  [ -n "$tag" ] || tag=$(printf '%s\n' "$line" | cut -d ' ' -f 1 | sort -V | tail -n 1)
  [ -n "$tag" ] || { echo "Bach has no release $wanted.x. See https://github.com/$repo/releases" >&2; exit 1; }
  echo "Bach $wanted resolves to $tag"
elif [ -n "$wanted" ]; then
  tag="v$wanted"
else
  tag=$(gh_api "https://api.github.com/repos/$repo/releases/latest" 2>/dev/null | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)
  # "latest" skips pre-releases; while there are only those, take the newest.
  if [ -z "$tag" ]; then
    tag=$(gh_api "https://api.github.com/repos/$repo/releases?per_page=1" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)
  fi
fi
[ -n "$tag" ] || { echo "Couldn't find the latest Bach release; check your connection." >&2; exit 1; }

name="bach-${tag#v}-$os-$arch"
url="https://github.com/$repo/releases/download/$tag/$name.tar.gz"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "Downloading $name"
if ! curl -fsSL -o "$work/$name.tar.gz" "$url"; then
  echo "Release $tag has no $name download. Build bach with: cargo install --git https://github.com/$repo bach-cli" >&2
  exit 1
fi
curl -fsSL -o "$work/$name.tar.gz.sha256" "$url.sha256"
expected=$(cut -d ' ' -f 1 < "$work/$name.tar.gz.sha256")
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$work/$name.tar.gz" | cut -d ' ' -f 1)
else
  actual=$(shasum -a 256 "$work/$name.tar.gz" | cut -d ' ' -f 1)
fi
[ "$expected" = "$actual" ] || { echo "The download doesn't match its published SHA-256; nothing was installed." >&2; exit 1; }

tar -xzf "$work/$name.tar.gz" -C "$work"

# Administrator rights are asked for only when a system-wide install was
# requested and the folder isn't writable by this user; the one copy is all
# that runs with them.
sudo=""
can_write() {
  # Walk up to the first folder that exists: mkdir -p would create below it.
  probe="$1"
  while [ ! -d "$probe" ]; do probe=$(dirname "$probe"); done
  [ -w "$probe" ]
}
if ! can_write "$dir"; then
  if [ "$scope" = system ] && command -v sudo >/dev/null 2>&1; then
    echo "Installing for every user needs administrator rights to write to $dir; sudo may ask for your password."
    if sudo -v; then
      sudo="sudo"
    fi
  fi
  if [ -z "$sudo" ]; then
    if [ "$scope" = system ] && [ -z "${BACH_INSTALL_DIR:-}" ]; then
      echo "No administrator rights, so bach is installed for you alone in $user_dir instead." >&2
      scope=user
      dir="$user_dir"
    else
      echo "$dir isn't writable by you. Set BACH_INSTALL_DIR to a folder of your own, or use BACH_SCOPE=system to install for every user." >&2
      exit 1
    fi
  fi
fi
$sudo mkdir -p "$dir"
# A `bach` this script didn't put there belongs to something else: keep it.
mark="$dir/.bach-cli"
if [ -e "$dir/bach" ] && [ ! -e "$mark" ]; then
  echo "$dir/bach already exists and isn't from this script, so nothing was installed. Set BACH_INSTALL_DIR to another folder." >&2
  exit 1
fi
$sudo install -m 755 "$work/$name/bach" "$dir/bach"
echo "Installed by https://github.com/usebach/bach/blob/main/install.sh" > "$work/mark"
$sudo install -m 644 "$work/mark" "$mark"
echo "Installed $("$dir/bach" --version) to $dir/bach"
case ":$PATH:" in
  *":$dir:"*) ;;
  *) echo "Add $dir to your PATH to run bach from anywhere." ;;
esac
