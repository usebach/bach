# Signing a setup file

Windows SmartScreen warns about unsigned programs from publishers it doesn't
know yet. A code-signing certificate makes the warning go away over time and
shows your name as the publisher when people download and run your setup file.

Bach setup files sign like any other program. Build first, then sign the
finished file:

```bash
bach build
```

```bash
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a "dist\My App-Setup-1.0.0.exe"
```

The signature is appended after Bach's payload and covers it, so any change
to the file after signing breaks the signature. Check it with:

```bash
signtool verify /pa "dist\My App-Setup-1.0.0.exe"
```

Notes:

- Sign after `bach build`, never before: building writes into the file.
- The `uninstall.exe` an install leaves behind is unsigned. It's written on
  the user's computer from the signed setup file, without the signature,
  because a signature only covers the exact file it was made for.
- `bach inspect` reads signed setup files too.
