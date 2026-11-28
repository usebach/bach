# Threat model

What Bach Installer trusts, what it doesn't, and where the known gaps are. It describes the
code as it is; where a protection does not exist yet, it says so.

## Who and what Bach Installer deals with

| Party | Trusted to | Not trusted to |
| --- | --- | --- |
| The developer (`bach.toml`, the files) | Choose what is installed and run | Name paths outside the install folder, or data folders outside the user's profile |
| The build machine | Run `bach build` | Hand over a swapped `bach-setup.exe` (see below) |
| The setup file on the user's computer | Carry what the developer built | Be well-formed: its manifest and every path are checked before use |
| The update feed host | Say which version is newest | Change what a download contains: the feed's SHA-256 is checked |
| A mirror (`mirrors`) | Offer another place to download the same file | Offer another version or another hash |
| The staging folder `%LOCALAPPDATA%\Bach\updates` | Hold downloads for this user | Be trusted without a check: a staged file is verified before it runs |
| Other users and programs on the computer | Nothing | Write into a machine-wide install folder (its access list is checked) |

## What Bach Installer protects

- **The user's files.** Packaged paths are relative and cannot leave the
  install folder. The uninstaller deletes only files its manifest lists, inside a
  folder that is plainly the app's install, keeps a file that changed after
  setup wrote it, and deletes data folders only when the user asks and only below
  `%APPDATA%`, `%LOCALAPPDATA%` or `%USERPROFILE%` (never `Documents`, `.ssh`,
  developer tool folders, other programs' browser or chat profiles and the like).
  The page lists the folders it would delete, as they are on that computer,
  instead of showing only the developer's description. A first install into a folder that already holds files warns first.
- **The developer's secrets.** `bach build` refuses a source folder that holds
  `.env` files, private keys, certificates, `.git` or `bach.toml` unless
  `files.allow_secrets` says so.
- **The build.** The setup runtime is downloaded over https only. A released
  `bach` has the runtime's SHA-256 built in, so a changed release host or
  `BACH_RELEASES` mirror cannot swap it. A cached copy is checked again before
  every use.
- **Updates.** Feeds and downloads are https only, redirects included. A staged
  update is deleted if it does not match the feed's size and SHA-256. The access
  key of a protected app goes only to the host it was meant for, redirects are
  followed by hand so it is never sent elsewhere, and it is kept sealed with the
  Windows Data Protection API. With `[updates] signer`, a downloaded update must
  also carry a valid Authenticode signature from that publisher.
- **Administrator rights.** Setup asks only when the app says so
  (`installer.scope = "machine"`, an `elevated` step or an elevated `[[migrate]]`).
  A machine-wide install folder must be inside Program Files, and its access list,
  owner, subfolders and files must allow only administrators to change them
  before anything runs elevated.
- **Other programs' settings.** `[[registry]]` cannot write below `Classes`,
  `Policies`, Bach Installer's own key or whole vendors such as Microsoft, Google and
  Mozilla. `[[env]]` cannot set variables that load code into other programs or
  redirect their traffic.

## Known gaps

- An update feed is only as trustworthy as its host unless the app has an
  `[updates] public_key` or `signer`. Without either, a compromised host can
  point to another file together with that file's hash, and the updater will
  accept it. With a `public_key`, every entry must carry the publisher's
  signature (see [signed feeds](updates.md#signed-feeds)). A key can be handed
  over with `next_public_key` in two releases
  (see [Changing the key](updates.md#changing-the-key)), but not revoked remotely.
  A signature has no time unless `[updates] max_age_days` is set: without it a
  taken-over host can keep serving an older entry that is still validly signed
  (a freeze, or a step back to an older version that is still newer than what is
  installed). See [Keeping a signed feed fresh](updates.md#keeping-a-signed-feed-fresh).
- `uninstall.exe` is written on the user's computer and is not signed.
- A run step started with `elevated = true` runs a file from the install
  folder with administrator rights; keep it to well-known installers and sign
  them. Setup checks the file against its recorded SHA-256 right before it asks
  Windows for rights (so a file swapped after the install is not started), but
  a swap in the moment between that check and the start is not prevented, and
  `bach check` warns about the combination with a per-user install. Setup can't
  end an elevated step that runs past `timeout_minutes` unless it is elevated
  itself; it only stops waiting.
- Antivirus heuristics may flag unsigned setup files; see
  [antivirus warnings](antivirus.md).

## Out of scope

- A developer who builds a malicious app: Bach Installer installs what it is given.
- An attacker who is already an administrator on the computer.
- The safety of the installed app's own files and behavior.
