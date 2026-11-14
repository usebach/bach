<p align="center">
  <img src="crates/setup/icon/bach-icon-1024.png" width="72" height="72" alt="Bach logo">
</p>

<h1 align="center">Bach</h1>

<p align="center">
  <strong>One <code>bach.toml</code> in, one <code>Setup.exe</code> out.</strong><br>
  Windows installers with a native window, in-place updates and a clean uninstaller,<br>
  without an installer script and without an administrator prompt.
</p>

<p align="center">
  <a href="docs/getting-started.md">Getting started</a> ·
  <a href="docs/config.md">Config reference</a> ·
  <a href="https://github.com/usebach/bach/releases">Releases</a> ·
  <a href="LICENSE">MIT license</a>
</p>

Bach packs the folder your app builds into, plus a short config file, into a
single setup file. Your users get one calm window in their own language; you
get updates over older installs and an uninstaller in **Settings › Apps**
without writing installer logic.

<p align="center">
  <img src="docs/images/install-light.png" width="440" alt="Bach's install window in light mode, in English: install location, a sample-notes component, desktop shortcut and start-with-Windows options, Cancel and Install buttons">
  <img src="docs/images/install-dark.png" width="440" alt="The same window in dark mode, in German">
</p>

> [!NOTE]
> Bach is early and in active development: **0.1.0**, a pre-release, came out
> on 13 November 2026, and [`main`](https://github.com/usebach/bach/commits/main)
> has moved on since. Requirement
> checks, self-updates from the app, extra shortcuts, logs, the archive
> checksum, "close and continue", keyboard navigation, `bach feed`,
> `bach winget`, project detection in `bach init`, `bach version`,
> `bach workflow` and the `@usebach/cli` package on npm are on `main` and ship
> in the next release; [build from source](#build-from-source) to use them today.

## Quick start

Install the `bach` command. No Rust needed, no administrator rights:

```bash
# Windows, macOS and Linux (Node.js 18 or newer)
npm install --global @usebach/cli
```

No Node.js? One line does it too, and it works the same in CI:

```powershell
# Windows (PowerShell): installs to %LOCALAPPDATA%\Programs\bach-cli and adds it to PATH
irm https://raw.githubusercontent.com/usebach/bach/main/install.ps1 | iex
```

```bash
# macOS or Linux, for building Windows installers in CI
curl -fsSL https://raw.githubusercontent.com/usebach/bach/main/install.sh | sh
```

Then, in your project:

```bash
bach init      # writes a commented bach.toml (and, on GitHub, a release workflow)
bach check     # validates it and lists every file the installer will carry
bach preview   # opens the real window; nothing is installed
bach build     # writes the setup file
```

In a GitHub repository, `bach init` also adds a one-click release: commit it,
then press **Actions › Release › Run workflow**. See [releasing](docs/releasing.md).

A complete config, for a Rust app:

```toml
[app]
name = "Hello"
version = "1.0.0"
publisher = "Hello Inc."
executable = "hello.exe"

[files]
source = "target/release"
exclude = ["*.pdb", "deps/**", "build/**"]
```

```console
$ bach build
Wrote dist\Hello-Setup-1.0.0.exe (9.7 MB)
  Packed       101.8 KB of 216.5 KB (47%), in 0.9s
```

[examples/hello](examples/hello) is a working project to try it on, and
[getting started](docs/getting-started.md) walks through it step by step.

## Why Bach

[NSIS](https://nsis.sourceforge.io), [Inno Setup](https://jrsoftware.org/isinfo.php)
and [WiX](https://wixtoolset.org) can build almost any installer, and ask you
to learn a scripting language or an XML schema to do it. Bach covers the common
case, an app that installs for one user, with settings instead of code:

- **No script.** Everything is a setting in one TOML file with an
  [editor schema](schema/bach.schema.json). Every mistake is reported as one
  sentence naming the setting at fault.
- **No admin prompt.** Apps install per user into `%LOCALAPPDATA%\Programs`
  and only write to the user's own folders and `HKCU`.
- **Updates that can't half-fail.** A newer setup file updates an install in
  place: files are unpacked first and swapped in together, and if one is
  locked, everything is put back. Files the new version no longer ships are
  removed.
- **A window people can read.** 13 languages built in, following the Windows
  display language; light and dark mode; every text replaceable.
- **Builds anywhere.** `bach` runs on Windows, macOS and Linux, so installers
  can come out of any CI. Setup files need no .NET and no Visual C++ runtime.

**Use something else when** you need a machine-wide install or anything that
requires administrator rights, an MSI for Group Policy deployment, Windows
services or drivers, custom install-time scripts, delta updates
([Velopack](https://velopack.io) does those), or a small stub: a Bach setup
file adds about 10 MB to your app.

## What setup files can do

| | |
| --- | --- |
| **Choices** | Desktop and Start menu shortcuts, start with Windows, add to `PATH`, each offered ticked, unticked, always or never; optional components |
| **Integration** | `myapp://` link schemes and file types, never taking over another app's |
| **Look and words** | Your logo, accent colour and icon; light, dark or system theme; any text, per language; a license page |
| **Safety** | Free-space check, archive checksum, one setup per app at a time, "close and continue" when the app is open, data deleted only on request |
| **Automation** | `Setup.exe /S` with `--dir`, `--with`, `--without`, `--log`; quiet uninstall from Settings › Apps |
| **Updates** | Over older installs always; by themselves in the background from GitHub releases (or any https feed), installed when the app is closed, undone cleanly if a power cut interrupts them: see [updates](docs/updates.md) |

Every setting is in the [config reference](docs/config.md).

## Commands

| Command | What it does |
| --- | --- |
| `bach init` | Writes a starter `bach.toml`; fills it in for Rust, Tauri, Electron and .NET projects. In a GitHub checkout it also writes the release workflow (`--no-workflow` skips that). |
| `bach workflow` | Adds, or updates, the one-click release workflow in `.github/workflows/bach-release.yml`. See [releasing](docs/releasing.md). |
| `bach version [patch\|minor\|major\|X.Y.Z]` | Lists the version in every file that has one, or raises it in all of them at once: `bach.toml`, `package.json` and its lock file, `Cargo.toml` and `Cargo.lock`, Tauri's config, `.csproj`. |
| `bach check` | Validates the config and lists every file and option, with warnings. |
| `bach preview` | Opens the window without installing; `--uninstall`, `--theme dark`, `--lang de`. |
| `bach build` | Writes the setup file; `--fast` compresses less, `--out` picks the path, `--app-version v1.4.0` sets the version (from a release tag). |
| `bach inspect FILE` | Shows what a setup file or an installed `uninstall.exe` carries. |
| `bach doctor` | Says whether everything a build needs is in place. |
| `bach feed FILE` | Writes the update feed for a (signed) setup file; the link comes from `bach.toml` (or `--url`), `--notes-file` says what's new. |
| `bach winget FILE --url URL` | Writes the manifests that list a setup file in winget. |

## Other ways to install

- **npm** (`npm install --global @usebach/cli`, or `--save-dev` for one
  project and `npx bach`): update with `npm update --global @usebach/cli`,
  remove with `npm uninstall --global @usebach/cli`.
- **For every user:** the install scripts install per user and never ask for
  administrator rights. Ask for a system-wide install with
  `$env:BACH_SCOPE = 'system'` before the PowerShell line (to
  `%ProgramFiles%\bach-cli`; Windows asks for permission once, and declining
  installs it for you alone) or `sh -s -- --system` after the shell one (to
  `/usr/local/bin`, using `sudo` for that copy only).
- **Scoop:** `scoop install https://raw.githubusercontent.com/usebach/bach/main/scoop/bach.json`
- **GitHub Actions**, on any runner. Build and publish a release for every
  pushed tag like `v1.4.0` (the tag is the app's version; installed apps
  update from it, see [updates](docs/updates.md)):

  ```yaml
  permissions:
    contents: write
  steps:
    - uses: actions/checkout@v4
    # ... build your app ...
    - uses: usebach/bach@v0
      with:
        config: bach.toml
        release: true
  ```

  Or only build, and publish the file yourself:

  ```yaml
  - uses: usebach/bach@v0
    id: bach
    with:
      config: bach.toml
  - uses: actions/upload-artifact@v4
    with:
      path: ${{ steps.bach.outputs.setup }}
  ```

  `@v0` follows the newest 0.x release, `@v0.2` the newest 0.2.x, `@v0.2.1`
  never moves; the Bach version that builds follows what you pin.

- **By hand:** download `bach-<version>-windows-x64.zip` from
  [releases](https://github.com/usebach/bach/releases) and keep `bach.exe` and
  `bach-setup.exe` together.

Coding agents can learn all of this from [skills/bach/SKILL.md](skills/bach/SKILL.md)
(`npx skills add usebach/bach`).

## Build from source

With Rust (stable) and the Visual Studio C++ build tools:

```bash
git clone https://github.com/usebach/bach.git
cd bach
cargo build --release
```

That gives `target\release\bach.exe` and `target\release\bach-setup.exe`.
[How it works](docs/how-it-works.md) explains the setup file format and the
safety rules; [AGENTS.md](AGENTS.md) covers the code layout, checks and
conventions for changes.

## Contributing

Bug reports, translations and pull requests are welcome; start with
[CONTRIBUTING.md](CONTRIBUTING.md). Questions go to [SUPPORT.md](SUPPORT.md),
security problems to the [security policy](SECURITY.md), and everyone here
follows the [Code of Conduct](CODE_OF_CONDUCT.md).
