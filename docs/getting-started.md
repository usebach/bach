# Getting started with Bach

This guide takes you from an app on your disk to a `Setup.exe` you can
share. It takes about five minutes.

## 1. Get Bach

Download `bach-<version>-windows-x64.zip` from the
[releases page](https://github.com/usebach/bach/releases) and unzip it
somewhere permanent, for example `C:\Tools\Bach`. You get two files:

- `bach.exe`, the command you run.
- `bach-setup.exe`, the installer window every setup file is built from.

Keep them in the same folder. Optionally add that folder to your `PATH`, so
`bach` works from any terminal.

## 2. Build your app into a folder

Bach installs the contents of one folder. Build or copy your app into it,
with the `.exe` and everything it needs next to it (DLLs, assets, data
files). For example:

```
my-app/
  bach.toml
  build/
    my-app.exe
    resources/
```

## 3. Write bach.toml

In your project folder, run:

```bash
bach init
```

Open the `bach.toml` it wrote. Only two sections are required:

```toml
[app]
name = "My App"            # shown in the window, the Start menu and Settings › Apps
version = "1.0.0"
publisher = "Your Name"
executable = "my-app.exe"  # the program to start, inside the files folder

[files]
source = "build"           # everything in this folder is installed
```

Check it:

```bash
bach check
```

`bach check` lists every file the installer will install, or tells you in a
sentence what's wrong, naming the setting at fault.

## 4. Make it yours (optional)

```toml
[app]
icon = "assets/app.ico"    # the setup file's icon in Explorer and the taskbar

[installer]
logo = "assets/logo.svg"   # shown at the top of the window
accent = "#2563eb"         # buttons, the progress bar, ticked boxes
theme = "system"           # or "light" / "dark"

[uninstall]
data = ['%APPDATA%\My App']
data_description = "Your settings and saved projects."
```

See it without installing anything:

```bash
bach preview
```

```bash
bach preview --uninstall --theme dark
```

## 5. Build

```bash
bach build
```

The setup file lands in `dist\My App-Setup-1.0.0.exe` (change it with
`[installer] output`). Run it to try the real thing; remove it again from
**Settings › Apps**.

## 6. Ship updates

Raise `version` in `bach.toml`, rebuild, and hand out the new setup file.
Running it over an install updates it in place: it keeps the folder and the
desktop shortcut choice, and removes files the new version no longer ships.
Keep `[app] id` (it defaults to the name) the same across versions; it's
how Bach recognises the install.

## Tips

- **SmartScreen.** Windows warns about unsigned downloads from new
  publishers. Sign your setup file to build trust: see [signing](signing.md).
- **Close the app first.** If the app is running, setup says so and offers
  "Try again" instead of replacing files in use.
- **Leave out build leftovers** with `[files] exclude = ["*.pdb", "*.log"]`.
- **CI.** `bach build` exits with a non-zero code and a readable message on
  any problem, so it fits in a pipeline as is.
