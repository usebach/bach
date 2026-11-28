# AGENTS.md

Instructions for AI coding agents (Claude Code, Codex, and others) working on
**Bach Installer**. Read this before making changes. `CLAUDE.md` points here.

Bach Installer is a shared project. You may be working alongside other people or agents.
Respect their work, stay within the scope of your task, and never discard or
silently overwrite unrelated changes. Before starting, inspect the local Git
state and check whether the repository is current with
https://github.com/usebach/bach-installer. Fetch or update safely where appropriate;
never blindly pull, reset, force-push, or merge over uncommitted work. If the
remote is unavailable, continue from the local checkout and say so.

## What Bach Installer is

**Bach Installer builds modern, clean Windows installers, and macOS ones.** A developer
writes one `bach.toml`, runs `bach build`, and gets a single `Setup.exe` with a
calm, native-looking install window, a built-in updater and a clean uninstaller.
With a `[macos]` section the same run also writes a Setup app in a `.dmg`, which
does the same on a Mac (see "macOS" below).

- **`bach` (crates/cli)** is the command developers run: `init`, `check`,
  `build`, `preview`, `inspect`. It runs on Windows, macOS and Linux.
- **`bach-setup` (crates/setup)** is the installer and uninstaller window,
  a native GPUI (Rust) app. Every Windows setup file is a copy of it with a
  payload appended; on macOS it is `Contents/MacOS/bach-setup` in a Setup app,
  with the payload in `Contents/Resources/setup.bach`.
- **`bach-pack` (crates/pack)** is what both agree on: the config, the
  manifest, the archive, the container format and the path rules. No UI and
  no Windows calls, so it is tested in isolation.

### Product principles

- **Simple for the developer.** One small config file with sensible
  defaults. Every mistake is reported as one sentence naming the setting at
  fault. No script language, no compiler, nothing to learn first.
- **Calm for the end user.** One window, few choices, honest progress,
  readable errors with "Try again", light and dark mode. No administrator
  prompt by default: installs are per user. A machine-wide install
  (`installer.scope = "machine"`) or an `elevated` run step asks Windows once.
- **Safe by construction.** Packaged paths can't leave the install folder.
  The uninstaller removes only what it installed, deletes data only when the
  user asks, and only inside the user's profile.
- **Updates just work.** A newer setup file over an install updates it in
  place and removes files the old version had and the new one doesn't.
- **Stable format.** Setup files in the wild must keep working. Change the
  payload format only with a new `FORMAT_VERSION` and a clear error for the
  old one.

The installer window's look comes from Bach's design system (originally the
Bach desktop app). Keep it: Inter, the token colours in
`crates/setup/src/theme.rs`, the springs in `ui/motion`, the Bach mark.

## Layout

```
crates/pack     bach-pack: bach.toml (config/), manifest (manifest.rs and
                manifest/: choices, components, validation), i18n/ (every
                window text in 19 languages), archive.rs, container.rs (the
                payload on the exe), paths.rs (safety rules)
crates/cli      bach-cli, binary `bach`: commands, runtime lookup and
                download, doctor, init's stack detection (detect.rs), the
                update feed (feed.rs), winget manifests, Windows resources,
                `bach version` (version.rs, version_files.rs: one version in
                every file) and `bach workflow` (workflow.rs and
                workflow/release.yml: the one-click GitHub release)
npm/cli         @usebach/cli, the npm package: a shim that starts the native
                program from @usebach/cli-<platform> (built by
                scripts/build-npm.mjs, published by the release workflow)
crates/setup    bach-setup: the installer window
  src/launch.rs   what this run is (install, remove, preview, silent)
  src/silent.rs   --silent: the same jobs without a window
  src/package.rs  the payload read from its own file
  src/start.rs    fresh install or update, and the choices to start from
  src/texts.rs    built-in texts + the developer's [text], filled in
  src/view.rs     window state; pages/ draws it; actions.rs (links,
                  requirements, close the app), keyboard.rs (Tab, Space, Esc)
  src/updater.rs  --check-update / --update from the installed app
  src/log.rs      %TEMP%\<id>-setup.log
  src/work/       install (place.rs: all-or-nothing swap), remove, preview
  src/system_mac/ the same API for macOS (links, launch agents, PATH block,
                  install record, Keychain, osascript elevation); chosen
                  with `#[path]` in main.rs, so shared code is platform-free
  src/system/     shortcuts, Settings › Apps, known folders; host.rs
                  (languages, free space, lock), env_path.rs (PATH),
                  integration.rs (autostart, link schemes, file types),
                  apps.rs (Restart Manager, open a link, requirements)
  src/ui/         UI primitives (buttons, rows, checkbox, motion, …)
  assets/         Inter, Tabler icons, the Bach mark's petals
examples/hello  a tiny app and the bach.toml that packages it
skills/bach     SKILL.md: how agents install and use Bach Installer (skills.sh)
schema/         bach.schema.json for editors (a test keeps it complete)
scoop/          Bach Installer's own Scoop manifest (the release updates it)
action.yml      the GitHub Action
docs/           getting started, config reference, how it works, signing
                (Markdown in the repo; the full docs site is a Mintlify
                project, see "Documentation" below)
scripts/        build-release.ps1, build-example.ps1, release and asset scripts
install.ps1/.sh one-line installs of bach without Rust (never into a
                folder they didn't make: Programs\Bach may be another app)
```

## Commands

```bash
cargo build --release                         # bach.exe and bach-setup.exe
cargo run -p bach-setup -- --preview          # the window, nothing installed
cargo run -p bach-setup -- --preview --uninstall --theme dark --lang de
cargo run -p bach-cli -- doctor examples/hello/bach.toml
cargo test --workspace
cargo clippy --workspace --all-targets        # must be warning-free
powershell -ExecutionPolicy Bypass -File scripts\e2e.ps1   # real install/update/uninstall of a throwaway app
scripts/e2e-macos.sh                          # the same on a Mac, in a throwaway HOME
scripts/build-example.sh                      # dist/Hello-Setup-1.0.0.dmg
powershell -ExecutionPolicy Bypass -File scripts\build-example.ps1   # dist\Hello-Setup-1.0.0.exe
powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1   # dist\bach-<version>-windows-x64.zip
gh workflow run release.yml -f bump=minor     # changelog, zip, GitHub release
bun install && bun scripts/export-icons.mjs IconName ...   # Tabler icons
```

Use **Bun** for JavaScript, never npm, pnpm or yarn. On Windows, a running
`bach-setup.exe` (or a setup file built from it) locks the build output;
close it before rebuilding.

**Never run a real install or uninstall on the developer's machine without
being asked.** Use `--preview`, which pretends every step and changes
nothing. When a real end-to-end test is needed, install the example app
(`Hello`) and remove it again through its uninstaller.

## How to work: thoughtful, efficient, and thorough

**Own the task and deliver genuinely excellent work.** The default is one
capable primary agent working directly. Your primary model is selected by the
user or host environment; this file cannot and should not change it.

0. **Start from the current repo, every time.** Before you read or change
   anything, run `git fetch origin` and compare your checkout with
   `origin/main` (`git status -sb`, `git log HEAD..origin/main`). If you are
   behind, bring it up to date first: fast-forward when the tree is clean;
   with uncommitted work, stash it, branch from `origin/main`, and restore it
   (never reset, never discard, never merge over someone else's changes).
   Start new work on a fresh branch from the updated `origin/main`. If the
   remote is unreachable, say so and continue from what you have.
1. **Understand before editing.** Read the relevant code, trace existing
   behavior and contracts, and check nearby patterns. For substantial work,
   make a concise plan with acceptance criteria. For a trivial fix, act.
2. **Implement the complete experience.** Consider the developer writing
   `bach.toml`, the end user clicking through setup, error and retry states,
   updates over older installs, and the uninstaller. Keep scope disciplined.
3. **Take quality seriously.** Think through edge cases: files in use, long
   paths, odd app names, interrupted copies, signed files, damaged payloads.
4. **Use time and tokens intelligently.** Investigate enough to be confident;
   don't repeat equivalent analyses or skip tests to save tokens.
5. **Verify evidence, not confidence.** Run targeted tests, then the full
   checks. For UI, run `--preview` and look at the changed page in light and
   dark. If something can't be verified, say exactly what.
6. **Keep the docs true.** If a feature is new or changed, update the docs
   in the same piece of work (see "Documentation").
7. **Communicate usefully.** Finish with what changed, why, which checks
   passed, which docs pages you updated, and any honest limitations.

## macOS

macOS is a first-class platform, built from the same `bach.toml` (`[macos]`,
`crates/pack/src/config/macos.rs`). The rules that keep it right:

- **The payload is never appended to a Mach-O program** (it breaks code signing);
  it is `Contents/Resources/setup.bach`. `bach_pack::macapp` is the one place that
  knows the bundle layout and `Info.plist`.
- **Shared code asks `crate::system` and `work::uninstaller`**, never `cfg(windows)`
  for behaviour. A Windows-only idea (registry, services, MSI, ACLs) is a no-op in
  `system_mac`, and the config leaves it out of a macOS build.
- **Mac needs `xcodebuild -downloadComponent MetalToolchain`** before GPUI compiles, if the
  Xcode in use doesn't have it (`xcrun metal --version` says so).
- **Tests:** `cargo test --workspace` and clippy must be green on a Mac too, and
  `scripts/e2e-macos.sh` must pass. Never run a real install on the developer's own
  profile: that script sets a throwaway `HOME`, and so must any other test.
- The window can't be screenshotted from a headless session; say so when a window change
  wasn't looked at, and look at it with `cargo run -p bach-setup -- --preview` when you can.

## Documentation

The docs are part of the product. A feature is not done until a developer
can find how to use it, and nobody reads a page that is no longer true.

**When to update the docs.** In the same piece of work, whenever you add,
change, rename or remove any of these:

- a `bach.toml` setting, its default, its accepted values or its rules;
- a `bach` command, flag, argument, output line or exit code;
- a flag or exit code of a setup file or `uninstall.exe`;
- a window text (`Key`), a page or state of the installer window;
- an environment variable, a file, folder or registry location Bach Installer uses;
- install, update, removal or safety behaviour (`paths.rs`, `work/`,
  `launch.rs`, `updater.rs`);
- install scripts, the GitHub Action, Scoop, winget output, or the update
  feed format;
- a release (changelog), or anything a user would be surprised by.

**Where the docs live.** There are two places, and both stay in sync:

1. **The docs site**, a Mintlify project (subdomain `bach`, served under
   `/docs`). Edit it with the Mintlify Admin MCP tools (`list_nodes`,
   `read`, `edit_page`, `write_page`, `create_node`, `update_config`, ...)
   on the deploy branch `main`. Read the Mintlify skill
   (`mintlify:mintlify`) first. **Always publish** the docs site yourself
   once your edits are done, without asking, and check with
   `deployment_status` that it is live.
2. **The Markdown in the repo**: `docs/*.md`, `README.md`,
   `schema/bach.schema.json` (a test keeps it complete) and
   `skills/bach/SKILL.md` (if agents need to know).

**Which page covers what.**

| Change | Site pages | Repo files |
| --- | --- | --- |
| `bach.toml` setting | `reference/config/*` (one page per section), `guides/*` that mention it | `docs/config.md`, `schema/bach.schema.json`, `SKILL.md` |
| Window text | `reference/config/text`, `concepts/installer-window` | `docs/config.md` |
| `bach` command or flag | `reference/cli/*`, `reference/cli/overview` | README commands table, `SKILL.md` |
| Setup or uninstaller flag, exit code | `reference/setup/command-line`, `reference/setup/exit-codes`, `guides/silent-installs` | `docs/config.md` |
| Env variable, file or registry location | `reference/setup/environment-variables`, `reference/setup/files-and-locations` | |
| Install, update, remove, safety behaviour | `concepts/how-it-works`, `concepts/safety`, `guides/updates`, `guides/uninstall-and-user-data` | `docs/how-it-works.md` |
| Signing, CI, winget | `guides/signing`, `guides/ci`, `guides/winget` | `docs/signing.md` |
| New release | `resources/changelog` | `CHANGELOG.md` (the release workflow writes it) |
| New page | Add it to the navigation, link it from a related page | |

**How to write docs here.**

- Write only what is true. Every setting, flag, default, message and example
  must come from the code or a command you ran. If you cannot verify
  something, leave it out or say so; never invent a plausible detail.
- English, second person, sentence-case headings, active voice, no marketing
  words. Every page opens with a plain paragraph that says what it covers,
  and has a `title`, a `description` of 130 to 160 characters and, for long
  titles, a short `sidebarTitle`.
- Show a working example for every setting or command, with the expected
  output where there is one. Code blocks have a language tag; TOML examples
  are titled `bach.toml`.
- Link internal pages with root-relative paths and no extension
  (`/guides/updates`). Describe the user's goal first, then the settings.
- Use the site's components (`Steps`, `Tabs`, `Card`, `Accordion`, `Note`,
  `Warning`) instead of raw HTML.
- Screenshots of the window are real, in English, and come from
  `bach preview --lang en` (light and dark; `BACH_PREVIEW_DIR` hides the
  real install path). Keep the raw capture in `docs/images/raw/` and make
  the published image with `python scripts/polish-screenshot.py RAW OUT
  --edge light|dark`: it removes the desktop ring and corners, neutralises
  the title bar and adds a shadow. Never publish a raw capture.
- Theme and look live in the Mintlify project's `docs.json` (Maple theme,
  Inter, blue accent `#2563EB`, dark surface `#181818` to match
  `crates/setup/src/theme.rs`). Change the look there, not page by page.
- Never put secrets, personal paths or machine names in the docs.

**Before you finish.** List the docs pages you changed, or say "no docs
needed" and why. A change to settings, flags or behaviour with no docs
change is not done.

### Subagents: use rarely, and never Opus

Subagents are tools, not the default process. Normally use zero. Start one
only for a substantial, independent task (a bounded investigation, or a large
implementation with disjoint files). Never choose Opus for a subagent; use
Haiku for mechanical work and Sonnet for bounded code or investigation. Give
a precise brief and verify the result yourself. No agent swarms.

## Quality bar: what "done" means

- **Correct:** works end to end, including errors, retries and updates.
- **Consistent:** follows the existing architecture and components.
- **Polished:** clear copy, precise spacing, finished states in both themes.
- **Robust:** handles malformed data, files in use and failures without
  crashing or leaving a half-installed app.
- **Maintainable:** clear names, cohesive modules, focused tests, comments
  that explain why.
- **Verified:** tests and clippy pass; changed UI was looked at.
- **Documented:** new and changed features are in the docs, and the pages
  you touched are named in your summary.

## UI and interaction quality

The setup window should feel like a refined native tool: one calm column,
the mark, a heading, one line of copy, a card of choices, buttons at the
bottom. Pages rise in as they replace each other; nothing jumps.

- Every colour comes from `theme.rs` tokens, never a literal. The app's
  accent replaces only `primary` (and its hover and ink).
- Copy uses sentence case, names the app ("Install Hello"), and says what
  will happen. Errors are human sentences with a way forward.
- Every text the window shows is a `bach_pack::i18n::Key` with a
  translation in every language under `crates/pack/src/i18n`; never write a
  user-facing literal in `crates/setup`. Adding a key means adding it to
  every language (the build fails otherwise) and to docs/config.md.
- Rows of choices are whole click targets (`Row::on_click`); a control
  inside a row stops propagation so a click never counts twice.
- Finish every state: ready, working, done, failed, update, reinstall,
  remove, preview, and the "nothing to install" problem page.
- Test long app names and long install paths; text must wrap, not clip.

## Commits

- Commit focused, working slices once their tests pass. Imperative,
  human-readable subjects ("Fix the folder picker on Windows").
- Before finalizing, `cargo test --workspace` and
  `cargo clippy --workspace --all-targets` must be green.
- Never commit secrets, `target/`, `dist/` or `node_modules/`.
- Subjects feed the release changelog (`scripts/release-notes.mjs` groups by
  their first word, e.g. "Add" or "Fix"); write for release-note readers.

## Code rules

- **Small, cohesive files.** Aim for under 300 lines, avoid exceeding 500.
- **Reuse.** Check `crates/setup/src/ui` before building a component.
- **Readable, documented code.** Doc comments explain why and any surprising
  contracts, in plain sentences.
- **Errors** are readable sentences in the CLI and the window; never show raw
  Debug output or JSON.
- **Never block the UI thread.** File copies, COM and the registry (beyond
  the quick read at startup) belong on the worker thread.
- **Never panic on the UI thread.** No `unwrap` on runtime data, no unchecked
  arithmetic on lengths read from files; use checked/saturating math.
- **Paths from a payload are untrusted.** Always go through
  `bach_pack::paths` before touching the disk.

## GPUI pitfalls we already paid for

- `svg()` paints using its own text color. Use `ui::Icon`, which falls back
  to the inherited color like CSS `currentColor`. SVGs render as one-colour
  masks; full-colour logos must be PNG.
- On Windows, GPUI's window `display_handle()` is `unimplemented!()`.
  Open folder pickers through `ui::folder_dialog::pick_folder` only.
- GPUI loads the window icon from icon resource id 1; `bach build` replaces
  that icon in the neutral language, which `crates/setup/build.rs` uses.
- Style helpers must not set `position`: `.relative()` after `.absolute()`
  silently wins.
- There is no backdrop blur; surfaces are opaque.
- Icons: export with `scripts/export-icons.mjs`; stroke variants use
  `name@2.5.svg`.

## Security

- A setup file's payload is untrusted input for the runtime: validate the
  manifest and every path before use.
- The uninstaller must only delete files listed in its manifest, inside a
  folder that is verifiably the app's install, plus data folders the user
  opted into, which must sit inside the user's profile.
- Setup asks for administrator rights only when the app says so
  (`installer.scope = "machine"`, an `elevated` run step, or `elevated` on a
  `[[migrate]]`), and without that never writes outside the user's own
  folders and `HKCU`. Anything it runs elevated comes from a place only
  administrators can write, and Windows' folders come from Windows, not from
  environment variables.
- Do not weaken these rules for convenience, and test them when touching
  `paths.rs`, `work/` or `launch.rs`.
