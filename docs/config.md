# bach.toml reference

Every setting Bach reads. Paths are relative to the `bach.toml` file. Unknown
settings are an error, so a typo never goes unnoticed.

## [app]

| Setting | Required | Meaning |
| --- | --- | --- |
| `name` | yes | The app's name, as people see it: the window, the Start menu, the desktop shortcut and Settings › Apps. |
| `version` | yes | Any version text, like `1.2.0` or `2.0.0-beta.1`. Shown in the window and Settings › Apps; the first numbers also go into the setup file's details. |
| `publisher` | yes | You or your company. Shown in the window's footer and Settings › Apps. |
| `executable` | yes | The program to start, relative to `[files] source`, for example `my-app.exe` or `bin/my-app.exe`. Shortcuts and the "Open" button start it. |
| `id` | no | Identifies the install across versions (registry key name). Defaults to `name`. Letters, digits, spaces, `.`, `-` and `_`. Never change it after your first release, or updates install side by side. |
| `description` | no | One line, used as the shortcut's tooltip. |
| `website` | no | An `https://` link, shown as the support link in Settings › Apps. |
| `icon` | no | A `.ico` file. It becomes the setup file's icon and the window's icon. Without it the setup file shows the Bach icon. Include sizes 16 to 256 for crisp results. |

## [files]

| Setting | Required | Meaning |
| --- | --- | --- |
| `source` | yes | The folder whose contents are installed, subfolders included. |
| `exclude` | no | Patterns to leave out. `*` matches within a name, `**` across folders, `?` one character. A pattern without `/` matches file names anywhere (`*.pdb`); one with `/` matches from the top of `source` (`docs/**`). Case doesn't matter. |

The source folder may not contain symbolic links or a file named
`uninstall.exe` at its top (Bach adds its own).

## [installer]

| Setting | Default | Meaning |
| --- | --- | --- |
| `output` | `dist/{name}-Setup-{version}.exe` | Where the setup file goes. `{name}`, `{version}` and `{id}` are filled in. |
| `folder` | the app id | The install folder's name under `%LOCALAPPDATA%\Programs`. When the user picks another place, this name is added to it. |
| `desktop_shortcut` | `true` | Whether the desktop shortcut box starts ticked on a first install. Updates keep the user's earlier choice. |
| `theme` | `system` | `system` follows Windows' light or dark setting, live. `light` or `dark` fixes it. |
| `accent` | none (black / white) | A colour like `#2563eb` for buttons, the progress bar and ticked boxes. The button text picks black or white for contrast. |
| `logo` | the Bach mark | An `.svg` or `.png` shown at the top of the window. An SVG is drawn in one colour that follows the theme (like the Bach mark); use a PNG for a full-colour logo. 56 × 56 points; give a PNG at least 112 px for sharp results. |
| `compression` | `best` | `best` makes the smallest file, `normal` and `fast` build quicker. Unpacking is fast either way. `bach build --fast` overrides it once. |

## [uninstall]

| Setting | Default | Meaning |
| --- | --- | --- |
| `data` | none | Folders the app keeps its data in. The uninstaller offers "Also delete my data" (unticked) and deletes them only when the user ticks it. Each must start with `%APPDATA%`, `%LOCALAPPDATA%` or `%USERPROFILE%`, followed by your app's own folder, so the uninstaller can only ever delete inside the user's profile. |
| `data_description` | a generic line | What lives in those folders, in a few words, shown under the checkbox. |

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
desktop_shortcut = true
compression = "best"

[uninstall]
data = ['%APPDATA%\Hello', '%LOCALAPPDATA%\Hello\Cache']
data_description = "Your settings and the download cache."
```
