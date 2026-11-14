# Security policy

Bach writes installers that other people run on their computers, so a
security bug here can reach a lot of machines. Reports are welcome and taken
seriously.

## Supported versions

Bach is early. Only the latest release and `main` get security fixes.

| Version | Supported |
| --- | --- |
| Latest [release](https://github.com/usebach/bach/releases) | Yes |
| `main` | Yes |
| Older releases | No, update to the latest |

Setup files you built with an older Bach keep the runtime they were built
with. After a fix ships, rebuild and re-release your setup file with the new
version of `bach`.

## Report a vulnerability

**Please don't open a public issue or pull request for a security problem.**

Report it privately through GitHub:
[Report a vulnerability](https://github.com/usebach/bach/security/advisories/new).
Only the maintainers can see the report.

Include what you can:

- What the problem is and which part of Bach it touches (`bach`,
  `bach-setup`, the installer scripts, the GitHub Action).
- The version or commit, and your Windows version if it matters.
- Steps to reproduce, ideally a small `bach.toml` or a crafted setup file.
- What an attacker gains: for example "writes a file outside the install
  folder" or "deletes a folder the app never installed".

You'll get an answer within a few days. We'll confirm the problem, work on a
fix with you, and publish an advisory and a release. We credit reporters who
want it. Please give us a reasonable time to ship the fix before you
disclose the details.

## What counts as a vulnerability

Bach's [safety rules](docs/how-it-works.md#safety-rules) are promises. A
report that breaks one of them is a vulnerability, for example:

- A setup file or payload that writes, replaces or deletes anything outside
  its install folder or the data folders the user opted into (path
  traversal, absolute paths, reserved names, links, case tricks).
- An uninstaller or update that removes files it didn't install, or acts on
  a folder that isn't the app's install.
- Setup asking for or gaining administrator rights, or writing outside the
  user's own folders and `HKCU`.
- Changing or deleting registry keys, `PATH` entries, link schemes or file
  types that belong to another program.
- A damaged or edited payload, manifest or update feed that is accepted
  instead of rejected, or a self-update that installs a file that doesn't
  match the feed's SHA-256 or the app's id, or that downloads over http.
- Memory-safety bugs, panics or crashes triggered by an untrusted setup file.
- `install.ps1`, `install.sh` or `action.yml` running or replacing something
  the user didn't ask for.

## What doesn't

- A developer putting something harmful into their own `bach.toml` or app
  files. Bach packages what you give it; the setup file is as trustworthy as
  the app inside.
- Unsigned setup files triggering SmartScreen. See [signing](docs/signing.md).
- The unsigned `uninstall.exe` an install leaves behind. It is unsigned by
  design, because a signature only covers the exact file it was made for.
- Problems that need an attacker who already runs code as the same user.
- Findings in a dependency that Bach doesn't use in a way that is
  reachable. If you're unsure, report it anyway.

If you're not sure whether something is a vulnerability, report it
privately. We'd rather read one report too many.
