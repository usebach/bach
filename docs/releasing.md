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
workflow. It only replaces a file that starts with Bach's own first line; any
other file with that name is left alone unless you pass `--force`.

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
   --self-contained` for .NET. For a project Bach doesn't recognise the
   **Build your app** step holds a `TODO`: put your build commands there.
5. Pushes the commit and the tag together. Nothing is pushed if an earlier
   step failed.
6. Builds the setup file with the [Bach action](updates.md#action-settings),
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

## Sign the setup file

The workflow holds a commented `sign-command` input in its last step. Remove
the `#`, keep the certificate in a repository secret, and the setup file is
signed before the update feed is written, so the feed carries the signed
file's SHA-256. See [signing](signing.md).

## A project in a subfolder

When `bach.toml` isn't in the repository's root, `bach workflow desktop` (the
folder with the `bach.toml`) writes a workflow whose steps run in that folder.
Version files are the ones next to `bach.toml`.
