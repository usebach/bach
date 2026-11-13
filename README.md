<p align="center">
  <img src="crates/setup/icon/bach-icon-1024.png" width="56" height="56" alt="">
</p>

<h1 align="center">Bach</h1>

<p align="center">
  Modern, clean Windows installers from one small config file.
</p>

<p align="center">
  <a href="https://github.com/usebach/bach/releases"><strong>Download Bach</strong></a>
  ·
  <a href="docs/getting-started.md">Getting started</a>
  ·
  <a href="docs/config.md">Config reference</a>
</p>

Bach turns a folder with your app into a single `Setup.exe` with a calm,
native-looking installer window: light and dark mode, smooth progress,
readable errors, and no administrator prompt. The same file updates an
existing install, and the uninstaller it adds to **Settings › Apps** removes
everything again.

- **One file to write.** A `bach.toml` with your app's name, version and
  folder. Everything else has sensible defaults.
- **Per-user installs.** Apps go to `%LOCALAPPDATA%\Programs`, with Start
  menu and desktop shortcuts. No UAC prompt, ever.
- **Updates built in.** Running a newer setup file over an install updates it
  in place and removes files the old version had and the new one doesn't.
- **A clean uninstaller.** It removes exactly the files it installed, the
  shortcuts and the Apps entry, and deletes the app's data only when the
  user ticks the box.
- **Your look.** Your icon on the setup file, your logo in the window, your
  accent colour on the buttons.
- **Signable.** Sign the setup file with `signtool` like any other program;
  the payload is covered by the signature.

## Quick start

1. [Download Bach](https://github.com/usebach/bach/releases) and unzip it.
   Keep `bach.exe` and `bach-setup.exe` in the same folder.
2. In your project, write a starter config:

   ```bash
   bach init
   ```

3. Fill in `[app]` (name, version, publisher, the `.exe` to start) and point
   `[files] source` at the folder your app's build ends up in:

   ```toml
   [app]
   name = "Hello"
   version = "1.0.0"
   publisher = "Hello Inc."
   executable = "hello.exe"

   [files]
   source = "build"
   ```

4. Look at the window without installing anything, then build:

   ```bash
   bach preview
   ```

   ```bash
   bach build
   ```

   You get `dist\Hello-Setup-1.0.0.exe`, ready to hand out.

The [getting started guide](docs/getting-started.md) goes through it step by
step, and the [config reference](docs/config.md) lists every setting.

## Commands

| Command | What it does |
| --- | --- |
| `bach init` | Writes a commented `bach.toml` to start from. |
| `bach check` | Reads `bach.toml` and lists every file the installer would install. |
| `bach build` | Writes the setup file. `--fast` compresses less, `--out` picks the file. |
| `bach preview` | Opens the installer window as it will look; nothing is installed. `--uninstall` shows the uninstaller, `--theme dark` the other theme. |
| `bach inspect FILE` | Shows what a setup file (or an installed `uninstall.exe`) carries. |

## How it works

A setup file is `bach-setup.exe`, the installer window, with your app's
files compressed and appended to it. When it runs, it finds them at its own
end and unpacks them. There is no script language and nothing to compile.
[How it works](docs/how-it-works.md) has the details, including the file
format.

## Build from source

You need Rust (stable) and the Visual Studio C++ build tools on Windows.

```bash
git clone https://github.com/usebach/bach.git
```

```bash
cargo build --release
```

That gives you `target\release\bach.exe` and `target\release\bach-setup.exe`.
To build the example installer end to end:

```bash
powershell -ExecutionPolicy Bypass -File scripts\build-example.ps1
```

`bach` itself also runs on macOS and Linux, so you can build installers in
any CI. It needs a `bach-setup.exe` from a release (`--runtime`); outside
Windows it keeps Bach's icon on the setup file.

## Project layout

```
crates/pack    bach-pack: bach.toml, the payload format and the path rules
crates/cli     bach-cli: the `bach` command
crates/setup   bach-setup: the installer and uninstaller window (GPUI)
examples/hello a tiny app and the bach.toml that packages it
docs/          guides and the config reference
scripts/       release, example and asset scripts
```

## License

[GPL-3.0](LICENSE). The installer window is built with
[GPUI](https://www.gpui.rs/), [Inter](crates/setup/assets/fonts/LICENSE.txt)
and [Tabler Icons](crates/setup/assets/icons/LICENSE).
