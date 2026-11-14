---
name: bach
description: Build a modern Windows installer (Setup.exe) for any desktop app with Bach - one bach.toml, then `bach build`. Use when the user wants to package, ship or distribute a Windows app (Rust, Electron, Tauri, .NET, Go, C++, Python bundles, anything with an .exe), make a setup file or installer, add an uninstaller, auto-updates over an old install, Start menu or desktop shortcuts, PATH, file associations, URL protocols, a license page or a localized install window, or replace an NSIS / Inno Setup / WiX / MSI script.
---

# Bach: Windows installers from one bach.toml

Bach turns a folder with an app into one `Setup.exe`: a calm, native-looking
install window (light and dark, 13 languages, follows Windows' language), a
built-in updater (a newer setup over an install updates it in place) and a
clean uninstaller in Settings › Apps. Installs are per user, so there is no
administrator prompt. There is no script language: everything is declared in
`bach.toml`.

## 1. Get the `bach` command

Check first: `bach --version`. If that works, skip to step 2.

No Rust is needed. Pick the first that applies:

- **Windows (PowerShell):**
  `irm https://raw.githubusercontent.com/usebach/bach/main/install.ps1 | iex`
  installs `bach.exe` and `bach-setup.exe` to `%LOCALAPPDATA%\Programs\bach-cli`
  and puts it on `PATH`. In the same terminal, call it by full path or open a
  new terminal.
- **macOS or Linux (CI too):**
  `curl -fsSL https://raw.githubusercontent.com/usebach/bach/main/install.sh | sh`
  installs `bach` to `~/.local/bin`. It downloads the Windows runtime it
  builds on (`bach-setup.exe`, SHA-256 checked) on the first `bach build`.
- **No network to GitHub's raw files:** download
  `bach-<version>-windows-x64.zip` from https://github.com/usebach/bach/releases,
  unzip, and keep `bach.exe` and `bach-setup.exe` in the same folder.
- **Building from source** (only if Rust is already installed, or the user
  wants it): `cargo install --git https://github.com/usebach/bach bach-cli`,
  then `bach` downloads its runtime when it needs it. In a checkout of the
  Bach repository, `cargo build --release` gives both binaries in
  `target/release`.

Then run `bach doctor` in the project folder. It says whether the runtime
and the config are in place and exactly what to fix if not.

## 2. Build the app into one folder

Bach installs the contents of one folder: the `.exe` plus everything it needs
next to it (DLLs, resources). Find or produce that folder first, for example:

| Stack | Typical folder |
| --- | --- |
| Rust | `target/release` (exclude `*.pdb`, `*.d`, `build/**`, `deps/**`, `incremental/**`, `.fingerprint/**`), or better a folder you copy the `.exe` and assets into |
| Electron (electron-builder / forge) | `dist/win-unpacked` or `out/<name>-win32-x64` |
| Tauri | copy `src-tauri/target/release/<app>.exe` (and any resources) into a folder |
| .NET | `dotnet publish -c Release -r win-x64 --self-contained` output (`bin/Release/<tfm>/win-x64/publish`) |
| Go | a folder with the built `.exe` |
| PyInstaller | `dist/<name>` (one-folder mode) |

## 3. Write bach.toml

Run `bach init` (it won't overwrite an existing file). In a Rust, Tauri,
Electron or .NET project it fills in name, version, publisher, the program,
the build folder, leftovers to exclude and needed runtimes (WebView2, .NET)
from the project's own files; check them, then edit. Minimal by hand:

```toml
[app]
name = "My App"
version = "1.0.0"
publisher = "Your Name"
executable = "my-app.exe"        # relative to [files] source

[files]
source = "build"                 # relative to bach.toml
exclude = ["*.pdb"]
```

Take name, version and publisher from the project's own metadata
(`Cargo.toml`, `package.json`, `.csproj`) instead of inventing them, and keep
`version` in sync with it. Never change `[app] id` (defaults to `name`) after
a release: it is how updates recognise the install.

Common additions (all optional; full reference in `docs/config.md` of the
repository, https://github.com/usebach/bach/blob/main/docs/config.md):

```toml
[app]
description = "One line about the app."   # subtitle on the first page
icon = "assets/app.ico"                   # setup file icon; also the logo if it holds a PNG

[installer]
accent = "#2563eb"
logo = "assets/logo.png"                  # .svg (one colour), .png, or "none"
languages = ["en", "de"]                  # default: all 13; one entry = always that language
license = "LICENSE.txt"                   # accept page before installing
# options: true (offered, ticked) | false (offered, unticked) | "always" | "never"
desktop_shortcut = true
start_menu_shortcut = "always"
start_with_windows = "never"
add_to_path = "never"                     # "always"/true for CLI tools
path_folder = "bin"
change_folder = true
open_button = true

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
kind = "webview2"                         # also "dotnet" (version, desktop), "vcredist", "file", "registry"

[[shortcuts]]                             # more Start menu shortcuts
name = "My App Docs"
target = "docs.exe"

[updates]                                 # the installed app can update itself
github = "owner/repo"                     # from that repo's GitHub releases; or:
# feed = "https://example.com/myapp/latest.json"
# download_url = "https://example.com/myapp/{file}"
# auto = true                             # offer "Automatic updates" (false / "always" / "never")

[uninstall]
data = ['%APPDATA%\My App']               # single quotes keep backslashes
data_description = "Your settings."
feedback_url = "https://example.com/why"  # "Tell us why" after removing
```

The file starts with `#:schema https://raw.githubusercontent.com/usebach/bach/main/schema/bach.schema.json`,
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
bach preview --uninstall
bach build                 # writes dist/<name>-Setup-<version>.exe
bach inspect dist/My\ App-Setup-1.0.0.exe
```

Treat every `warning:` line from `bach check` as something to fix or to
mention to the user (for example a `[text]` given in one language while the
window speaks several).

## 5. Verify, safely

- Never run the built setup (a real install) on the user's machine unless
  they ask. `bach preview` shows every page without changing anything.
- When a real end-to-end test is wanted, use a silent install and remove it
  again afterwards:
  `Setup.exe --silent` then
  `"%LOCALAPPDATA%\Programs\<folder>\uninstall.exe" --silent`.
  Exit code 0 means done; 1 failed (reason printed); 2 a flag mistake.
- Silent flags: `--dir PATH`, `--with a,b`, `--without a,b` (ids:
  `desktop-shortcut`, `start-menu`, `autostart`, `path`, component ids),
  `--lang de`, `--allow-downgrade`, `--log FILE`, and `--delete-data` for the
  uninstaller.
- Every run logs to `%TEMP%\<app id>-setup.log`; read it when something
  fails instead of guessing.
- In the Bach repository itself, `scripts\e2e.ps1` installs, updates and
  removes a throwaway app for real; it's the safe way to test changes to
  setup.

## 6. Ship

- Sign the setup file with `signtool` like any program (the payload is
  covered by the signature); unsigned downloads get SmartScreen warnings.
- For an update: raise `version`, `bach build`, ship the new file. Running it
  over an install updates in place, keeps the user's choices and data, and
  removes files the new version no longer has.
- In CI, `bach build` exits non-zero with a one-sentence reason on any
  problem. On Linux/macOS runners it works too (the setup file keeps Bach's
  icon there; build on Windows to apply `app.icon`). The GitHub Action does
  both steps: `uses: usebach/bach@v0` with `config: bach.toml`, output
  `setup`.
- To publish updates from GitHub: set `[updates] github = "owner/repo"`, and
  in a workflow that runs on pushed tags (`v*`, with `contents: write`) use
  `usebach/bach@v0` with `config: bach.toml` and `release: true`. The tag is
  the app's version (`bach build --app-version v1.4.0`), the release holds the
  setup file and `latest.json`. `@v0.2` pins a line, a SHA pins a commit.
  `sign-command`, `notes` and `version` (which Bach: `0.2`, `latest`) are the
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
- To list the app in winget: `bach winget SETUP.exe --url URL --license MIT`,
  then `winget validate` and a pull request to microsoft/winget-pkgs.

## Troubleshooting

| Message | Fix |
| --- | --- |
| `couldn't get bach-setup.exe …` | No network or no release for this version: put `bach-setup.exe` next to `bach`, or pass `--runtime PATH` / set `BACH_RUNTIME`. |
| `app.executable is "…", but there's no such file` | Build the app first, or fix `source`/`executable` (paths are relative to `bach.toml` and `source`). |
| `… is a symbolic link` | Copy real files into the source folder. |
| `couldn't replace … (is it open?)` | Close the old setup file or the running app, then build again. |
| `bach preview … needs Windows` | Use `bach check` on macOS/Linux. |
| `requirements[0].…` | Give `kind` one of the listed kinds; `file` and `registry` need `name` and an https `url`. |
| The installer says a newer version is installed | Raise `version` above the installed one, or pass `--allow-downgrade` on a silent run. |
| `install.ps1` refuses a folder | It never writes into a folder it didn't make; set `BACH_INSTALL_DIR` to an empty one. |
