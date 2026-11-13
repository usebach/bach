<p align="center">
  <img src="crates/setup/icon/bach-icon-1024.png" width="64" height="64" alt="">
</p>

<h1 align="center">Bach</h1>

<p align="center">
  <strong>Modern, clean Windows installers from one small config file.</strong><br>
  No script language. No admin prompt. One <code>Setup.exe</code>.
</p>

<p align="center">
  <a href="#install">Install</a>
  ·
  <a href="docs/getting-started.md">Getting started</a>
  ·
  <a href="docs/config.md">Config reference</a>
  ·
  <a href="skills/bach/SKILL.md">Agent skill</a>
</p>

---

Bach turns a folder with your app into a single setup file with a calm,
native-looking install window. The same file updates an existing install,
and the uninstaller it adds to **Settings › Apps** removes everything again.

```toml
[app]
name = "Hello"
version = "1.0.0"
publisher = "Hello Inc."
executable = "hello.exe"

[files]
source = "build"
```

```bash
bach build
```

That's a complete installer: `dist\Hello-Setup-1.0.0.exe`.

## Why Bach

- **One file to write.** A `bach.toml` with sensible defaults. Every mistake
  is one sentence naming the setting at fault. Nothing to learn first.
- **Calm for your users.** One window, honest progress, readable errors with
  "Try again", light and dark mode, and every row of choices is one big
  click target.
- **Speaks their language.** 13 languages built in, following Windows'
  display language, or exactly the ones you list.
- **Yours, all of it.** Logo, accent colour, icon, every text on every page,
  per language. No Bach branding unless you want it.
- **Per-user installs.** Apps go to `%LOCALAPPDATA%\Programs`. No UAC prompt,
  ever, and nothing written outside the user's own folders and `HKCU`.
- **Updates built in.** A newer setup over an install updates it in place,
  keeps the user's choices, and removes files the old version had and the new
  one doesn't. Files are unpacked first and swapped in all at once, so a
  failed update leaves the old version exactly as it was.
- **Safe uninstalls.** Removes exactly what it installed. Deletes the user's
  data only when they tick the box, and only inside their profile.
- **Runs everywhere.** Setup files need nothing installed first: no .NET, no
  Visual C++ runtime. `bach` itself runs on Windows, macOS and Linux.

## Features

| | |
| --- | --- |
| **Options** | Desktop and Start menu shortcuts, start with Windows, add to `PATH`, each offered ticked, unticked, always or never |
| **Components** | Optional parts (samples, language packs, plugins) the user can leave out |
| **Integration** | `myapp://` link schemes and file types, without taking over another app's |
| **License** | An accept page before installing, per language if you like |
| **Look** | Your logo (SVG or PNG), accent colour, light / dark / system theme, your icon on the setup file |
| **Text** | Replace any text in the window, in one language or per language |
| **Silent mode** | `Setup.exe /S` for IT tools, with `--dir`, `--with`, `--without`; quiet uninstall from Settings › Apps |
| **Safety checks** | Free space checked before unpacking, one setup per app at a time, a running app detected before anything changes |
| **Signing** | Sign with `signtool` like any program; the payload is covered by the signature |

## Install

No Rust needed.

**Windows** (PowerShell):

```powershell
irm https://raw.githubusercontent.com/usebach/bach/main/install.ps1 | iex
```

**macOS / Linux** (for building Windows installers in CI):

```bash
curl -fsSL https://raw.githubusercontent.com/usebach/bach/main/install.sh | sh
```

Or [download the zip](https://github.com/usebach/bach/releases) and keep
`bach.exe` and `bach-setup.exe` in one folder. `bach doctor` tells you if
anything is missing.

## Quick start

```bash
bach init        # writes a commented bach.toml
bach check       # lists every file the installer will install
bach preview     # opens the real window; installs nothing
bach build       # writes dist\<name>-Setup-<version>.exe
```

`bach preview --lang de --theme dark` shows the German, dark window;
`bach preview --uninstall` the uninstaller. The
[getting started guide](docs/getting-started.md) goes through it step by
step, and the [config reference](docs/config.md) lists every setting.

## A fuller config

```toml
[app]
name = "Hello"
version = "1.4.0"
publisher = "Hello Inc."
executable = "hello.exe"
description = "Say hello in style."
icon = "assets/hello.ico"

[files]
source = "build"
exclude = ["*.pdb"]

[installer]
accent = "#2563eb"
languages = ["en", "de"]
license = "LICENSE.txt"
start_with_windows = false     # offered, unticked
add_to_path = "always"

[[components]]
id = "samples"
title = { en = "Sample projects", de = "Beispielprojekte" }
files = ["samples/**"]

[[protocols]]
scheme = "hello"

[text]
footer = "Made with care by {publisher}"

[uninstall]
data = ['%APPDATA%\Hello']
```

## Commands

| Command | What it does |
| --- | --- |
| `bach init` | Writes a commented `bach.toml` to start from. |
| `bach check` | Reads `bach.toml`, lists every file and option, and points out likely mistakes. |
| `bach build` | Writes the setup file. `--fast` compresses less, `--out` picks the file. |
| `bach preview` | Opens the installer window as it will look; nothing is installed. `--uninstall`, `--theme dark`, `--lang de`. |
| `bach inspect FILE` | Shows what a setup file (or an installed `uninstall.exe`) carries. |
| `bach doctor` | Checks that everything a build needs is in place. |

## For AI agents

[`skills/bach/SKILL.md`](skills/bach/SKILL.md) teaches coding agents how to
install Bach, write a `bach.toml` for any stack, check it, build it and verify
it safely. Add it to an agent with:

```bash
npx skills add usebach/bach
```

## How it works

A setup file is `bach-setup.exe`, the installer window, with your app's files
compressed (zstd) and appended to it. When it runs, it finds them at its own
end and unpacks them. [How it works](docs/how-it-works.md) has the details,
including the file format and the safety rules.

## Build from source

You need Rust (stable) and the Visual Studio C++ build tools.

```bash
git clone https://github.com/usebach/bach.git
```

```bash
cargo build --release
```

That gives you `target\release\bach.exe` and `target\release\bach-setup.exe`.
To build the example installer end to end:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-example.ps1
```

```
crates/pack    bach-pack: bach.toml, the payload format, texts and path rules
crates/cli     bach-cli: the `bach` command
crates/setup   bach-setup: the installer and uninstaller window (GPUI)
examples/hello a tiny app and the bach.toml that packages it
skills/bach    the agent skill
docs/          guides and the config reference
```

## License

[GPL-3.0](LICENSE). The installer window is built with
[GPUI](https://www.gpui.rs/), [Inter](crates/setup/assets/fonts/LICENSE.txt)
and [Tabler Icons](crates/setup/assets/icons/LICENSE).
