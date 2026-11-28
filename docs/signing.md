# Signing a setup file

Windows SmartScreen warns about unsigned programs from publishers it doesn't
know yet. A code-signing certificate makes the warning go away over time and
shows your name as the publisher when people download and run your setup file.

Bach Installer setup files sign like any other program. Build first, then sign the
finished file:

```powershell
bach build
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a "dist\My App-Setup-1.0.0.exe"
```

`signtool` comes with the Windows SDK (install its "Windows SDK Signing Tools
for Desktop Apps"), and it is usually **not on your `PATH`**: PowerShell says
the name isn't recognised. Use its full path, which looks like this, or open
"Developer PowerShell for VS" from the Start menu, which knows it:

```powershell
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe" sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a "dist\My App-Setup-1.0.0.exe"
```

(Your SDK folder has its own version number. `Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin"` lists them.) `/a` picks the best code-signing
certificate in your certificate store, so you need one installed (from a
certificate authority, or on a hardware token); without it signtool says that
no certificates were found. Bach Installer itself finds `signtool` for you when it signs
(see below).

The signature is appended after Bach Installer's payload and covers it, so any change
to the file after signing breaks the signature. Check it with:

```bash
signtool verify /pa "dist\My App-Setup-1.0.0.exe"
```

## Sign during the build (optional)

Signing is never required: without a `[signing]` section `bach build` writes
unsigned setup files, as before. If you'd rather not sign by hand, tell Bach Installer
which program signs and with what. `bach build --sign` (or `BACH_SIGN=1`, which
suits CI) then signs every setup file it writes, the offline and the online
installer, before it writes the update feed, so the feed already carries the
signed file's SHA-256:

```toml
[signing]
program = "signtool"      # the default; any program that signs a file
arguments = 'sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a'

# After `signtool` ran, Bach Installer asks Windows (Get-AuthenticodeSignature) what the
# file now carries. No signature or a broken one fails the build. A valid
# signature without a timestamp, or one Windows doesn't trust yet (a self-signed
# certificate), only gives a warning. Other signing programs aren't checked.
```

For `signtool` the arguments start with the word `sign`; Bach Installer refuses them
otherwise, because signtool answers "Invalid command". When `signtool` isn't on
`PATH`, Bach Installer uses the newest one in the Windows SDK's folder and says which. Bach Installer
runs `program` with `arguments` and the setup file added at the end
(write `{file}` where it belongs when it isn't last), without a shell, and
prints the command first. A signer that fails fails the build with its exit
code, so an unsigned file never goes out by accident.

Signing only ever runs when you ask for it with `--sign` or `BACH_SIGN=1`.
That is on purpose: a `bach.toml` that came from somewhere else can name any
program, and building it must not run that program behind your back. Keep secrets out of `bach.toml` by writing
`{env:NAME}`: Bach Installer reads the variable when it builds and fails when it isn't
set. The command Bach Installer prints shows `{env:NAME}` as written, never the value.

```toml
[signing]
# Azure Artifact Signing and other cloud signers work the same way.
program = "AzureSignTool"
arguments = 'sign --azure-key-vault-url {env:KV_URL} --azure-key-vault-certificate {env:KV_CERT} -tr http://timestamp.digicert.com'
```

```bash
bach build --sign                       # build and sign
bach sign dist/My-App-Setup-1.0.0.exe   # sign a file you built earlier
```

Without `--sign`, a build with `[signing]` says so and leaves the files
unsigned. Signing needs the signer on the machine that builds; `bach build` on macOS or
Linux can use a signer that runs there (for example `osslsigncode`). `bach
doctor` says whether `[signing]` is set.

Notes:

- Sign after `bach build`, never before: building writes into the file.
- The `uninstall.exe` an install leaves behind is unsigned. It's written on
  the user's computer from the signed setup file, without the signature,
  because a signature only covers the exact file it was made for.
- Signing the setup file does not sign the programs inside it. Windows 11's
  Smart App Control looks at every program that starts, so sign your own
  `.exe` and `.dll` files in your build, before Bach Installer packs them. With
  `[signing]` in `bach.toml`, `bach check` and `bach build` warn when packed
  programs or libraries have no signature (it names the first three). It only
  reads the file's signature entry; Windows still decides whether it is valid.
  Without `[signing]` there is no warning, since you may not sign at all.
- `bach inspect` reads signed setup files too.
- Signing with `[signing]` and signing by hand give the same result; use
  whichever fits your pipeline. After changing a signed file in any way,
  write the feed again with `bach feed`.

## Sign the macOS Setup app

`[macos.signing]` and `bach build --sign` sign the Setup app with `codesign` (a Mac is needed): the identity
is a Developer ID Application certificate in your keychain, the signature gets the hardened runtime and a
secure timestamp, and the `.dmg` is signed too. With `notarize = "profile"`, the build then submits the
`.dmg` with `xcrun notarytool submit --keychain-profile profile --wait` and staples the ticket, so the app opens
without Gatekeeper's warning. Store the profile once with `xcrun notarytool store-credentials`. Without `--sign`,
a Mac seals the Setup app ad hoc: it runs, but a downloaded copy asks the user to confirm the first open.
The payload is inside the bundle (`Contents/Resources/setup.bach`), so the signature covers it.

With [`[updates] signer`](updates.md#signed-updates-on-a-mac), installed copies take an update only when its
Setup app is signed with a Developer ID Application certificate that Apple issued and that carries that name.
A self-signed certificate doesn't count, even with the same name. `bach build` checks it, so you find out before
your users do. With `--sign`, a certificate they would refuse stops the build:

```text
error: The Setup app is signed by "CN=Other Company, OU=ABCDE12345", so installed Macs would refuse it as an update from Example Inc. ([updates] signer); set [macos.signing] identity to that publisher's Developer ID Application certificate and build with --sign, or change [updates] signer.
```

Without `--sign` the build succeeds and warns:

```text
warning: [updates] signer is Example Inc., but this Setup app has no Developer ID signature, so installed Macs will refuse it as an update; before you publish it, set [macos.signing] identity to that publisher's Developer ID Application certificate and build with --sign, or change [updates] signer.
```

The name has to be on both certificates when you ship for Windows and macOS
([Signed updates on a Mac](updates.md#signed-updates-on-a-mac)).

## Check a Bach Installer download

Every file of a Bach Installer release (the zip, `bach-setup.exe`, the example installer,
the macOS and Linux downloads and the Windows ARM64 zip) has a signed
statement from GitHub's artifact attestations: which commit and which workflow
run built it. With the GitHub CLI:

```bash
gh attestation verify bach-0.7.0-windows-x64.zip --repo usebach/bach-installer
```

The release also lists what `bach` and `bach-setup.exe` are built from, as
CycloneDX files: `bach-<version>-sbom-cli.cdx.json` and
`bach-<version>-sbom-setup.cdx.json` (each crate with its version, licence
and the checksum from `Cargo.lock`). The runtime's own list is attached to
`bach-setup.exe` as an attestation too, and `bun scripts/sbom.mjs bach-cli
out.json` writes the same list from a checkout. The `.sha256` files remain.
