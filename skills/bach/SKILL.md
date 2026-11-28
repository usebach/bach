---
name: bach
description: Build a modern Windows installer (Setup.exe) for any desktop app with Bach Installer - one bach.toml, then `bach build`. Use when the user wants to package, ship or distribute a Windows app (Rust, Electron, Tauri, .NET, Go, C++, Python bundles, anything with an .exe), make a setup file or installer, add an uninstaller, auto-updates over an old install, Start menu or desktop shortcuts, PATH, file associations, URL protocols, a license page or a localized install window, or replace an NSIS / Inno Setup / WiX / MSI script.
---

# Bach Installer: Windows installers from one bach.toml

Bach Installer turns a folder with an app into one `Setup.exe`: a calm, native-looking
install window (light and dark, 19 languages, follows Windows' language), a
built-in updater (a newer setup over an install updates it in place) and a
clean uninstaller in Settings › Apps. Installs are per user by default, so
there is no administrator prompt; `installer.scope = "machine"` installs for
every user into Program Files after one Windows prompt, and a `[[run]]` step
can say `elevated = true`. There is no script language: everything is declared
in `bach.toml`.

## macOS too

A `bach.toml` with a `[macos]` section (`source` = the folder with `Hello.app`,
`executable = "Hello.app"`, `data = ["~/Library/Application Support/Hello"]`) makes
`bach build` also write a Setup app in a `.dmg` (a `.zip` away from a Mac) that installs
per user into `~/Applications`, updates in place, and uninstalls with `Uninstall <app>.app`.
`bach build --platform macos` builds only that; `--sign` signs (and, with
`[macos.signing] notarize`, notarizes) it on a Mac. With `[updates] signer`, installed Macs take only
an update signed with an Apple-issued Developer ID of that name: `--sign` with any other certificate
fails, and a macOS build without `--sign` warns. Registry, services, MSI and the other
Windows-only sections are left out of a macOS build. Reference: `docs/config.md#macos`.

## 1. Get the `bach` command

Check first: `bach --version`. If that works, skip to step 2.

No Rust is needed, and none of these needs administrator rights (do not use
`sudo` or an elevated shell). Pick the first that applies:

- **Node.js 18+ is installed (any OS):** `npm install --global @usebach/cli`
  (or `npm install --save-dev @usebach/cli` and `npx bach` for one project).
  If npm's global folder isn't writable, run
  `npm config set prefix ~/.npm-global` and add its `bin` to `PATH`; don't
  use `sudo`. Update: `npm update --global @usebach/cli`; remove:
  `npm uninstall --global @usebach/cli`.
- **Windows (PowerShell):**
  `irm https://raw.githubusercontent.com/usebach/bach-installer/main/install.ps1 | iex`
  installs `bach.exe` and `bach-setup.exe` to `%LOCALAPPDATA%\Programs\bach-cli`
  and puts it on `PATH`. In the same terminal, call it by full path or open a
  new terminal.
- **macOS or Linux (CI too):**
  `curl -fsSL https://raw.githubusercontent.com/usebach/bach-installer/main/install.sh | sh`
  installs `bach` to `~/.local/bin`. It downloads the Windows runtime it
  builds on (`bach-setup.exe`, SHA-256 checked) on the first `bach build`.
- **For every user of the machine:** only if the user asks for it. Set
  `BACH_SCOPE=system` (PowerShell: `$env:BACH_SCOPE = 'system'` before the
  line; shell: `sh -s -- --system`). Windows or `sudo` then asks the user
  once; if they decline, it falls back to a per-user install. Never run the
  user-level install elevated.
- **No network to GitHub's raw files:** download
  `bach-<version>-windows-x64.zip` from https://github.com/usebach/bach-installer/releases,
  unzip, and keep `bach.exe` and `bach-setup.exe` in the same folder.
- **Building from source** (only if Rust is already installed, or the user
  wants it): `cargo install --git https://github.com/usebach/bach-installer bach-cli`,
  then `bach` downloads its runtime when it needs it. In a checkout of the
  Bach Installer repository, `cargo build --release` gives both binaries in
  `target/release`.

Then run `bach doctor` in the project folder. It says whether the runtime
and the config are in place and exactly what to fix if not.

## 2. Build the app into one folder

Bach Installer installs the contents of one folder: the `.exe` plus everything it needs
next to it (DLLs, resources). Find or produce that folder first, for example:

| Stack | Typical folder |
| --- | --- |
| Rust | `target/release` (exclude `*.pdb`, `*.d`, `build/**`, `deps/**`, `incremental/**`, `.fingerprint/**`), or better a folder you copy the `.exe` and assets into |
| Electron (electron-builder / forge) | `dist/win-unpacked` or `out/<name>-win32-x64` |
| Tauri | copy `src-tauri/target/release/<app>.exe` (and any resources) into a folder |
| .NET | `dotnet publish -c Release -r win-x64 --self-contained` output (`bin/Release/<tfm>/win-x64/publish`) |
| Go with Wails | `build/bin` |
| Go (other) | a folder with the built `.exe` |
| Flutter | `build/windows/x64/runner/Release` |
| PyInstaller | `dist/<name>` (one-folder mode) |
| Qt | the folder `windeployqt` filled, often `build/Release` |

## 3. Write bach.toml

Run `bach init` (it won't overwrite an existing file). In a Rust, Tauri,
Wails, Electron, .NET, Flutter, PyInstaller, Qt, Godot or Unity project it fills in name, version, publisher, the program,
the build folder, leftovers to exclude and needed runtimes (WebView2, Visual C++)
from the project's own files; check them, then edit. For .NET the `dotnet`
requirement is commented out, because a self-contained publish carries the
runtime: uncomment it only if you publish with `--self-contained false`. The window always starts
in Bach Installer's own style (`bach`), whatever the project: set `installer.style` only
when the developer asks for another look. Minimal by hand:

```toml
[app]
name = "My App"
version = "1.0.0"
publisher = "Your Name"
executable = "my-app.exe"        # relative to [files] source

[files]
source = "build"                 # relative to bach.toml
exclude = ["*.pdb"]
# allow_secrets = [".env"]   # only for a file that looks like a secret but must ship
```

In a git checkout whose `origin` is on GitHub, `bach init` also wrote
`.github/workflows/bach-release.yml`: a one-click release (Actions › Release ›
Run workflow). In a project that already has a `bach.toml`, run
`bach workflow` to add it (never hand-write a release workflow instead). Tell
the user to commit and push it, and look at its **Build your app** step: it is
filled in for Rust, Tauri, Electron and .NET and holds a `TODO` for anything
else. See "Ship" below.

Take name, version and publisher from the project's own metadata
(`Cargo.toml`, `package.json`, `.csproj`) instead of inventing them. Never
edit the version in several files by hand: `bach version` shows every file's
version and `bach version patch|minor|major|X.Y.Z` raises it in all of them
(`bach.toml`, `package.json`, `package-lock.json`, `Cargo.toml`,
`Cargo.lock`, `tauri.conf.json`, `.csproj`) without reformatting them, so the
lock files stay in step with the manifests. Never change `[app] id` (defaults to
`name`) after a release: it is how updates recognise the install.

Common additions (all optional; full reference in `docs/config.md` of the
repository, https://github.com/usebach/bach-installer/blob/main/docs/config.md):

```toml
[app]
description = "One line about the app."   # subtitle on the first page
icon = "assets/app.ico"                   # setup file icon; also the logo if it holds a PNG

[installer]
# style = "aurora"                        # default "bach"; `bach styles` lists all 14
accent = "#2563eb"
logo = "assets/logo.png"                  # .svg (one colour), .png, or "none"
languages = ["en", "de"]                  # default: all 19; one entry = always that language
license = "LICENSE.txt"                   # accept page before installing
# options: true (offered, ticked) | false (offered, unticked) | "always" | "never"
desktop_shortcut = true
start_menu_shortcut = "always"
start_with_windows = "never"
add_to_path = "never"                     # "always"/true for CLI tools
path_folder = "bin"
change_folder = true
open_button = true

[style]                                   # your own look on top of installer.style
font = "fonts/MyFont.ttf"                 # or a font Windows has, like "Segoe UI"
background = ["#1d0b36", "#0a0416"]       # one colour, or a top-to-bottom gradient
radius = 12                               # card corners, 0 to 24
[style.dark]
card = "#ffffff0b"                        # any colour per mode; [style] sets both modes

[layout]                                  # size, arrangement, spacing, motion (all optional)
preset = "showcase"                       # `bach layouts`: classic, showcase, spotlight, banner, ...
panel_image = "art/hero.jpg"              # panel = "left" | "right" | "top"; header = "content" | "panel"
width = 940                               # height too; portrait or landscape
align = "left"                            # heading: left | center | right
rows = "separate"                         # or "joined"; icons, dividers, row_padding
buttons = "stretch"                       # right | left | center | stretch; primary = "first"
footer = "above"                          # beside | above | hidden
animation = "slide"                       # rise | fade | slide | drop | none; animation_speed

[[components]]                            # optional parts, one box each
id = "samples"
title = { en = "Sample projects", de = "Beispielprojekte" }
files = ["samples/**"]
default = true

[[protocols]]                             # myapp:// links start the app
scheme = "myapp"

[[file_types]]                            # offered under "Open with"
extension = ".mydoc"
description = "My App document"

[text]                                    # replace any text in the window
install_subtitle = { en = "Fast and private.", de = "Schnell und privat." }
footer = "Made by {publisher}"

[[requirements]]                          # checked before installing
kind = "webview2"                         # also "dotnet" (version, desktop), "vcredist", "windows" (build), "file", "registry"

[[shortcuts]]                             # more Start menu shortcuts
name = "My App Docs"
target = "docs.exe"

[[run]]                                   # a packaged program setup runs (runtime installers, drivers)
file = "redist/vc_redist.x64.exe"
arguments = "/quiet /norestart"
when = "first_install"                    # or "install" (default), "uninstall" (before removal)
success_codes = [0, 3010]

[[services]]                              # a Windows service (needs installer.scope = "machine")
name = "MyAppAgent"                       # start = "auto" (default) | "delayed" | "manual" | "disabled"
file = "agent/agent.exe"                  # account = "local_service" (default) | "network_service" | "local_system"
arguments = "--service"

[[firewall]]                              # a Windows Firewall rule (needs installer.scope = "machine")
name = "My App server"
ports = "8080"                            # or file = "agent/agent.exe"; ports need protocol = "tcp" or "udp"
protocol = "tcp"

[access]                                  # private app: asks for a key, encrypts the files with it
# The key is never written here: `bach key --out key.txt`, then `bach build --key-file key.txt`
# (a key file with many lines = one setup file that each key opens; remove a line to revoke on the next build)
# get_key_url = "https://example.com/buy"

[updates]                                 # the installed app can update itself
github = "owner/repo"                     # from that repo's GitHub releases; or:
# feed = "https://example.com/myapp/latest.json"
# download_url = "https://example.com/myapp/{file}"
# auto = true                             # offer "Automatic updates" (false / "always" / "never")
# mirrors = ["https://mirror.example/myapp/latest.json"]   # fallback feeds
# public_key = "..."   # from `bach feed-key`: the app then accepts only feeds signed with it (BACH_FEED_KEY signs)
# beta_feed = "https://.../latest-beta.json"   # beta channel: `uninstall.exe --channel beta|stable|show`
# next_public_key = "..."   # to hand over to a new key: both are trusted (see docs/updates.md "Changing the key")

[uninstall]
data = ['%APPDATA%\My App']               # single quotes keep backslashes; your own folder, never Documents or AppData itself
data_description = "Your settings."
feedback_url = "https://example.com/why"  # "Tell us why" after removing
```

The file starts with `#:schema https://raw.githubusercontent.com/usebach/bach-installer/main/schema/bach.schema.json`,
so editors (and you) can check every key against the schema.

Rules the checker enforces (fix the setting it names, don't work around it):
data folders must start with `%APPDATA%`, `%LOCALAPPDATA%` or `%USERPROFILE%`
plus the app's own folder; the source folder can't contain symlinks or a
top-level `uninstall.exe`; schemes like `http` and extensions like `.exe` are
refused; `[text]` keys must be real text names (listed in docs/config.md).

## 4. Check, look, build

```bash
bach check                 # validates and lists every file; prints warnings
bach preview               # opens the real window; installs nothing (Windows only)
bach preview --lang de --theme dark
bach preview --style arcade  # try another built-in style; bach styles lists them
bach preview --uninstall
bach watch                 # for people: like preview, reopens it on every save; never exits, so agents must not run it
bach build                 # writes dist/<name>-Setup-<version>.exe
bach inspect dist/My\ App-Setup-1.0.0.exe
# In scripts, add --json to check, build, inspect, feed, doctor or version: stdout is one
# JSON object ({"ok", "setup", "version", "sha256", "size", "warnings", ...}),
# never text to parse. Fields: schema/bach-output.schema.json.
```

Treat every `warning:` line from `bach check` as something to fix or to
mention to the user (for example a `[text]` given in one language while the
window speaks several).

## 5. Verify, safely

- Never run the built setup (a real install) on the user's machine unless
  they ask. `bach preview` shows every page without changing anything.
- When a real end-to-end test is wanted, use a silent install and remove it
  again afterwards:
  `Start-Process .\Setup.exe -ArgumentList '--silent' -Wait -PassThru` (setup
  is a windowed program, so plain `.\Setup.exe --silent` doesn't wait or give
  an exit code), then `Start-Process
  "$env:LOCALAPPDATA\Programs\<folder>\uninstall.exe" -ArgumentList
  '--silent' -Wait -PassThru`. Exit code 0 means done; 1 failed (reason
  printed); 2 a flag mistake.
- Silent flags: `--dir PATH`, `--with a,b`, `--without a,b` (ids:
  `desktop-shortcut`, `start-menu`, `autostart`, `path`, component ids),
  `--lang de`, `--allow-downgrade`, `--overwrite` (a first install into a folder
  that already holds files of the same names is refused without it), `--log FILE`,
  and `--delete-data` for the uninstaller.
- Every run logs to `%TEMP%\<app id>-setup.log`; read it when something
  fails instead of guessing.
- In the Bach Installer repository itself, `scripts\e2e.ps1` installs, updates and
  removes a throwaway app for real; it's the safe way to test changes to
  setup.

## 6. Ship

- Sign the setup file with `signtool` like any program (the payload is
  covered by the signature); unsigned downloads get SmartScreen warnings.
  `signtool` is part of the Windows SDK and rarely on `PATH` (Bach Installer finds it
  when it signs). Signing is optional: `[signing]` (`program`, `arguments`
  that start with `sign` for signtool, with `{file}` and `{env:NAME}`) lets `bach build --sign` (or `BACH_SIGN=1`) sign;
  without `--sign` nothing runs. `bach sign FILE` signs a built file.
- To release from GitHub with no setup: `bach workflow`, commit, push, then
  Actions › Release › Run workflow (inputs: `bump`, `version`, `notes`,
  `prerelease`, `draft`, `latest`, `dry_run`; or
  `gh workflow run bach-release.yml -f bump=minor`). It raises the version
  everywhere, installs dependencies (a drifted `package-lock.json` is brought
  up to date from `package.json` and committed, any other install failure
  stops it), builds, tags and publishes. Run it with `dry_run=true` first to
  see it build without publishing. Docs: docs/releasing.md.
- For an update by hand: raise `version`, `bach build`, ship the new file. Running it
  over an install updates in place, keeps the user's choices and data, and
  removes files the new version no longer has.
- In CI, `bach build` exits non-zero with a one-sentence reason on any
  problem. On Linux/macOS runners it works too (the setup file keeps Bach Installer's
  icon there; build on Windows to apply `app.icon`). The GitHub Action does
  both steps: `uses: usebach/bach-installer@v0` with `config: bach.toml`, output
  `setup`.
- To publish updates from GitHub: set `[updates] github = "owner/repo"`, and
  in a workflow that runs on pushed tags (`v*`, with `contents: write`) use
  `usebach/bach-installer@v0` with `config: bach.toml` and `release: true`. The tag is
  the app's version (`bach build --app-version v1.4.0`), the release holds the
  setup file and `latest.json`. `@v0.2` pins a line, a SHA pins a commit.
  `sign-command`, `notes` and `version` (which Bach Installer: `0.2`, `latest`) are the
  other inputs. Docs: docs/updates.md.
- Without GitHub, upload the setup file and the `latest.json` written next to
  it. After signing, rewrite the feed: `bach feed SETUP.exe` (the link comes
  from `bach.toml`, or `--url URL`).
- The installed `uninstall.exe` does the updating: `--check-update [--json]`
  (exit 10 = newer version), `--download-update` (fetch in the background),
  `--apply-update [--wait-for PID] [--launch]` (install what was fetched,
  once the app has closed), `--update [--silent]`. The "Automatic updates"
  box adds a per-user scheduled task that does it every few hours while the
  app is closed. A good app runs `--download-update` at start and
  `--apply-update --wait-for <pid> --launch` when it quits.
- An **online installer** (a small setup that downloads the newest version
  when it runs, so one download link is always current): `online = true` under
  `[installer]` or `bach build --online`; it needs `[updates]` and is written as
  `dist/<name>-Web-Setup.exe`. The action signs and publishes it too.
- To ship a risky release to a share of users first: `bach feed SETUP.exe
  --rollout 10`, then write the feed again without `--rollout`.
- A private app: `[access]` asks for a key and encrypts the files. Keep the key
  out of `bach.toml` (env var `BACH_ACCESS_KEY`, `bach key` makes one). Silent
  installs take `--key-file FILE`. Updates send the key as a Bearer header, so a
  private server can refuse anyone without one. Installed files stay readable on
  the user's disk: for real protection check a license inside the app.
- To list the app in winget: `bach winget SETUP.exe --url URL --license MIT`,
  then `winget validate` and a pull request to microsoft/winget-pkgs.
- To make a Chocolatey package: `bach choco SETUP.exe --url URL --license MIT`,
  then `choco pack` in the folder it writes (use `installer.scope = "machine"`,
  since Chocolatey installs as an administrator).
- For Group Policy or Configuration Manager: `bach build --msi` also writes a real `.msi` (files, shortcuts,
  registry, optional components, upgrades); it says what it can't carry. 3-number versions only. Opening the `.msi`
  shows the same Bach Installer window as the setup file (Windows Installer does the work under it); `/qn` stays quiet.
- To deploy with Microsoft Intune: `bach intune SETUP.exe` writes the install and
  uninstall commands and the detection rule (`intune.md`) and a detection script
  (`detect.ps1`); use `installer.scope = "machine"` for devices.
- To have each release also open the winget update PR: `bach workflow --winget Publisher.App`
  (needs a `WINGET_TOKEN` repository secret; the first winget submission is still by hand).
- To try a setup file on a clean Windows: `bach test [SETUP.exe]` runs install, start and
  uninstall in Windows Sandbox (Pro/Enterprise/Education only) and reports failures and
  leftovers; `--dry-run` only writes the sandbox files. `bach test --here` runs it on this
  computer instead (for CI runners and virtual machines: it installs for real, and refuses
  when the app is already installed).
- To check a setup file without installing it: `bach verify SETUP.exe` (add
  `--key-file FILE` for an encrypted one; `--json` gives `verified` and `signature`).
- For one download that serves x64 and ARM64: add `[arch.arm64] source = "<arm64 folder>"`; `bach build
  --runtime-arm64 <ARM64 bach-setup.exe>` writes both setup files and one `latest.json` that lists both
  (`--x64-only` skips the ARM64 one).
- For files most people won't want (language packs, HD textures, demo files): give the `[[components]]`
  entry `download = true` (or an https link). `bach build` writes them to `Name-component-version.bachpack`
  to upload; the installer downloads it only when the box is ticked.
- For two independent choices over the same files (pick vendors, and one "Tools" box for all of them):
  set `installer.shared_files = "all"` and let the `[[components]]` patterns overlap; a file is then
  installed only when every component matching it is ticked. Needs Bach Installer newer than 0.11.0; no `--msi`.
- For a portable ZIP or a company (machine-wide) version of the same app: add `[profiles.NAME]`
  to `bach.toml` (`kind = "zip"`, or settings that differ), then `bach build --profile NAME` or
  `bach build --all-profiles`; `bach profiles` lists them.
- To start from an existing Inno Setup or NSIS installer: `bach import installer.iss` (or `.nsi`)
  writes a first `bach.toml` next to it and lists what it couldn't convert; `--dry-run` prints it.
  Read it, then `bach check`.
- In GitHub Actions, `compare: true` on the `usebach/bach-installer` action writes `bach diff` against the
  latest release to the job summary and one pull request comment (needs `pull-requests: write`).
- To see what changed between two releases: `bach diff OLD.exe NEW.exe` (files added,
  removed and changed, new registry values, which files grew the app; `--json` too).
- For a download page: `bach page SETUP.exe --url URL` writes `download/index.html`.
- In CI, `bach check --strict` (or `BACH_STRICT=1`) fails on any warning.

## Troubleshooting

| Message | Fix |
| --- | --- |
| `A new version of Bach Installer is available …` | Run `bach update` (or the command the line names for npm, Scoop, winget or Cargo installs). `bach update --check --json` reports `update_available` without changing anything; `BACH_NO_UPDATE_CHECK=1` silences the line. |
| `couldn't get bach-setup.exe …` | No network or no release for this version: put `bach-setup.exe` next to `bach`, or pass `--runtime PATH` / set `BACH_RUNTIME`. |
| `app.executable is "…", but there's no such file` | Build the app first, or fix `source`/`executable` (paths are relative to `bach.toml` and `source`). |
| `… is a symbolic link` | Copy real files into the source folder. |
| `couldn't replace … (is it open?)` | Close the old setup file or the running app, then build again. |
| `bach preview … needs Windows` | Use `bach check` on macOS/Linux. |
| `requirements[0].…` | Give `kind` one of the listed kinds; `file` and `registry` need `name` and an https `url`. |
| The installer says a newer version is installed | Raise `version` above the installed one, or pass `--allow-downgrade` on a silent run. |
| `The bach program for … isn't installed` (npm) | npm skipped the optional platform package (`--omit=optional`). Install again without that flag. |
| `bach workflow` says the file wasn't written by Bach Installer | A different `.github/workflows/bach-release.yml` exists; keep it, or pass `--force` to replace it. |
| The release workflow can't push | The branch is protected: let `github-actions[bot]` push to it, or run the workflow from another branch. |
| `install.ps1` refuses a folder | It never writes into a folder it didn't make; set `BACH_INSTALL_DIR` to an empty one. |
