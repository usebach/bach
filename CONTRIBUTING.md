# Contributing to Bach

Thank you for helping. Bach builds modern, clean Windows installers: one
`bach.toml` in, one `Setup.exe` out. Bug reports, ideas, docs, translations
and code are all welcome.

By taking part you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
For security problems, don't open an issue; follow the
[security policy](SECURITY.md).

## Ways to help

- **Report a bug.** Use the
  [bug report form](https://github.com/usebach/bach/issues/new?template=bug_report.yml).
  A small `bach.toml` that shows it, the output of `bach doctor`, and the
  setup log (`%TEMP%\<id>-setup.log`) make it much faster to fix.
- **Suggest something.** Open a
  [feature request](https://github.com/usebach/bach/issues/new?template=feature_request.yml)
  that starts from what you're trying to do, not from the setting you'd
  like. For anything big, talk to us before you write code.
- **Fix a translation.** Every text in the window exists in 13 languages in
  `crates/pack/src/i18n`. A native speaker's correction is worth a lot.
- **Improve the docs.** If something in [docs](docs) confused you, it
  confuses others.
- **Send a pull request.** Small and focused is easiest to review.

## What Bach is about

Keep these in mind when you propose a change; they decide what fits.

- **Simple for the developer.** One small config file with sensible
  defaults, no script language. A mistake is reported as one sentence naming
  the setting at fault.
- **Calm for the end user.** One window, few choices, honest progress,
  readable errors, light and dark mode, no administrator prompt.
- **Safe by construction.** Packaged paths can't leave the install folder,
  and the uninstaller removes only what it installed.
- **Updates just work**, and setup files in the wild keep working. Changing
  the payload format needs a new `FORMAT_VERSION` and a clear error for the
  old one.

## Set up

You need Rust (stable) and, on Windows, the Visual Studio C++ build tools.
`bach` itself builds on macOS and Linux too; the installer window
(`bach-setup`) is Windows only.

```bash
git clone https://github.com/usebach/bach.git
cd bach
cargo build --release
```

To look at the window without installing anything:

```bash
cargo run -p bach-setup -- --preview
cargo run -p bach-setup -- --preview --uninstall --theme dark --lang de
```

[AGENTS.md](AGENTS.md) describes the code layout, the checks and the
conventions in detail. It is written for coding agents, and it is just as
useful for people.

## Before you open a pull request

Both of these must pass; CI runs them too:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Also:

- **Look at UI changes.** Run `--preview` and check the changed page in
  light and dark, with a long app name and a long install path.
- **Try real installs through the example only.** `scripts/e2e.ps1` installs,
  updates and removes a throwaway app. Don't run a real install of anything
  else on your machine to test a change; `--preview` pretends every step.
- **Add tests** next to the code. Changes to `paths.rs`, `work/` or
  `launch.rs` need tests for the unsafe case, not just the happy one.
- **Update the docs** when a setting, command or behaviour changes:
  `docs/config.md`, `schema/bach.schema.json` (a test keeps it complete) and
  `skills/bach/SKILL.md` if agents need to know.
- **Texts** the window shows are `bach_pack::i18n::Key`s with a translation
  in every language, never a literal in `crates/setup`. Adding a key means
  adding it to all 13 languages and to `docs/config.md`.
- **Use `bun`** for JavaScript, never npm, pnpm or yarn.

## Code style

- Small, cohesive files: aim for under 300 lines, avoid more than 500.
- Reuse what is in `crates/setup/src/ui` before building a new component.
- Colours come from `crates/setup/src/theme.rs` tokens, never a literal.
- Comments explain *why*, in plain sentences.
- Errors are readable sentences, never raw Debug output or JSON.
- Never block or panic on the UI thread: no `unwrap` on runtime data, use
  checked or saturating arithmetic for anything read from a file.
- Paths from a payload are untrusted: always go through `bach_pack::paths`.

## Commits and pull requests

- Make several small commits that each build and pass the tests, not one
  big one.
- Write the subject in the imperative and start with a verb: "Add…", "Fix…",
  "Improve…". The release changelog is generated from these subjects, so
  write for someone reading release notes: "Fix the folder picker on
  Windows", not "fix stuff".
- Branch from `main`, open the pull request against `main`, and fill in the
  template. Say what changed and why, and which checks you ran.
- Never commit secrets, `target/`, `dist/` or `node_modules/`.
- Keep a pull request to one thing. Unrelated cleanups go in their own.

A maintainer will review it, probably ask for a few changes, and merge it
when CI is green.

## License

Bach is [MIT licensed](LICENSE). By contributing you agree that your
contribution is released under the same license.
