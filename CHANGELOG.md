# Changelog

What changed in each release of Bach, newest first. The release workflow writes this from the commit history.

## 0.2.0 (2026-11-14)

### New

- Add automatic updates, a GitHub shortcut and a version override to the config ([71cf34a](https://github.com/usebach/bach/commit/71cf34abbdf94f16ee60f1f287c64486383a858c))
- Add weekly Dependabot updates for Cargo, Actions and Bun ([7e502ab](https://github.com/usebach/bach/commit/7e502aba816e25c795c758f983d918d047623adc))
- Add a pull request template with the checks and a safety question ([eb7f5a6](https://github.com/usebach/bach/commit/eb7f5a633dd0fd81bbc8f7a4d77c3ef1bd335caf))
- Add issue forms for bugs, feature requests and translations, and a support page ([9d0f690](https://github.com/usebach/bach/commit/9d0f6903d083b933f962ffb1cf6cdec3eba7e0c7))
- Add a contributing guide and link the community files from the README ([9ac0f1d](https://github.com/usebach/bach/commit/9ac0f1d946ad9bd40fba99d0f5c85503b80aef49))
- Add a code of conduct based on the Contributor Covenant ([ef84c5c](https://github.com/usebach/bach/commit/ef84c5c59a9672f3213de9d853a6af3677025fc7))
- Add a security policy with private vulnerability reporting ([db34ff5](https://github.com/usebach/bach/commit/db34ff52e00e93b1e4985e25a22e7da64f322f82))

### Improvements

- Document updates from GitHub, the background updater and the action ([15e738e](https://github.com/usebach/bach/commit/15e738ec1568cac3d06a0e205a2026ca23acaab9))
- Make the action a whole release pipeline and let people pick the Bach version ([6a7ce1a](https://github.com/usebach/bach/commit/6a7ce1abb7f9529327e0b8f4e4aeb4f6a3fc0a0e))
- Test the update flow for real: staged downloads, wait-for, launch, cut-short updates, release builds ([9130084](https://github.com/usebach/bach/commit/9130084404df8ad015772271a4b6f91c96c25dd7))
- Take the version from a release tag and write the feed from the config ([8e0f665](https://github.com/usebach/bach/commit/8e0f665270b7d770b3b2ad13dc4059986759e407))
- Download updates in the background and install them when the app is closed ([36be987](https://github.com/usebach/bach/commit/36be987d78534ac7589b620dfcf5cd9149f7cedf))
- Roll back an update that a power cut or crash stopped halfway ([a0ae633](https://github.com/usebach/bach/commit/a0ae633d25dd5f9110068ecb11e865ba04b72912))
- Update: Bump @tabler/icons-react from 3.50.0 to 3.51.0 ([7caea3c](https://github.com/usebach/bach/commit/7caea3c1a89bb791e67ac350ef35548503688478))

All changes: [v0.1.1...v0.2.0](https://github.com/usebach/bach/compare/v0.1.1...v0.2.0)

## 0.1.1 (2026-11-14)

### New

- Add bach feed, bach winget and stack detection to bach init ([f90e4f9](https://github.com/usebach/bach/commit/f90e4f9fe611f88cb6f1341c5f1dfc36be905707))
- Add keyboard use, requirement checks, self-updates and logs to setup ([ebd1f8a](https://github.com/usebach/bach/commit/ebd1f8a2e0e536606ca4f136b67025e27d2889dc))
- Add requirements, extra shortcuts and update feeds to bach.toml ([f74dfe4](https://github.com/usebach/bach/commit/f74dfe4b7e05dc39f66d6335593517151df55f55))
- Add LICENSE file ([e5cfc10](https://github.com/usebach/bach/commit/e5cfc10cb9c8c43b8915a6f29123833e0b870e2b))

### Fixes

- Fix error messages capitalising file and setting names ([b22e285](https://github.com/usebach/bach/commit/b22e285d6a40fee68bace17f0c02ee9dcb95e0f8))
- Fix bach build failing on every app.icon ([23309ff](https://github.com/usebach/bach/commit/23309ffec85510ff17e5cb3c13765fe7dec74f8f))
- Fix install.ps1 replacing another app called Bach ([d0cfa55](https://github.com/usebach/bach/commit/d0cfa550f29a48872280984f1e4f64f3e1c81bd5))

### Improvements

- Rewrite the README around what Bach is, why and how to start ([586e3ba](https://github.com/usebach/bach/commit/586e3bae446cd5fddfa778a433e1830f27f9ecdf))
- Teach the agent skill about requirements, updates and winget ([bbd89be](https://github.com/usebach/bach/commit/bbd89be4264e6a625960e88c8356a7b48cfaeed7))
- Document the new settings and commands, with screenshots ([c844ca5](https://github.com/usebach/bach/commit/c844ca52e9741bb19ad6a32e2bb585176a061a6d))
- Publish Windows ARM64 builds, a Scoop manifest and a GitHub Action ([f9ee960](https://github.com/usebach/bach/commit/f9ee9605060235b088ea29384658eb243eaa9fe7))
- Test real installs, updates and removals in CI ([ff368e0](https://github.com/usebach/bach/commit/ff368e03663204cd73970bc1595df85adf6f7eaf))
- Install apps with thousands of files twice as fast ([d13cc33](https://github.com/usebach/bach/commit/d13cc3394d2f0f5e5e08a9763fb4c469a5d1e34e))
- Switch the license metadata to MIT ([aa36528](https://github.com/usebach/bach/commit/aa365288978673648a74b6cdc35e070601415d59))

All changes: [v0.1.0...v0.1.1](https://github.com/usebach/bach/compare/v0.1.0...v0.1.1)

## 0.1.0 (2026-11-13)

### New

- Add a Bach skill for coding agents ([2eef408](https://github.com/usebach/bach/commit/2eef408e30f7524b9920162679506c07e434d542))
- Add bach doctor and fetch the setup runtime when it's missing ([568ad80](https://github.com/usebach/bach/commit/568ad801d1b5dcd27520052edef39d0c936abfea))
- Add installer options, components, languages and a license page ([c65e475](https://github.com/usebach/bach/commit/c65e4758c381785428cb098e7c48cf8f209a8f85))
- Add the setup window's texts in 13 languages ([76fb489](https://github.com/usebach/bach/commit/76fb489b880ae51524d9a94bb430fd5f6c52c713))

### Improvements

- Rewrite the README ([ddd05ad](https://github.com/usebach/bach/commit/ddd05adb5a22468b1cbbb287746d7ef0a4ee5d5e))
- Document every new setting ([14c5c71](https://github.com/usebach/bach/commit/14c5c715bd90ed51fa2c1ab672ced6958e885759))
- Install bach with one line, no Rust needed ([0b98246](https://github.com/usebach/bach/commit/0b98246fca2491ccecdf83b15d58b3b2d21453bc))
- Make whole rows of choices clickable ([0587349](https://github.com/usebach/bach/commit/0587349c339a72149a4672927896cdd6e75d83d5))
- Link the C runtime into every Windows binary ([f7167f8](https://github.com/usebach/bach/commit/f7167f8abeabd6d7dd5d570c9991b74508310ac4))
- Start Bach as a Windows installer builder ([34d1ca4](https://github.com/usebach/bach/commit/34d1ca4883b83286c40a732f0e5c005a3529bc8b))

