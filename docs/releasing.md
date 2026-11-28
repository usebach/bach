# Releasing

This page covers the one-click release for a GitHub repository: how to add
it, what it does when you press the button, and how the version is kept in
one place. The apps your users installed update themselves from what it
publishes (see [updates](updates.md)).

## Add the workflow

In your repository, run one of:

```bash
bach init        # a new project: writes bach.toml and the workflow
bach workflow    # a project that already has a bach.toml
```

`bach init` adds the workflow by itself when the folder is a git checkout
whose `origin` is on GitHub; `bach init --no-workflow` skips it. Both write
`.github/workflows/bach-release.yml`. Commit and push it, then start a release
on GitHub under **Actions › Release › Run workflow**, or from a terminal:

```bash
gh workflow run bach-release.yml -f bump=minor
gh workflow run bach-release.yml -f version=2.0.0
gh workflow run bach-release.yml -f bump=patch -f prerelease=true
gh workflow run bach-release.yml -f bump=patch -f dry_run=true
```

Run `bach workflow` again whenever you want the newest version of the
workflow. It only replaces a file that starts with Bach Installer's own first line; any
other file with that name is left alone unless you pass `--force`.

The workflow publishes with a write token, so it names Bach Installer's action by the
commit of the Bach Installer release you ran, as `usebach/bach-installer@<40-digit commit> # v0.x.y`
(looked up with `git ls-remote`; without network it writes `@v0`, says so, and
pins the next time you run it online). The file explains how to keep the pin
current with Dependabot (`package-ecosystem: github-actions`). `bach workflow
--pin float` writes `usebach/bach-installer@v0`, which moves with every release.

## The inputs

| Input | Meaning |
| --- | --- |
| `bump` | Which part of the version goes up: `patch`, `minor`, `major`, or `current` to release the version as it is (the first release). |
| `version` | An exact version like `2.0.0` or `2.0.0-rc.1`. It replaces `bump` and is never lower than the current version. |
| `notes` | What's new, shown by apps that offer the update. Empty: GitHub's generated notes. |
| `prerelease` | Marks the release as a pre-release, so installed apps don't update to it. A version with a `-` is always a pre-release. |
| `draft` | Saves the release as a draft instead of publishing it. |
| `latest` | Turn it off for a fix to an older line, so installed apps stay on the newest release. |
| `dry_run` | Builds everything and keeps the setup file as a workflow artifact, but doesn't commit, tag or publish. |

## What a release does

1. Raises the version in every file that has one ([below](#one-version-everywhere)).
   It stops if `v<version>` already exists as a tag.
2. For a Node.js project, installs the dependencies from the lock file
   ([below](#lock-files)).
3. Commits `Release v<version>` locally and tags it.
4. Builds your app into the folder `[files] source` names. This step depends on
   the project: `cargo build --release` for Rust, `npx tauri build --no-bundle`
   for Tauri, `npx electron-builder --win --dir` (or Electron Forge's
   `package`) for Electron, `dotnet publish -c Release -r win-x64
   --self-contained` for .NET. For a project Bach Installer doesn't recognise the
   **Build your app** step holds a `TODO`: put your build commands there.
   A self-contained .NET publish carries the runtime, so `bach init` leaves the
   `dotnet` requirement commented out. If you change the step to
   `--self-contained false`, uncomment it. Bach Installer 0.11.3 and earlier wrote the
   block uncommented: delete it from such a `bach.toml`.
5. Pushes the commit and the tag together. Nothing is pushed if an earlier
   step failed.
6. Builds the setup file with the [Bach Installer action](updates.md#action-settings),
   writes `latest.json` and publishes both as the GitHub release.

If step 6 fails, the tag is deleted again, but the version files on the branch
already say the new version. Run the workflow again with `bump` set to
`current` to retry.

The workflow commits to the branch it runs on. If that branch is protected,
let `github-actions[bot]` push to it, or run the workflow from a release
branch.

## One version everywhere

The workflow doesn't edit versions itself: it runs `bach version`, which you
can run by hand too.

```console
$ bach version
  bach.toml                    1.4.2
  package.json                 1.4.2
  package-lock.json            1.4.2
Everything agrees on 1.4.2.

$ bach version minor
bach.toml: 1.4.2 → 1.5.0
package.json: 1.4.2 → 1.5.0
package-lock.json: 1.4.2 → 1.5.0
1.5.0
```

| Argument | Result |
| --- | --- |
| none | Lists each file's version. Exit code 1 if they differ. |
| `patch`, `minor`, `major` | Raises that part, from the version in `bach.toml`. |
| `current` | Keeps the version and writes it to every file, bringing them together. |
| `2.0.0`, `2.0.0-rc.1` | Exactly that version, if it isn't lower than the current one. |

Options: `--config FILE` (the files are the ones next to it) and `--dry-run`
(say what would change, write nothing). The new version is the only thing
written to standard output, and the changed files go to standard error.

These files are kept in step, when they exist next to `bach.toml`:

| File | Where |
| --- | --- |
| `bach.toml` | `[app] version` |
| `package.json` | `version` |
| `package-lock.json`, `npm-shrinkwrap.json` | `version` and `packages[""].version`; dependencies are not touched |
| `Cargo.toml`, `src-tauri/Cargo.toml` | `[package]` or `[workspace.package]` `version`, when it is written out (not `version.workspace = true`) |
| `Cargo.lock`, `src-tauri/Cargo.lock` | The crates built from this checkout (those without a `source`); dependencies are not touched |
| `src-tauri/tauri.conf.json` | `version` |
| `*.csproj` | `<Version>`, unless it is a property like `$(…)` |

Only the version text is replaced, so line endings, key order and
formatting stay as they were and the commit shows one changed line per file.

## Lock files

The release installs dependencies with the frozen command of your package
manager (`npm ci`, `pnpm install --frozen-lockfile`, `bun install
--frozen-lockfile`, `yarn install --frozen-lockfile` or `--immutable`),
picked by the lock file that is committed.

If that fails because `package.json` and the lock file have drifted apart,
the workflow recognises the package manager's message, brings the lock file up
to date from `package.json` (what the lock file pins stays pinned), prints a
warning and installs again. The new lock file is part of the release commit.
Any other failure stops the release with the real error. For Yarn Classic
the lock file is updated with a plain `yarn install`.

With no lock file committed the workflow runs `npm install` and says so;
commit a lock file for repeatable releases.

## Update winget with each release

```bash
bach workflow --winget Example.Hello
```

adds a second job, `Update winget`, that runs after the release is published.
It downloads Microsoft's `wingetcreate`, points it at the setup file of the
release and opens the pull request that updates `Example.Hello` in
[microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs).

- The package must already be in winget-pkgs: submit the first version by hand
  (`bach winget`, see [Listing your app in winget](config.md#listing-your-app-in-winget)).
  The job updates it from then on.
- Add a repository secret named `WINGET_TOKEN`: a classic personal access token
  with the `public_repo` scope. Without it the job ends with a notice and
  opens nothing, so the release is never held back.
- The job is skipped for a dry run, a draft, and any pre-release or version
  with a `-` in it.
- Run `bach workflow` again later without `--winget` and the job stays: Bach Installer
  keeps the package id it finds in the file. The id is `Publisher.App`
  (letters, digits, `-` and `_`, parts joined by dots).

To follow the pull request, watch the job's log under Actions: `wingetcreate`
prints the link to it.

## Sign the setup file

The workflow holds a commented `sign-command` input in its last step. Remove
the `#`, keep the certificate in a repository secret, and the setup file is
signed before the update feed is written, so the feed carries the signed
file's SHA-256. See [signing](signing.md).

## A project in a subfolder

When `bach.toml` isn't in the repository's root, `bach workflow desktop` (the
folder with the `bach.toml`) writes a workflow whose steps run in that folder.
Version files are the ones next to `bach.toml`.

## Releasing for macOS from GitHub Actions

A Windows app is never built for macOS by itself. To also release the macOS Setup app, add a `[macos]` section to
`bach.toml` and run `bach workflow --macos`: the generated workflow gets a second job, `macos`, that runs on a Mac after the
release job, builds the Setup app and adds it to the same release and the same update feed (`latest.json`). Run
`bach workflow` again without the flag and the job stays; delete the job from the file to drop it.

Without any secrets the app is sealed ad hoc: it runs, but macOS asks people to confirm the first open. To sign and
notarize it (what Apple expects for software people download), you need the Apple Developer Program (99 US dollars a year).
Then:

1. In your developer account, create a **Developer ID Application** certificate, export it from Keychain Access as a `.p12`
   with a password, and store it base64 encoded: `base64 -i certificate.p12 | pbcopy`.
2. In App Store Connect (Users and Access, Integrations), create an API key and download its `.p8` file; store it base64 encoded.
3. Add these repository secrets (Settings, Secrets and variables, Actions): `MACOS_CERTIFICATE`, `MACOS_CERTIFICATE_PASSWORD`,
   `APPLE_API_KEY`, `APPLE_API_KEY_ID` and `APPLE_API_ISSUER`.
4. In `bach.toml`:

   ```toml
   [macos.signing]
   identity = "Developer ID Application: Example Inc. (ABCDE12345)"
   notarize = "bach-notary"
   ```

The job imports the certificate into a temporary keychain, stores the notarization credentials under the name `bach-notary`,
and builds with `--sign`; Apple's notary service checks the `.dmg` and its ticket is stapled to it, so it opens without a warning.
For a `.pkg` for MDM tools, also create a **Developer ID Installer** certificate and set `installer_identity` (see [`[macos]`](config.md#macos)).

