# @usebach/cli

The `bach` command: build a modern, clean Windows installer (`Setup.exe`) from one `bach.toml`.

```bash
npm install --global @usebach/cli
bach init        # a starter bach.toml, and a one-click release workflow on GitHub
bach build       # dist/<name>-Setup-<version>.exe
```

It works the same on Windows, macOS and Linux. The package holds no program of
its own: npm installs the matching `@usebach/cli-<platform>` package next to
it, and `bach` starts that. No install script runs and nothing is downloaded
afterwards, so it works with `--ignore-scripts` and from an offline cache.

- **In a project:** `npm install --save-dev @usebach/cli`, then `npx bach build`.
- **Update:** `npm update --global @usebach/cli`.
- **Remove:** `npm uninstall --global @usebach/cli`.

No administrator rights are needed. If npm's global folder isn't writable,
point it at your own with `npm config set prefix ~/.npm-global` instead of
using `sudo`.

Docs: https://github.com/usebach/bach#readme
