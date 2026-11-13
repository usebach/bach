# How Bach works

## One executable, three parts

`bach-setup.exe` is the installer window. On its own it installs nothing.
`bach build` copies it, gives the copy your icon and version details, and
appends your app to its end:

```
[bach-setup.exe] [archive] [logo] [manifest] [padding] [trailer]
```

- **archive**: every file from `[files] source` in one zstd stream.
- **logo**: the bytes of `[installer] logo` (or the PNG inside `app.icon`), if any.
- **manifest**: JSON describing the app: name, version, publisher, the files
  and their sizes, the options, components, texts and license, the look.
- **trailer**: 40 bytes: `BACHPACK`, the format version, the padding length
  and the length of each section.

Windows ignores bytes after an executable's last section, so the setup file
still runs as `bach-setup.exe`. On start it reads its own trailer, finds the
manifest, and shows the install window. `crates/pack/src/container.rs` has
the exact layout. Fields added to the manifest over time all have defaults,
so a newer setup file still reads the uninstaller an older one installed.

## Installing

1. Checks before anything changes: no other setup for the same app is
   running, the app isn't open, and the drive has room for every file.
2. Every chosen file is unpacked next to its target as `<name>.bach-new`.
   Only when all of them are on disk are they swapped into place, each old
   file stepping aside as `<name>.bach-old` first. If one can't be moved (a
   file in use), every move is undone: an update either completes or leaves
   the earlier version exactly as it was.
3. If an earlier version is installed, the files it installed and this one
   doesn't (including components the user unticked) are removed. The earlier
   file list comes from its uninstaller.
4. `uninstall.exe` is written: `bach-setup.exe` again, with the manifest of
   what was installed and the user's choices, and no archive.
5. Shortcuts, start with Windows, the `PATH` entry, link schemes and file
   types are added (or taken away, when an update unticked them), and the
   app is listed under **Settings › Apps**
   (`HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\<id>`).

Everything is per user, so nothing asks for administrator rights. The
window can't be closed while files are moving.

## Languages

The window's texts live in `crates/pack/src/i18n`, one table per language.
At start the window reads the user's Windows display languages, most
preferred first, and picks the first one the app offers
(`installer.languages`, or every built-in language), else the app's first
language. The developer's `[text]` replaces any built-in text, per language.

## Removing

Settings › Apps runs `uninstall.exe`. A running program can't delete its own
file, so it copies itself to the temp folder, starts the copy and exits. The
copy shows the window and removes:

- the program first (if the app is still open, nothing has changed yet),
- every file the manifest lists, and the folders they leave empty,
- the shortcuts and the Apps entry,
- start with Windows, the `PATH` entry, link schemes and file types, each
  only if it's this install's own,
- the data folders, only if the user ticked the box,
- the install folder, only if it's empty, so files a user kept there stay.

Then a hidden helper deletes the temp copy once it has exited. A silent
removal (`uninstall.exe --silent`) works in place instead, so whoever
started it can wait for its exit code.

## Safety rules

- Packaged paths are relative and can never contain `..`, so a setup file
  can't write outside its install folder.
- Data folders must sit inside the user's profile (`%APPDATA%`,
  `%LOCALAPPDATA%`, `%USERPROFILE%`) and name a folder of their own.
- The uninstaller refuses a folder that doesn't hold the app, and an update
  only cleans up after an earlier install of the same app id.
- The manifest is checked again when a setup file starts, so a damaged or
  edited file fails with a sentence instead of misbehaving.
- `PATH` is edited as raw text: exactly one entry is added or removed, the
  value's type (`REG_EXPAND_SZ`) is kept, and nothing is written when
  there's nothing to change.
- Registry keys Bach creates for link schemes and file types carry a
  `BachApp` mark; keys without it belong to another program and are never
  changed or deleted.
- The C runtime is linked in, so setup files start on a fresh Windows
  without the Visual C++ redistributable.

## Signing

The payload sits before the signature's certificate table, so `signtool`
signs a setup file like any other program and the signature covers the
payload. See [signing](signing.md).
