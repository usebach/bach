# Updates

Bach apps can keep themselves up to date. This page covers publishing
updates from GitHub, what the installed app does with them, and how an
update survives a closed window, a closed app or a power cut.

There are two ways to update:

- **By hand.** Hand out a newer setup file. Running it over an install
  updates it in place (see [getting started](getting-started.md#6-ship-updates)).
- **By themselves.** The installed app reads an update feed, downloads the
  newer setup file in the background and installs it when the app is closed.
  The rest of this page is about that.

## Publish updates from GitHub

Add one line to `bach.toml`:

```toml
[updates]
github = "owner/repo"
```

Then publish with the Bach action. This workflow builds on every pushed tag
like `v1.4.0`:

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
      - uses: usebach/bach@v0
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
| `sign-command` | A PowerShell command that signs `$env:BACH_SETUP_FILE`. It runs before the feed is written, so the feed has the signed file's SHA-256. |
| `tag` | The release's tag (default `v` plus the version). |
| `prerelease`, `draft`, `latest` | How the release is published. |
| `version` | Which Bach builds: `0.2.1` exactly, `0.2` or `0` for the newest of that line, `latest`. Default: the tag you pinned the action with. |

Outputs: `setup`, `version`, `feed`, `tag`, `release-url`, `bach-version`.

### Pick the Bach version

`usebach/bach@v0` always builds with the newest 0.x release of Bach.
`@v0.2` stays on 0.2.x and `@v0.2.1` never moves. A full commit SHA never
moves either: it builds with the Bach version in that commit's `Cargo.toml`,
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
| `--check-update` | Says whether a newer version is out. Exit code `10`: yes, `0`: up to date. |
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

Everything the updater does is logged to `%TEMP%\<app id>-update.log`.

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
  new files are flushed to disk before the old ones go.
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
- The download is the whole setup file each time, no deltas.
- A private repository can't serve the feed.

## Your own server

Without GitHub, give the feed and where the file lives:

```toml
[updates]
feed = "https://example.com/hello/latest.json"
download_url = "https://example.com/hello/{file}"   # {file}: the setup file's name
```

`bach build` writes `latest.json` next to the setup file. After signing, write
it again with `bach feed SETUP.exe` (it reads the link from `bach.toml`, or
pass `--url`; `--notes` or `--notes-file` says what's new). Upload both files.
