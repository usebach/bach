# AGENTS.md

Instructions for AI coding agents (Claude Code, Codex, and others) working on
**Bach**. Read this before making changes. `CLAUDE.md` points here.

Bach is a shared project. You may be working alongside other people or agents.
Respect their work, stay within the scope of your task, and never discard or
silently overwrite unrelated changes. Before starting, inspect the local Git
state and check whether the repository is current with
https://github.com/usebach/bach. Fetch or update safely where appropriate;
never blindly pull, reset, force-push, or merge over uncommitted work. If the
remote is unavailable, continue from the local checkout and say so.

## What Bach is

**Bach builds modern, clean Windows installers.** A developer writes one
`bach.toml`, runs `bach build`, and gets a single `Setup.exe` with a calm,
native-looking install window, a built-in updater and a clean uninstaller.

- **`bach` (crates/cli)** is the command developers run: `init`, `check`,
  `build`, `preview`, `inspect`. It runs on Windows, macOS and Linux.
- **`bach-setup` (crates/setup)** is the installer and uninstaller window,
  a native GPUI (Rust) app. Every setup file is a copy of it with a payload
  appended.
- **`bach-pack` (crates/pack)** is what both agree on: the config, the
  manifest, the archive, the container format and the path rules. No UI and
  no Windows calls, so it is tested in isolation.

### Product principles

- **Simple for the developer.** One small config file with sensible
  defaults. Every mistake is reported as one sentence naming the setting at
  fault. No script language, no compiler, nothing to learn first.
- **Calm for the end user.** One window, few choices, honest progress,
  readable errors with "Try again", light and dark mode. No administrator
  prompt: installs are per user.
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
                window text in 13 languages), archive.rs, container.rs (the
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
  src/system/     shortcuts, Settings › Apps, known folders; host.rs
                  (languages, free space, lock), env_path.rs (PATH),
                  integration.rs (autostart, link schemes, file types),
                  apps.rs (Restart Manager, open a link, requirements)
  src/ui/         UI primitives (buttons, rows, checkbox, motion, …)
  assets/         Inter, Tabler icons, the Bach mark's petals
examples/hello  a tiny app and the bach.toml that packages it
skills/bach     SKILL.md: how agents install and use Bach (skills.sh)
schema/         bach.schema.json for editors (a test keeps it complete)
scoop/          Bach's own Scoop manifest (the release updates it)
action.yml      the GitHub Action
docs/           getting started, config reference, how it works, signing
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
6. **Communicate usefully.** Finish with what changed, why, which checks
   passed, and any honest limitations.

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
- Setup never asks for administrator rights and never writes outside the
  user's own folders and `HKCU`.
- Do not weaken these rules for convenience, and test them when touching
  `paths.rs`, `work/` or `launch.rs`.
