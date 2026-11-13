# bach.toml reference

Every setting Bach reads. Paths are relative to the `bach.toml` file. Unknown
settings are an error, so a typo never goes unnoticed, and every mistake is
reported as one sentence naming the setting at fault.

Only `[app]` and `[files]` are required. Everything else has a sensible
default, so a five-line config already builds a complete installer.

- [`[app]`](#app): name, version, publisher, the program
- [`[files]`](#files): what gets installed
- [`[installer]`](#installer): look, languages, license and options
- [`[[components]]`](#components): optional parts the user can leave out
- [`[[protocols]]`](#protocols): links like `myapp://` that open your app
- [`[[file_types]]`](#file_types): files your app opens
- [`[text]`](#text): any text in the window, in any language
- [`[uninstall]`](#uninstall): the user's data
- [A complete example](#a-complete-example)

## [app]

| Setting | Required | Meaning |
| --- | --- | --- |
| `name` | yes | The app's name, as people see it: the window, the Start menu, the desktop shortcut and Settings › Apps. |
| `version` | yes | Any version text, like `1.2.0` or `2.0.0-beta.1`. Shown in the window and Settings › Apps; the first numbers also go into the setup file's details. |
| `publisher` | yes | You or your company. Shown in the window's footer and Settings › Apps. |
| `executable` | yes | The program to start, relative to `[files] source`, for example `my-app.exe` or `bin/my-app.exe`. Shortcuts and the "Open" button start it. |
| `id` | no | Identifies the install across versions (registry key name). Defaults to `name`. Letters, digits, spaces, `.`, `-` and `_`. Never change it after your first release, or updates install side by side. |
| `description` | no | One line. The shortcut's tooltip, and the line under "Install My App" on the first page (unless `[text] install_subtitle` replaces it). Without it, that line is left out. |
| `website` | no | An `https://` link, shown as the support link in Settings › Apps. |
| `icon` | no | A `.ico` file. It becomes the setup file's icon and the window's icon, and, when it holds a PNG image (most modern icons do) and `installer.logo` isn't set, the logo in the window. Include sizes 16 to 256 for crisp results. |

## [files]

| Setting | Required | Meaning |
| --- | --- | --- |
| `source` | yes | The folder whose contents are installed, subfolders included. |
| `exclude` | no | Patterns to leave out. `*` matches within a name, `**` across folders, `?` one character. A pattern without `/` matches file names anywhere (`*.pdb`); one with `/` matches from the top of `source` (`docs/**`). Case doesn't matter. |

The source folder may not contain symbolic links or a file named
`uninstall.exe` at its top (Bach adds its own).

## [installer]

### Look

| Setting | Default | Meaning |
| --- | --- | --- |
| `output` | `dist/{name}-Setup-{version}.exe` | Where the setup file goes. `{name}`, `{version}` and `{id}` are filled in. |
| `theme` | `system` | `system` follows Windows' light or dark setting, live. `light` or `dark` fixes it. |
| `accent` | none (black / white) | A colour like `#2563eb` for buttons, the progress bar and ticked boxes. The button text picks black or white for contrast. |
| `logo` | your icon, else the Bach mark | An `.svg` or `.png` shown at the top of the window, or `"none"` for no logo at all. An SVG is drawn in one colour that follows the theme; use a PNG for a full-colour logo. 56 × 56 points; give a PNG at least 112 px for sharp results. |
| `compression` | `best` | `best` makes the smallest file, `normal` and `fast` build quicker. Unpacking is fast either way. `bach build --fast` overrides it once. |

### Where it installs

| Setting | Default | Meaning |
| --- | --- | --- |
| `folder` | the app id | The install folder's name under `%LOCALAPPDATA%\Programs`. When the user picks another place, this name is added to it. |
| `change_folder` | `true` | Whether the user may pick another folder ("Change", or a click on the Location row). Updates always stay where they are. |
| `open_button` | `true` | Whether the last page offers "Open My App". |

### Options

Each option takes one of four values:

| Value | The install page | What happens |
| --- | --- | --- |
| `true` | shows a ticked box | done, unless the user unticks it |
| `false` | shows an unticked box | done only if the user ticks it |
| `"always"` | shows nothing | always done |
| `"never"` | shows nothing | never done |

| Setting | Default | What it does |
| --- | --- | --- |
| `desktop_shortcut` | `true` | A shortcut on the desktop. |
| `start_menu_shortcut` | `"always"` | A shortcut in the Start menu. |
| `start_with_windows` | `"never"` | Starts the app when the user signs in (`HKCU\…\Run`). |
| `add_to_path` | `"never"` | Adds the app's folder to the user's `PATH`, so command-line tools run from any terminal. Only that one entry is ever added or removed; the rest of `PATH` is kept byte for byte. |
| `path_folder` | the install folder | The folder, inside the install folder, that goes on `PATH`, like `"bin"`. |

The whole row is a click target: clicking anywhere on it ticks or unticks
its box. On an update, the boxes start the way the user left them last time,
and unticking one takes the shortcut, PATH entry or autostart away again.

### Languages and license

| Setting | Default | Meaning |
| --- | --- | --- |
| `languages` | every built-in language | The languages the window may speak, like `["en", "de"]`. The window follows the user's Windows display language when it's in this list, and otherwise uses the first one. With a single language, like `["de"]`, the window always speaks it. |
| `license` | none | A UTF-8 text file the user accepts before the install page, like `"LICENSE.txt"`, or one per language: `{ en = "LICENSE.txt", de = "LIZENZ.txt" }`. An update shows it again only when it changed. |

Built-in languages: `en` English, `de` Deutsch, `nl` Nederlands, `fr`
Français, `es` Español, `it` Italiano, `pt` Português, `pl` Polski, `ru`
Русский, `tr` Türkçe, `ja` 日本語, `ko` 한국어, `zh` 简体中文.

## [[components]]

Optional parts of your app, each shown as a box on the install page. Files
that no component claims are always installed.

```toml
[[components]]
id = "samples"                      # lowercase; also its name for --with
title = "Sample projects"           # or { en = "…", de = "…" }
description = "Examples to learn from."
files = ["samples/**"]              # patterns, like [files] exclude
default = true                      # ticked on a first install
```

A file belongs to the first component whose pattern matches it. A component
must match at least one file, and `app.executable` can't be in one. Unticking
a component on an update removes its files.

## [[protocols]]

Links like `myapp://open?file=x` start your app with the link as its first
argument. Registered for the user only.

```toml
[[protocols]]
scheme = "myapp"
description = "My App link"         # optional
```

Schemes that belong to Windows or browsers (`http`, `file`, `ms-…`, …) are
refused. If another program already owns the scheme, setup leaves it alone.

## [[file_types]]

Files your app opens. They're listed under "Open with", and your app becomes
the default only for types no other program opens yet, so installing your
app never takes files away from another one.

```toml
[[file_types]]
extension = ".mydoc"
description = "My App document"     # optional
```

Types that run code (`.exe`, `.bat`, `.ps1`, `.lnk`, …) are refused.

## [text]

Replaces any text the window shows. Give one text for every language, or a
text per language:

```toml
[text]
install_title = { en = "Get {name}", de = "Hol dir {name}" }
install_subtitle = "The fastest way to write things down."
footer = "Made with care by {publisher}"
```

Every text can use `{name}`, `{version}` and `{publisher}`. A few have their
own placeholders, listed below. When the window speaks several languages and
a text is given in only one, `bach check` points it out.

| Text | English default |
| --- | --- |
| `window_install` | {name} Setup |
| `window_remove` | Remove {name} |
| `install_title` | Install {name} |
| `install_subtitle` | (the app's description; `{description}` works here) |
| `install_button` | Install |
| `reinstall_title` | Reinstall {name} |
| `reinstall_subtitle` | {name} {version} is already installed. Reinstalling keeps your data. |
| `reinstall_button` | Reinstall |
| `update_title` | Update {name} |
| `update_subtitle` | {name} {old_version} is installed. This updates it to {version} and keeps your data. |
| `update_button` | Update |
| `remove_title` | Remove {name} |
| `remove_subtitle` | This removes {name} and its shortcuts from this computer. |
| `remove_button` | Remove |
| `license_title` | License agreement |
| `license_subtitle` | Please read the terms before you install {name}. |
| `license_accept` | I agree |
| `cancel`, `close`, `retry`, `change` | Cancel, Close, Try again, Change |
| `open`, `open_short` | Open {name}, Open |
| `footer` | {publisher} · Version {version} |
| `preview` | Preview |
| `location` | Location |
| `choose_folder` | Choose where to install {name} |
| `desktop_shortcut`, `desktop_shortcut_description` | Desktop shortcut, Start {name} from your desktop too. |
| `start_menu`, `start_menu_description` | Start menu, Find {name} in the Start menu. |
| `start_with_windows`, `start_with_windows_description` | Start with Windows, Open {name} automatically when you sign in. |
| `add_to_path`, `add_to_path_description` | Add to PATH, Run {name} from any terminal. |
| `delete_data`, `delete_data_description` | Also delete my data, Settings and files {name} saved on this computer. |
| `installing`, `updating`, `removing` | Installing {name}, Updating {name}, Removing {name} |
| `step_preparing`, `step_copying`, `step_tidying`, `step_uninstaller`, `step_shortcuts`, `step_registering` | The progress labels while installing. |
| `step_removing`, `step_removing_shortcuts`, `step_unregistering`, `step_deleting_data`, `step_cleaning`, `step_done` | The progress labels while removing. |
| `installed_title`, `installed_subtitle`, `installed_subtitle_plain`, `updated_title` | {name} is installed, Find it in the Start menu, or open it now., {name} is ready to use., {name} is up to date |
| `removed_title`, `removed_subtitle`, `removed_data_deleted`, `removed_data_kept` | The last page after removing. |
| `install_failed`, `update_failed`, `remove_failed` | {name} couldn't be installed (updated, removed) |
| `error_still_open`, `error_in_use` (`{path}`), `error_busy`, `error_disk_full` (`{size}`, `{free}`), `error_no_access` (`{path}`) | The errors people meet most. |
| `error_create`, `error_write`, `error_delete`, `error_shortcut`, `error_delete_data`, `error_not_found` (all `{path}`) | Errors about one file or folder; Windows' own reason follows them. |
| `error_read_setup`, `error_damaged`, `error_uninstaller`, `error_register`, `error_unregister`, `error_system`, `error_open_failed` (`{error}`) | The rarer errors. |
| `problem_title`, `problem_damaged` (`{error}`) | The page a damaged setup file shows. |

`crates/pack/src/i18n/en.rs` has every English text in full, and the other
files there every translation.

## [uninstall]

| Setting | Default | Meaning |
| --- | --- | --- |
| `data` | none | Folders the app keeps its data in. The uninstaller offers "Also delete my data" (unticked) and deletes them only when the user ticks it. Each must start with `%APPDATA%`, `%LOCALAPPDATA%` or `%USERPROFILE%`, followed by your app's own folder, so the uninstaller can only ever delete inside the user's profile. |
| `data_description` | a generic line | What lives in those folders, in a few words, shown under the checkbox. Text or `{ en = "…", de = "…" }`. |

Write Windows paths in TOML with single quotes so backslashes stay as they
are: `data = ['%APPDATA%\My App']`.

## A complete example

```toml
[app]
name = "Hello"
version = "1.4.0"
publisher = "Hello Inc."
executable = "hello.exe"
description = "Says hello, nicely."
website = "https://hello.example"
icon = "assets/hello.ico"

[files]
source = "target/release/package"
exclude = ["*.pdb"]

[installer]
output = "dist/Hello-Setup-{version}.exe"
theme = "system"
accent = "#2563eb"
logo = "assets/logo.svg"
languages = ["en", "de"]
license = { en = "LICENSE.txt", de = "LIZENZ.txt" }
desktop_shortcut = true
start_menu_shortcut = "always"
start_with_windows = false
add_to_path = false
path_folder = "bin"

[[components]]
id = "samples"
title = { en = "Sample projects", de = "Beispielprojekte" }
files = ["samples/**"]
default = true

[[protocols]]
scheme = "hello"

[[file_types]]
extension = ".hello"
description = "Hello note"

[text]
install_subtitle = { en = "Say hello in style.", de = "Sag Hallo mit Stil." }

[uninstall]
data = ['%APPDATA%\Hello', '%LOCALAPPDATA%\Hello\Cache']
data_description = { en = "Your settings and the download cache.", de = "Deine Einstellungen und der Download-Cache." }
```

## Silent installs

Every setup file also installs without a window, for IT tools and scripts:

```bash
Hello-Setup-1.4.0.exe --silent
```

| Flag | Meaning |
| --- | --- |
| `--silent` (also `/S`, `/quiet`) | No window. Exit code 0 when done, 1 when it failed (the reason is printed in a terminal), 2 for a command-line mistake. |
| `--dir PATH` | Install here instead of the default folder (a first install, when `change_folder` allows it). |
| `--with a,b` / `--without a,b` | Tick or untick options: `desktop-shortcut`, `start-menu`, `autostart`, `path`, and component ids. Only options the install page would offer. |
| `--lang CODE` | The language for messages, instead of Windows' own. |

A silent run accepts the license on the user's behalf. Settings › Apps runs
`uninstall.exe --silent` for a quiet removal; add `--delete-data` to delete
the user's data too.
