# Getting help

**Start with the docs.** Most questions are answered there:

- [Getting started](docs/getting-started.md): from nothing to a setup file.
- [Config reference](docs/config.md): every `bach.toml` setting.
- [How it works](docs/how-it-works.md): the setup file, updates and the
  safety rules.
- [Signing](docs/signing.md): code-signing a setup file.

**Then check your setup.** `bach doctor` says whether everything a build
needs is in place, and `bach check` explains problems in your `bach.toml` in
plain sentences.

**Still stuck?** Open an
[issue](https://github.com/usebach/bach/issues/new/choose). Say what you
tried, what you expected and what happened, and include your `bach.toml` and
the Bach version (`bach --version`). If a setup file failed on someone's
computer, the log is at `%TEMP%\<id>-setup.log`.

**Found a security problem?** Don't open an issue. Follow the
[security policy](SECURITY.md).

Bach is maintained in the open by people with other jobs. We read everything
and will answer as soon as we can.
