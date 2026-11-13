# How Bach works

## One executable, three parts

`bach-setup.exe` is the installer window. On its own it installs nothing.
`bach build` copies it, gives the copy your icon and version details, and
appends your app to its end:

```
[bach-setup.exe] [archive] [logo] [manifest] [padding] [trailer]
```

- **archive**: every file from `[files] source` in one zstd stream.
- **logo**: the bytes of `[installer] logo`, if any.
- **manifest**: JSON describing the app: name, version, publisher, the files
  and their sizes, the look.
- **trailer**: 40 bytes: `BACHPACK`, the format version, the padding length
  and the length of each section.

Windows ignores bytes after an executable's last section, so the setup file
still runs as `bach-setup.exe`. On start it reads its own trailer, finds the
manifest, and shows the install window. `crates/pack/src/container.rs` has
the exact layout.

## Installing

1. Every file is unpacked through a `.bach-new` file and renamed into place,
   so an interrupted copy never leaves a half-written program.
2. If an earlier version is installed, the files it had and the new one
   doesn't are removed. The earlier file list comes from its uninstaller.
3. `uninstall.exe` is written: `bach-setup.exe` again, with the manifest and
   no archive.
4. Start menu and desktop shortcuts are added, and the app is listed under
   **Settings › Apps** (`HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\<id>`).

Everything is per user, so nothing asks for administrator rights.

## Removing

Settings › Apps runs `uninstall.exe`. A running program can't delete its own
file, so it copies itself to the temp folder, starts the copy and exits. The
copy shows the window and removes:

- the program first (if the app is still open, nothing has changed yet),
- every file the manifest lists, and the folders they leave empty,
- the shortcuts and the Apps entry,
- the data folders, only if the user ticked the box,
- the install folder, only if it's empty, so files a user kept there stay.

Then a hidden helper deletes the temp copy once it has exited.

## Safety rules

- Packaged paths are relative and can never contain `..`, so a setup file
  can't write outside its install folder.
- Data folders must sit inside the user's profile (`%APPDATA%`,
  `%LOCALAPPDATA%`, `%USERPROFILE%`) and name a folder of their own.
- The uninstaller refuses a folder that doesn't hold the app.
- The manifest is checked again when a setup file starts, so a damaged or
  edited file fails with a sentence instead of misbehaving.

## Signing

The payload sits before the signature's certificate table, so `signtool`
signs a setup file like any other program and the signature covers the
payload. See [signing](signing.md).
