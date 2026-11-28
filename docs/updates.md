# Updates

Bach Installer apps can keep themselves up to date. This page covers publishing
updates from GitHub, what the installed app does with them, and how an
update survives a closed window, a closed app or a power cut.

There are two ways to update:

- **By hand.** Hand out a newer setup file. Running it over an install
  updates it in place (see [getting started](getting-started.md#7-ship-updates)).
- **By themselves.** The installed app reads an update feed, downloads the
  newer setup file in the background and installs it when the app is closed.
  The rest of this page is about that.

## How an installed app knows what is new

The installed app doesn't guess. It asks one small file, the **update feed**
(`latest.json`), whose address is baked into every setup file by
`[updates]` in `bach.toml`. The feed says which version is newest and where
to download it:

```json latest.json
{
  "version": "1.4.0",
  "url": "https://github.com/owner/repo/releases/download/v1.4.0/Hello-Setup-1.4.0.exe",
  "sha256": "<sha-256 of the setup file>",
  "notes": "Faster start",
  "size": 10187776
}
```

The whole loop, step by step:

1. **You publish.** `bach build` writes the setup file and `latest.json`
   next to it; you (or the release workflow) upload both to a place with a
   fixed address for the feed.
2. **The app asks.** The scheduled task (every four hours) or your app
   (`uninstall.exe --check-update`) fetches the feed and compares its
   `version` with the installed version. Only a higher version counts.
3. **It downloads.** The setup file from `url`, into the background folder,
   checked against `sha256`.
4. **It installs** when the app isn't running.

So there is no server of yours to run: any place that serves two static
files over https works. See [where to host it](#where-to-host-updates). The
online installer uses the same feed, so publishing a release updates
installed apps *and* what new users get.

## Where to host updates

| You have | Set | Notes |
| --- | --- | --- |
| A public GitHub repository | `github = "owner/repo"` | The release workflow does everything. The repository must be public. |
| Static hosting: Cloudflare R2 or Pages, S3, Netlify, your own web server | `feed` and `download_url` | Upload the setup file and `latest.json` to fixed addresses. Nothing else to run. |
| Two places for safety | `mirrors = [...]` | More feeds, asked when the main one is down. |
| A private server | `[access]` with `send_key = true` | The server checks `Authorization: Bearer <key>`. See [private updates](#private-updates). |

## Publish updates from GitHub

Add one line to `bach.toml`:

```toml
[updates]
github = "owner/repo"
```

Then publish. The quickest way is the one-click release workflow that
`bach init` or `bach workflow` writes into your repository: it raises the
version everywhere, builds your app and publishes the release (see
[releasing](releasing.md)). To write the workflow yourself, use the Bach Installer
action. This one builds on every pushed tag like `v1.4.0`:

```yaml
on:
  push:
    tags: ['v*']
permissions:
  contents: write          # the action publishes the release
jobs:
  release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      # ... build your app into the folder [files] source names ...
      - uses: usebach/bach-installer@v0
        with:
          config: bach.toml
          release: true
```

The tag is the version: `v1.4.0` builds the app as `1.4.0` (the action passes
`--app-version`), so there is no second place to keep it. The action builds
the setup file, writes `latest.json`, and publishes both as a GitHub release.
The installed apps find them at:

| | Link |
| --- | --- |
| Feed | `https://github.com/owner/repo/releases/latest/download/latest.json` |
| Setup file | `https://github.com/owner/repo/releases/download/v1.4.0/<file>` |

GitHub renames uploaded files that have spaces or odd characters, which would
break the link. With `github` set, `bach build` names the setup file the way
GitHub serves it (anything but letters, digits, `.`, `-` and `_` becomes
`-`), so `My App` is built as `My-App-Setup-1.4.0.exe`.

A pre-release (`prerelease: true`) is not GitHub's "latest", so installed apps
don't update to it. Releasing a fix to an older line with `latest: false`
keeps them on the newest one.

Your repository must be public: the installed app reads the feed without
signing in.

### Action settings

| Input | Meaning |
| --- | --- |
| `config` | The `bach.toml` to build. Without it the action only installs `bach`. |
| `release` | `true` publishes the setup file and `latest.json` as a release. Needs `contents: write`. |
| `app-version` | `auto` (default) takes the version from the tag the run is for; a version overrides `app.version`; empty keeps `bach.toml`'s. |
| `notes`, `notes-file` | What's new. Apps that offer the update can show it. |
| `args` | More arguments for `bach build`, like `--fast` or `--patch-from OLD.exe`, written like a command line: spaces separate words and quotes keep a value with spaces in one piece (`--patch-from "dist/My App 1.0.exe"`). Nothing is expanded, and a word can't hold a line break. |
| `sign-command` | A PowerShell command that signs `$env:BACH_SETUP_FILE`. It runs before the feed is written, so the feed has the signed file's SHA-256. |
| `compare` | `true` compares the new setup file with the one of the latest release (`bach diff`): files added, removed and changed, and how much bigger or smaller it got. The report goes to the job summary and, on a pull request, into one comment that later pushes update. The release's file is found by name: `Hello-Setup-1.4.0.exe` is looked for as `Hello-Setup-*.exe`. Nothing is compared when there is no earlier release or it has no such file. The comment needs `pull-requests: write`; a pull request from a fork gets only the summary. |
| `tag` | The release's tag (default `v` plus the version). |
| `prerelease`, `draft`, `latest` | How the release is published. |
| `version` | Which Bach Installer builds: `0.2.1` exactly, `0.2` or `0` for the newest of that line, `latest`. Default: the tag you pinned the action with. |

Outputs: `setup`, `version`, `feed`, `tag`, `release-url`, `bach-version`.

### Pick the Bach Installer version

`usebach/bach-installer@v0` always builds with the newest 0.x release of Bach Installer.
`@v0.2` stays on 0.2.x and `@v0.2.1` never moves. A full commit SHA never
moves either: it builds with the Bach Installer version in that commit's `Cargo.toml`,
so the pinned code and the program it downloads belong together. `version:`
picks another one. The install scripts take the same: `BACH_VERSION=0.2`
installs the newest 0.2.x.

## What the installed app does

With `[updates]`, the install page offers **Automatic updates**, ticked
(`auto = false` offers it unticked, `"always"` and `"never"` decide for the
user; see [`[updates]`](config.md#updates)). Ticked, setup adds a per-user
Windows scheduled task, `Bach\<app id>`, that runs
`uninstall.exe --auto-update` every four hours (spread by up to 20 minutes, so
machines don't all ask at once) and as soon as it can after a time it missed
because the PC was off. It needs no administrator rights and
runs hidden. Each run:

1. reads the feed (a failed read just waits for the next run),
2. downloads a newer setup file into the background folder, and keeps it,
3. installs it only when the app is not running, otherwise leaves it ready
   for the next run.

Updates never close a program somebody is working in. The box on the install
page is also how the user turns them off again; unticking it on a later
update deletes the task, and so does uninstalling.

## What your app can do

The installed `uninstall.exe` answers these. They work with or without the
scheduled task:

| Command | Result |
| --- | --- |
| `--check-update` | Says whether a newer version is out. Exit code `10`: yes, `0`: up to date (or a [staged rollout](#staged-rollouts) hasn't reached this computer). |
| `--check-update --json` | The same as one line of JSON: `app`, `current`, `latest`, `available`, `downloaded`, `notes`, `size`. |
| `--download-update` | Downloads it into the background folder and keeps it ready. Exit code `10`: ready, `0`: up to date. |
| `--apply-update` | Installs what was kept ready. `--wait-for PID` lets that process (your app) end first, `--launch` starts the app again after the update. Exit code `0`: installed, or nothing was waiting. |
| `--update` | Downloads it (or uses the one kept ready) and opens its setup window. `--silent` installs without a window and exits with the setup's own code. `--wait-for PID` works here too. |
| `--auto-update` | One background round, as the scheduled task runs it. |

A typical app, with the updating done while it isn't in the way:

1. At start, in the background: run `uninstall.exe --download-update`.
2. When the user quits (or picks "Restart to update"): start
   `uninstall.exe --apply-update --wait-for <your process id> --launch` and exit.

The download finishes while the app is used, so closing it costs a few
seconds. A "Later" button needs no code: leave the update kept, and the next
start or the scheduled task takes it.

A protected app's update runs read the key kept at install time; to give one
yourself, use `--key-file FILE` or the `BACH_ACCESS_KEY` environment variable.

Everything the updater does is logged to `%TEMP%\<app id>-update.log`.

A download failure is reported from the HTTP status the server really sent,
not from digits in curl's message: "the server refused the access key" appears
only for a real 401 or 403, and a resumed download counts as complete only on a
real 416. A key with a space, a quote or a backslash is refused with a sentence
instead of being cleaned up and sent as a different key.

## It survives interruptions

- **The setup window is closed:** nothing was changed. The download stays,
  so the next try needs no network.
- **The download is cut off:** it continues where it stopped, and is
  checked against the feed's SHA-256.
- **The app is open:** the update stays ready and installs when it closes.
  A silent install of a file in use fails with exit code `1` and keeps the
  earlier version.
- **The PC is switched off, or setup crashes, in the middle:** files move in
  together, with a journal in the install folder. The next install, update
  or removal (and each background round) puts back what was half moved, and
  new files are flushed to disk before the old ones go. If a file can't be put
  back because a program holds it, the journal stays and setup says which file
  to free before it tries again.
- **Something else holds a file** (a virus scanner just after the app closed):
  `--apply-update` tries up to three times, five seconds apart, and keeps the
  download for the next round if it still can't.

## What is checked

Downloads are https only, redirects included, through Windows' own
`curl.exe`. Before a downloaded setup file runs, again right before it runs
and not only after downloading, it must match the feed's SHA-256, be a setup
file for the same app id, and carry the version the feed says. A staged file
is never trusted just because it is in the folder.

Limits, so nobody is surprised:

- The SHA-256 comes from the same place as the file. It guards against
  damage and a wrong file, not against someone who controls your release
  page. Sign your setup file so Windows shows your name ([signing](signing.md)).
- Unless you offer an [update patch](#update-patches), the download is the
  whole setup file each time.
- A private GitHub repository can't serve the feed (the app has no GitHub
  token to send). Use a server that checks the access key instead: see
  [private updates](#private-updates).

## Mirrors

If the host of your feed is down, the app can ask another:

```toml
[updates]
feed = "https://example.com/hello/latest.json"
download_url = "https://example.com/hello/{file}"
mirrors = ["https://mirror.example.net/hello/latest.json"]
```

A mirror is a full feed with its own download link; write it with
`bach feed SETUP.exe --url https://mirror.example.net/hello/Hello-Setup-1.4.0.exe --out mirror/latest.json`
and upload it there. The **first feed that answers decides which version is
newest.** A mirror can't say "there's a newer one"; it only offers another
place to download the *same* file (same version and SHA-256), tried when the
first download fails. That way a weaker mirror can't widen what installed apps
trust. When the main feed is down, the first mirror that answers is trusted
like the main feed would be, so list only mirrors you run as carefully as the
main one, and sign your setup files.

## Signed feeds

A feed names a download and its SHA-256, and both come from the same host, so
whoever controls that host controls the update. A publisher key takes that
away:

```bash
bach feed-key
```

writes the private key to `feed-key.private` (it must not exist yet; on Linux
and macOS it is readable by you alone, on Windows it has the rights of its
folder, so keep it in one only you can read, and out of Git) and prints the line for `bach.toml`:

```toml bach.toml
[updates]
public_key = "3f9a…64 hex digits…"
```

The public key goes into every setup file built from then on. `bach feed` signs
each entry with the private key, which it reads from the `BACH_FEED_KEY`
environment variable (the key itself) or `BACH_FEED_KEY_FILE` (a file whose
first line is the key). Without the key `bach feed` refuses, and `bach build`
still builds but warns that `latest.json` is unsigned. A key that doesn't belong
to the public key is refused too.

An installed app whose setup file has a public key accepts only feed entries
with a valid signature, whichever feed or mirror served them: a changed link,
SHA-256, size, rollout or patch, an unsigned entry or one signed by another key
is ignored with a line in the log. Release notes are not signed. Old feeds without
a signature keep working for apps without a `public_key`. An app that has one
and receives an unsigned feed stops updating, so run `bach feed` with the key
before you publish. An app only trusts the keys its own setup file carries, so
keep the private key safe, and change it in two steps (below) if you must.

### Keeping a signed feed fresh

A signature proves who wrote an entry, not when. A host that was taken over
can keep serving an old entry that is still validly signed: installed copies
then never see the newer version, or are held at an older one that has a known
bug. `bach feed` therefore signs the time it ran into every entry
(`"published": "2026-11-17T06:36:00Z"`, UTC; `SOURCE_DATE_EPOCH` sets it for a
rebuild on purpose). An app that sets

```toml bach.toml
[updates]
public_key = "3f9a…64 hex digits…"
max_age_days = 90
```

ignores a signed entry that is older than that, one with no time, and one that
claims to be from more than a day ahead (a clock that is wrong, or a feed that
lies). The time is part of the signature: taking it off or changing it makes
the entry invalid. Left out, an entry never expires, as before.

The catch is yours: an entry only stays fresh when you sign it again, so run
`bach feed` on a schedule (for example a weekly workflow that signs the
current release again and uploads `latest.json`), or installed copies stop
seeing updates after `max_age_days`. Feeds signed before this existed have no
time; apps with a limit refuse them, so sign again before you build with it.

### Beta versions

An app can offer pre-release versions to the people who ask for them:

```toml bach.toml
[updates]
feed = "https://example.com/hello/latest.json"            # stable
beta_feed = "https://example.com/hello/latest-beta.json"  # beta
```

Build the beta (`version = "2.0.0-beta.1"`) and write its feed to its own file:
`bach feed dist/Hello-Setup-2.0.0-beta.1.exe --out latest-beta.json`. The stable
feed stays as it is, so nobody gets a beta by accident. Each installation
chooses for itself, with the installed `uninstall.exe` (your app can run it):

```bash
uninstall.exe --channel beta      # follow the beta feed from now on
uninstall.exe --channel stable    # back to the stable feed
uninstall.exe --channel show      # say which one this is
```

The choice lives under the user's own `HKCU\Software\Bach\Channel`, survives
updates and repairs, and is deleted when the app is removed. The next
`--check-update`, `--download-update` or `--update` (and the background task)
asks only the beta feed (no mirrors), with the same `public_key` and `signer`
checks. An app without `beta_feed` answers `--channel beta` with exit code `1`.
Going back to stable never installs an older version by itself: a beta that is
installed stays until a stable version newer than it is out.

### Signed updates (Authenticode)

`public_key` signs the feed. `signer` checks the *file*: with

```toml bach.toml
[updates]
signer = "Example Inc."     # as the signing certificate shows it
```

the installed app runs a downloaded update only when Windows says its
Authenticode signature is valid and the certificate's common name or
organisation is that text (case ignored, the whole name). It asks Windows'
own PowerShell (`Get-AuthenticodeSignature`) right before it installs, and
again before it opens an update's window. An unsigned file, a signature Windows
doesn't trust or a file signed by anyone else is deleted from the download
folder and not run, with the reason in the log. The online installer
checks the file it downloads the same way before it installs from it. This also protects against a
feed host that was taken over: it can name any file, but not sign as you.

A plain name matches any certificate that has it as its common name or
organisation, and a certificate authority can issue that to another company of
the same name. To pin the company, give the whole subject (the certificate's
"Issued to" details, as `Get-AuthenticodeSignature FILE | % { $_.SignerCertificate.Subject }`
prints it): `signer = "CN=Example Inc., O=Example, L=Berlin, C=DE"` has to match
the certificate's subject pair for pair, so another company of the same name,
in another place, doesn't. A renewed certificate keeps its subject, so the
setting survives renewals; a new company name needs a release that carries
both, which is not supported yet: change `signer` in a version that you
distribute by hand.

`bach build --sign` and `bach sign` check the same thing after signing: if the
signed file's certificate isn't the `signer`, the build stops, because every
installed copy would refuse the update. Use it together with `[signing]` and sign
every setup file you publish, patches too. A build without `--sign` still
works; its update would be refused by apps that have a `signer`. The first
version with `signer` has to be installed by hand (older copies don't know the
setting).

### Signed updates on a Mac

On a Mac the same `signer` asks `codesign` about the Setup app in the downloaded
`.dmg` or `.zip`. The signature has to cover every file (`--verify --deep
--strict`) and come from a Developer ID Application certificate that Apple
issued. A self-signed or ad hoc signature, an Apple Development or Apple
Distribution certificate and one from a certificate authority of your own are
refused, whatever name they carry.

The name is read from that certificate: `Example Inc.` for
`Developer ID Application: Example Inc. (ABCDE12345)`. A Mac knows only two
parts of a whole subject, the name as `CN` and the team id as `OU`, so
`signer = "CN=Example Inc., OU=ABCDE12345"` pins your team. A name with a comma
is quoted there (`CN="Example, Inc.", OU=ABCDE12345`) and written as it is when
`signer` is the plain name.

One `signer` serves Windows and macOS. A whole subject matches only the platform
it was written for, so an app that ships on both needs the plain name, and both
certificates have to carry it.

`bach build` asks the same question before you publish. With `--sign`, right
after it signed the Setup app, it stops when installed copies would refuse the
certificate: an ad hoc signature, one that isn't an Apple-issued Developer ID
Application, or another publisher's name. Without `--sign` it writes the file
and warns that installed copies will refuse it, which is what a publisher who
signs only for Windows sees. Sign every Setup app you publish with `--sign`
and your Developer ID
([Sign the macOS Setup app](signing.md#sign-the-macos-setup-app)). Neither
check asks whether the app was notarized.

`bach sign` on a `.dmg` or `.zip` looks at the Setup app inside too, after
your `[signing]` program ran: signing the file around the app changes nothing
for an installed copy. Away from a Mac it can't look, and says so in a warning.

### Changing the key

A key has to be replaced when it leaks, or is lost. Because installed copies only
trust what they were built with, the new key has to reach them while the old one
still works:

1. Make the new key with `bach feed-key` and keep the old one. Build a release
   whose `bach.toml` has both, the old key as `public_key` and the new one as
   `next_public_key`, and sign its feed with the **old** private key. Computers
   that update to it now trust both keys.
2. Wait until most computers have that release. Then build the next one with
   `public_key` set to the new key (and no `next_public_key`) and sign its feed
   with the **new** private key. Computers on the release from step 1 accept it.
   Computers that never took step 1 still hold only the old key and refuse it,
   so they need a new download of the setup file.

If the old key leaked, step 1's feed can be forged by whoever has it until
everyone has moved on; the key cannot be revoked remotely. `bach feed` accepts
a private key that belongs to either of the two keys.

## Staged rollouts

To ship a risky release to few people first, write the feed with a rollout:

```bash
bach feed "dist/Hello-Setup-1.4.0.exe" --rollout 10
```

```json latest.json
{ "version": "1.4.0", "url": "…", "sha256": "…", "rollout": 10 }
```

Each computer has a fixed number from 0 to 99 for each version (made from Windows'
machine id, the app's id and the version, so the first computers differ from
release to release and are not always the same test group; installed copies
built by an older Bach Installer keep using the number without the version), and takes the update by itself when that number is below the
rollout. Raise it (`--rollout 50`), then write the feed again without
`--rollout` for everybody. A computer that was in the first ten percent stays
in all later steps. Computers outside it say "up to date" for
`--check-update`, `--download-update` and the scheduled task; `--update`, which
a person asks for, always installs. The online installer ignores rollouts:
new installs get the newest. To pull a bad release, publish the feed of the
last good version again: apps only ever move up, so an app that already took
the bad one waits for a higher version.

## Update patches

A big app that updates often (an Electron app with its own Chromium, say) can
spare its users most of the download. `bach build --patch-from` also writes a
**patch**: a setup file that carries only the files that changed since an
earlier version.

```bash
bach build --patch-from "old/Hello-Setup-1.3.0.exe"
```

```console
Wrote dist\Hello-Setup-1.4.0.exe (37.8 MB)
Wrote dist\Hello-Setup-1.4.0-patch-from-1.3.0.exe (17.7 MB), the update patch from 1.3.0: 1 of 4 files changed
  Update feed  dist\latest.json
```

The feed then offers it next to the full file:

```json latest.json
{
  "version": "1.4.0", "url": "…/Hello-Setup-1.4.0.exe", "sha256": "…",
  "delta": { "from": "1.3.0", "url": "…/Hello-Setup-1.4.0-patch-from-1.3.0.exe", "sha256": "…" }
}
```

What the installed app does with it:

- It downloads the patch only when 1.3.0 is installed and every installed file
  has a checksum on record (setup files made with this Bach Installer version or newer do).
  Every other computer downloads the full file, as before.
- The patch is a setup file for the same app, checked like any update (SHA-256
  against the feed, same app id, the feed's version). Before it places anything
  it checks that every file it left out is in the install folder, byte for byte
  (size and SHA-256). If one isn't, because the user or something else changed
  it, nothing is touched and the app downloads the full file instead and installs
  that. An update never ends up half done.
- The window of `--update` (without `--silent`) always uses the full file: it
  can't fall back by itself. Silent updates, the background task and
  `--download-update` use patches.

Things to know:

- A file that changed goes into the patch as **what differs from its earlier
  version** (a binary delta), when that is clearly smaller than the file: a
  program that changed in a few places costs kilobytes, not the whole program.
  See [Binary deltas](#binary-deltas). A file that changed completely, or one
  under 64 KB, goes in whole.
- A patch is a setup file itself, so it carries the setup runtime (about
  10 MB). `bach build` only puts a patch in the feed when it's at least a fifth
  smaller than the full file, and says so when it leaves it out. Force it with
  `bach feed … --patch`.
- Build the patch from the setup file you published for that version (keep
  it, or download it from your release). You can build one patch per older
  version you want to support, and offer one in the feed: the most recent
  version most people have.
- After signing, write the feed again and pass the signed patch too:
  `bach feed dist/Hello-Setup-1.4.0.exe --patch dist/Hello-Setup-1.4.0-patch-from-1.3.0.exe`
  (the patch's link comes from `bach.toml`, or `--patch-url`). With `--sign`,
  `bach build` signs the patch too.
- With the GitHub Action, pass `args: --patch-from OLD.exe`. The action signs
  the patch with `sign-command` like the setup file, writes the feed again with
  `--patch` (so a signed or annotated feed keeps offering it), uploads it to the
  release next to the setup file and gives its path as the `patch` output.
- An app with `[access]` gets a patch that is encrypted with the same key.
- The online installer and a first install always use the full file.

### Binary deltas

When `bach build --patch-from` finds a file that is in both versions and
changed, and the file is at least 64 KB, it makes a delta of it: the new file
stored relative to the old one, using zstd's patch mode with a window as large
as both files and long-distance matching, so a block of code that moved a few
kilobytes along still matches. A 6 MB program that changed by one byte and 900
inserted bytes:

| | Archive inside the setup file |
| --- | --- |
| Full setup file | 6,001,092 bytes |
| Update patch | 1,075 bytes |

(The patch is a setup file, so on disk it is the setup runtime, about 10 MB,
plus that archive. A patch that only needs to replace one big program is
about the runtime's size, whatever the program's.)

- **It is exact.** A delta is made from one file and records that file's
  SHA-256. At update time setup reads the installed file, refuses the delta
  unless it is exactly that file, and checks the result against the new file's
  SHA-256 in the manifest like any other file. A program the user changed
  doesn't get a patched copy: nothing is touched, the update says it was made
  for another version, and the app takes the full file instead.
- **A delta is used only if it is under four fifths of the file.** Otherwise the
  whole file goes in, so a patch is never bigger than it would have been.
- **Memory.** Both files are held in memory while a delta is made or applied:
  a file over 512 MiB (or an earlier file of that size) goes in whole, and a
  build holds at most 2 GiB of earlier files at a time.
- **The earlier setup file has to be readable.** An earlier setup file that is
  locked with `[access]`, or is itself a patch, has no files to make deltas from
  (a note says so), and the patch holds whole files, as before.
- `bach verify` checks a patch's archive and every whole file in it, and steps
  over the deltas, which only the earlier files can turn into a file.
- Setup files in the wild keep working: a delta is a new kind of entry that only
  patches have, read by the runtime inside the patch itself, so an installed
  app needs no update to take one.

## Private updates

Without [`[access]`](config.md#access), the feed and the setup file must be
readable by anyone. For a private app, add:

```toml
[access]                  # encrypts the files with a key, asks for it at install
get_key_url = "https://example.com/buy"

[updates]
feed = "https://downloads.example.com/hello/latest.json"
download_url = "https://downloads.example.com/hello/{file}"
```

With `send_key` (the default when `[updates]` is set), the installed app sends
the key it was installed with as `Authorization: Bearer <key>` with every
request to the feed and the download. Your server decides: answer `401` or
`403` and the update is refused with "that key doesn't fit". The key is kept
sealed for the Windows user and goes only to the host of `feed` (the main feed,
which is in the signed setup file): a feed can name any download link, so a
download or a mirror on another host never gets it, and a redirect to another
host (a storage bucket's signed link) is followed without it. Put your private
download on the same host as the feed. Because the files are also encrypted with the key, the setup file is safe
on a public host too. See [`[access]`](config.md#access) for what this does and
does not protect.

## Your own server

Without GitHub, give the feed and where the file lives:

```toml
[updates]
feed = "https://example.com/hello/latest.json"
download_url = "https://example.com/hello/{file}"   # {file}: the setup file's name
```

`bach build` writes `latest.json` next to the setup file. After signing, write
it again with `bach feed SETUP.exe` (it reads the link from `bach.toml`, or
pass `--url`; `--notes` or `--notes-file` says what's new; `--rollout N` is
a [staged rollout](#staged-rollouts)). Upload both files.

## One feed for x64 and ARM64

With [`[arch.arm64]`](config.md#archarm64) in `bach.toml`, `bach build` writes
an x64 and an ARM64 setup file and one `latest.json` that lists both:

```json
{
  "version": "1.4.0",
  "url": "https://example.com/Hello-Setup-1.4.0.exe",
  "sha256": "…",
  "size": 11743232,
  "arch": {
    "arm64": {
      "url": "https://example.com/Hello-Setup-1.4.0-arm64.exe",
      "sha256": "…",
      "size": 11324416,
      "signature": "…"
    }
  }
}
```

The top-level link, SHA-256 and size are the x64 file, as before. A computer
takes the entry for its architecture: the online installer asks Windows what
the computer is, and an installed app updates with the architecture it was
installed with. If the listing for that architecture is missing, isn't an https
link or (with `[updates] public_key`) isn't signed by the publisher, the x64
file is taken, which also runs on Windows on ARM.

The `arm64` entry has its own signature, over the app id, the version, the
architecture, the link, the SHA-256 and the size, so a host that swaps the
link can't hand an ARM64 computer a file you didn't sign. The main entry's
signature is the same as before, so apps built before `arch` existed keep
verifying and updating.

## One feed for Windows and macOS

A bach.toml with `[macos]` builds a macOS setup file next to the Windows one, and the
same `latest.json` lists both. The Windows file is the entry's own `url`, `sha256` and
`size`; the macOS file (a `.dmg`, or a `.zip`) is listed under `arch.macos`, with its own
link, SHA-256, size and, with `[updates] public_key`, its own signature. A Mac takes only
`arch.macos`: a feed that doesn't list one offers it nothing, and it never downloads the
Windows file. Windows apps ignore `arch.macos`.

```json
{
  "version": "1.4.0",
  "url": "https://example.com/Hello-Setup-1.4.0.exe",
  "sha256": "…",
  "arch": {
    "macos": { "url": "https://example.com/Hello-Setup-1.4.0.dmg", "sha256": "…", "size": 4194304 }
  }
}
```

`bach build` writes the feed: the Windows build writes it, then the macOS build adds
`arch.macos` to it (and signs that entry), so build both in one run. A build that only has
`[macos]` writes a feed whose own entry is the macOS file. If you build the files separately, `bach feed
Hello-Setup-1.4.0.exe --macos Hello-Setup-1.4.0.dmg` writes it afterwards.

On a Mac the installed `Uninstall <app>.app` does what `uninstall.exe` does: `--check-update`,
`--download-update`, `--apply-update`, `--update` and `--auto-update` (a launch agent runs it every
four hours when "Automatic updates" is on). It downloads the `.dmg`, checks its SHA-256, opens it, copies the Setup app
out and runs that with `--silent`, then removes the copy. `[updates] signer` makes it run only a Setup app
signed with an Apple-issued Developer ID of that name ([Signed updates on a Mac](#signed-updates-on-a-mac)). Update patches (`--patch-from`) and staged
rollouts' Windows-only machine ids are not used for macOS: a rollout uses the Mac's hardware UUID.

