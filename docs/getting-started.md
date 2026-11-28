# Getting started with Bach Installer

This guide takes you from a built app to a `Setup.exe` you can give to other
people. You don't need Rust or an installer script. The same walkthrough, with
more detail and a sample app, is in the
[quickstart](https://usebach.app/docs/quickstart).

## 1. Install Bach Installer

With Node.js 18 or newer, on Windows, macOS or Linux:

```bash
npm install --global @usebach/cli
```

Without Node.js, in PowerShell:

```powershell
irm https://raw.githubusercontent.com/usebach/bach-installer/main/install.ps1 | iex
```

The script puts `bach.exe` (the command you run) and `bach-setup.exe` (the
install window every setup file is built from) in
`%LOCALAPPDATA%\Programs\bach-cli`, and adds that folder to your `PATH`. Open a
new terminal afterwards.

Neither way asks for administrator rights. The
[install docs](https://usebach.app/docs/installation) cover macOS and Linux
scripts, Scoop, a zip file and installing for every user.

Check that it works:

```bash
bach --version
```

## 2. Build your app into a folder

Bach Installer packages an app that is already built. It installs everything in one
folder, so that folder needs your program and every file it uses:

```text
my-app/
  build/
    my-app.exe
    resources/
```

Use the folder your build tool writes to. Don't point Bach Installer at your project
folder, or you'd ship source code and caches.

## 3. Write bach.toml

`bach.toml` is the one file that tells Bach Installer what to package. In your project
folder, run:

```bash
bach init
```

For Rust, Tauri, Wails, Electron, .NET, Flutter, PyInstaller, Qt, Godot and
Unity projects, it fills in the name, version, publisher, program and build
folder from your project files. Check them. These are the required settings:

```toml
[app]
name = "My App"            # shown in the window, the Start menu and Settings › Apps
version = "1.0.0"
publisher = "Your Name"
executable = "my-app.exe"  # the program to start, inside the files folder

[files]
source = "build"           # everything in this folder is installed
```

Then check it:

```bash
bach check
```

`bach check` lists every file and option the installer will have. If something
is wrong, you get one sentence naming the setting.

In a GitHub checkout, `bach init` also turns on automatic updates and adds a
release workflow. Delete the `[updates]` section if you don't want updates yet.

## 4. Preview the window

```bash
bach preview
```

This opens the window your users will see. Nothing is installed, even if you
click **Install**. Preview needs Windows or a Mac.

## 5. Build and test

```bash
bach build
```

The setup file is written to `dist\My App-Setup-1.0.0.exe`. This is the file
you give to people.

Running it really installs your app, for your Windows account only, without an
administrator prompt. You can remove it again from **Settings › Apps**.

Other people who download the file may see **Windows protected your PC** until
you [sign it](signing.md).

## 6. Make it yours

All of this is optional:

```toml
[app]
description = "Your notes, everywhere."  # the line under the heading
icon = "assets/app.ico"    # the setup file's icon, and the logo if it has a PNG inside

[installer]
logo = "assets/logo.svg"   # or a .png, or "none"
accent = "#2563eb"         # buttons, the progress bar, ticked boxes
theme = "system"           # or "light" / "dark"
languages = ["en", "de"]   # leave out to speak all 19, following Windows
license = "LICENSE.txt"    # an accept page before installing
start_with_windows = false # offered, unticked
add_to_path = "always"     # for command-line tools

[text]
footer = "Made with care by {publisher}"

[uninstall]
data = ['%APPDATA%\My App']
data_description = "Your settings and saved projects."
```

`bach styles` lists 14 built-in styles, and `bach preview --style aurora` shows
one on your app. `bach watch` keeps the preview open and reopens it every time
you save. Every setting is in the [config reference](config.md).

## 7. Ship updates

Raise `version`, build again and give people the new file. Running it updates
the app in place: it keeps the folder and the user's choices, and removes files
the new version no longer has. If something goes wrong, the old version is put
back.

Keep `app.name` (or `app.id`, if you set it) the same across versions. Bach Installer
uses it to recognize the installed app.

To have installed apps update themselves from GitHub releases:

```toml
[updates]
github = "owner/repo"
```

See [releasing](releasing.md) for the one-click release and
[updates](updates.md) for other hosts and what your app can do.

## Next

- Run `bach doctor` if a build doesn't work.
- [Sign your setup file](signing.md).
- For scripts and deployment tools, every setup file installs without a window
  with `--silent`; see [silent installs](config.md#silent-installs).
- For CI, `bach build` exits with a non-zero code and a readable message on any
  problem, and `--json` prints a machine-readable result.
