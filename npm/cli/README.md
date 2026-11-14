# @usebach/cli

**Modern, clean Windows installers from one small config file.** This package
installs the `bach` command: you describe your app in a `bach.toml`, run
`bach build`, and get a single `Setup.exe` with a calm install window, a
built-in updater and a clean uninstaller. No scripting language, no
administrator prompt for your users.

## Install

```bash
npm install --global @usebach/cli
```

Then check it:

```bash
bach --version
bach doctor
```

It works on Windows, macOS and Linux (x64 and arm64) with Node.js 18 or newer,
and it never needs administrator rights. Don't use `sudo`: if npm says it
can't write to its global folder, give npm a folder of your own once and try
again:

```bash
npm config set prefix ~/.npm-global      # then add ~/.npm-global/bin to your PATH
```

Prefer one project over your whole machine? `npm install --save-dev @usebach/cli`
and run `npx bach build`.

## Your first installer

In your project's folder:

```bash
bach init        # writes bach.toml, filled in for Rust, Tauri, Electron and .NET projects
bach check       # checks it and lists every file the installer will carry
bach preview     # opens the real install window; nothing is installed (Windows)
bach build       # writes dist/<name>-Setup-<version>.exe
```

A complete `bach.toml` can be this short:

```toml
[app]
name = "My App"
version = "1.0.0"
publisher = "Your Name"
executable = "my-app.exe"

[files]
source = "build"
```

## Release from GitHub with one click

In a GitHub repository, `bach init` also writes a release workflow (or run
`bach workflow` in a project that already has a `bach.toml`). Commit and push
it, then press **Actions > Release > Run workflow**. It raises the version in
`bach.toml`, `package.json` and its lock file, `Cargo.toml` and the other
version files, builds your app and the installer, and publishes a release your
installed apps update from.

`bach version patch|minor|major` does the version part on its own.

## Update and remove

```bash
npm update --global @usebach/cli
npm uninstall --global @usebach/cli
```

## How it works

This package holds no program itself. npm installs the matching
`@usebach/cli-<platform>` package next to it (it is an optional dependency,
chosen by your operating system and CPU), and `bach` starts that. No install
script runs and nothing is downloaded afterwards, so it works with
`--ignore-scripts`, behind proxies and from an offline cache. If `bach` says
its platform package isn't installed, npm ran with `--omit=optional`: install
again without that flag.

Without Node.js, the install scripts do the same job:
[github.com/usebach/bach](https://github.com/usebach/bach#quick-start).

## Links

- Documentation: https://bach.mintlify.app/docs
- Source and issues: https://github.com/usebach/bach
- License: MIT
