# Changelog

What changed in each release of Bach, newest first. The release workflow writes this from the commit history.

## 0.11.5 (2026-11-28)

### Improvements

- Update: Bump the cargo group with 5 updates ([72952ac](https://github.com/usebach/bach/commit/72952ac86683f1707998215854869de7dace0dff))
- Update: Bump EmbarkStudios/cargo-deny-action ([2de66ed](https://github.com/usebach/bach/commit/2de66edb63536d018ea34879ba8901cf37f07956))

All changes: [v0.11...v0.11.5](https://github.com/usebach/bach/compare/v0.11...v0.11.5)

## 0.11.4 (2026-11-27)

### Fixes

- Fix bach init asking .NET users for a runtime the release already ships ([e75e74c](https://github.com/usebach/bach/commit/e75e74c168fda0ae236e442a51582df7c781a684))

### Improvements

- Document that the .NET requirement starts commented out ([3c1eb86](https://github.com/usebach/bach/commit/3c1eb8619ebccc83758c3f04a598cf0154b6ac1e))

All changes: [v0.11...v0.11.4](https://github.com/usebach/bach/compare/v0.11...v0.11.4)

## 0.11.3 (2026-11-26)

### Fixes

- Fix outdated facts about requirements, errors and update trust in the docs ([48be13c](https://github.com/usebach/bach/commit/48be13c8a531cf341d1417ff7a284b3d7831f6c4))
- Fix the gap in the message about a wrong publisher's signature ([68e547c](https://github.com/usebach/bach/commit/68e547ccee428ddbb0fa3e7eb24fee7d7c790328))

### Improvements

- Point the npm package's docs link at usebach.app ([0c38666](https://github.com/usebach/bach/commit/0c386660fe60f95c977f692c54c7b765e8f27936))
- Rewrite the getting started guide in plain English ([d00a572](https://github.com/usebach/bach/commit/d00a5722990bd3c768f71cac745d41069dab1426))
- Document the signer check of a Mac build ([c7f481a](https://github.com/usebach/bach/commit/c7f481a092a2ab320067d0464b347969a3f0ce01))
- Fail a signed Mac build, and warn on an unsigned one, that installed apps would refuse ([3fdfdfe](https://github.com/usebach/bach/commit/3fdfdfe9d99cf014af2c46f8ba31143811e9fcab))
- Move the Mac signature check to bach-pack so the builder can ask it too ([5c8058e](https://github.com/usebach/bach/commit/5c8058e661bad3e07c31310b7a09c776f2192cfa))

All changes: [v0.11...v0.11.3](https://github.com/usebach/bach/compare/v0.11...v0.11.3)

## 0.11.2 (2026-11-26)

### New

- Add a Mac script that tries a self-signed publisher on the update check ([eb118a0](https://github.com/usebach/bach/commit/eb118a0d931cbff13f6857ef938b707d4a806b70))

### Fixes

- Fix the refused signature message naming Windows on a Mac ([929fc1f](https://github.com/usebach/bach/commit/929fc1f6d76c08229c7195278b7a7a99ec04dfdc))
- Fix the macOS update check to accept only an Apple-issued Developer ID ([1c2d72b](https://github.com/usebach/bach/commit/1c2d72b2b65cd4a088dd6dd98aa8c30048a8edf4))

### Improvements

- Say in the config reference what signer accepts on a Mac ([711b11a](https://github.com/usebach/bach/commit/711b11ae4b836a05f23b835445f8a000298323a9))
- Document signed updates on a Mac ([04d638a](https://github.com/usebach/bach/commit/04d638adc35c6b90044067673f1af966f34ec498))
- Run the signer check in the macOS CI job ([6f16ae7](https://github.com/usebach/bach/commit/6f16ae70a9c9b2138985c2a8388ad0423a29a834))
- Test who the macOS update check takes for the publisher ([4489ad2](https://github.com/usebach/bach/commit/4489ad2bad3438a989fb6546a78330e74869a3f5))

All changes: [v0.11...v0.11.2](https://github.com/usebach/bach/compare/v0.11...v0.11.2)

## 0.11.1 (2026-11-26)

### New

- Show every claim in bach check and inspect, and refuse them in an MSI ([9e58e74](https://github.com/usebach/bach/commit/9e58e749fc52ed79d80bfbded09ee322600a0edd))
- Add installer.shared_files so a file can need several components ([09b33c4](https://github.com/usebach/bach/commit/09b33c4ac470b739f7372e8c995266942d429713))

### Fixes

- Fix the platform field missing from the --json output schema ([6448fe3](https://github.com/usebach/bach/commit/6448fe30f55a33690f6410c194cff98c79c3ce77))

### Improvements

- Document files that need several components ([2eb6f47](https://github.com/usebach/bach/commit/2eb6f479f78029b1fb458ae67dc732348aba3d65))
- Install shared files only when all their components are ticked ([4a3e267](https://github.com/usebach/bach/commit/4a3e267219dcdbf902eff29dd01b585810d5e42d))
- Move the component rules into their own file ([b7ea484](https://github.com/usebach/bach/commit/b7ea4840a5ddfee826756fceff6e04ae9b7525bb))

All changes: [v0.11...v0.11.1](https://github.com/usebach/bach/compare/v0.11...v0.11.1)

## 0.11.0 (2026-11-22)

### New

- Let go of the disk image reliably after laying out its window ([4a0e48b](https://github.com/usebach/bach/commit/4a0e48bd88810405eae768c18e06ac37ef0839c3))
- Add the macOS end-to-end script and the example app ([a49015a](https://github.com/usebach/bach/commit/a49015a4e4171a401b6f6fc65d8e9194ff5bae52))
- Add the macOS system layer: links, launch agents, PATH and the install record ([1aa633e](https://github.com/usebach/bach/commit/1aa633efdc60c2424fc5d673bd4495fe70c529b0))
- Offer a Mac job in the generated release workflow ([92f4b6a](https://github.com/usebach/bach/commit/92f4b6aed54adf89204773694e0aba333bffc714))
- Add the macOS window texts in every language ([464760d](https://github.com/usebach/bach/commit/464760d7441213dc648e310677acb6bac44f4293))
- Add the path rules for macOS data folders and links ([596f2b9](https://github.com/usebach/bach/commit/596f2b99364573fdda63063ae7bb6b423e1aa124))
- Add the macOS platform, file modes and links to the manifest ([0640c34](https://github.com/usebach/bach/commit/0640c3474398ca805e857edb06930cb0d16b29b3))

### Improvements

- Build only the Windows file in the Windows CI and example script ([6cc4a7c](https://github.com/usebach/bach/commit/6cc4a7cb519dffb1855067397c97dd14829c2cf1))
- Document macOS in the reference, guides and agent notes ([ebe31e2](https://github.com/usebach/bach/commit/ebe31e20b23647ad742be4c92098870183ce4c05))
- Run the macOS jobs in CI, the release and the action ([5e490a0](https://github.com/usebach/bach/commit/5e490a0238ab1f1fa4450134c58fab2fcaac6949))
- Update from a downloaded disk image or zip on macOS ([551c727](https://github.com/usebach/bach/commit/551c7275b2e65b6bb467b1bf635cd7f8e7e3918e))
- Install, update and remove on macOS with permissions and links ([d6570b4](https://github.com/usebach/bach/commit/d6570b44fe81e9ee5931e8e136791b996ce81ded))
- Run the setup runtime on macOS ([86921ca](https://github.com/usebach/bach/commit/86921caf3593500a4f54c53d96f3084343fecc6b))
- Test macOS builds end to end in the CLI ([de5423c](https://github.com/usebach/bach/commit/de5423c19786ed3d87c07e0be152b1d9ba6e982b))
- Read macOS setup files in inspect, verify, diff and preview ([a128cea](https://github.com/usebach/bach/commit/a128cead811fc5e611c40d43aadceed24807e2ea))
- List the macOS setup file in the update feed ([d6c650d](https://github.com/usebach/bach/commit/d6c650d0d4190193067a422c176625973e9977eb))
- Build the macOS Setup app in a disk image or a zip ([b2c9ab1](https://github.com/usebach/bach/commit/b2c9ab1e7654cc18552b7f9f344a1f591ac87fe5))
- Find, download and check the macOS setup runtime ([ce0fe7c](https://github.com/usebach/bach/commit/ce0fe7c2a553de25a167610e04fb468c7d8e5a3a))
- Describe the [macos] settings in the editor schema ([a91baa3](https://github.com/usebach/bach/commit/a91baa37ab5aa63ae3b95bc1e049bef5597fa028))
- Read a [macos] section from bach.toml ([2302436](https://github.com/usebach/bach/commit/23024365d6f7110a2e734d5d543d2771f81b74d0))

All changes: [v0.10...v0.11.0](https://github.com/usebach/bach/compare/v0.10...v0.11.0)

## 0.10.1 (2026-11-21)

### New

- Show every command, grouped, when bach runs without arguments ([de0fe90](https://github.com/usebach/bach/commit/de0fe90192bb0d6d6b5d563ccbe89de3ca828c76))

### Fixes

- Fix the update notice for npm and the terminal that flashed after commands ([8c7d118](https://github.com/usebach/bach/commit/8c7d118bc2e4bf12cd4bfd1f0b0f762841e8fc2c))

### Improvements

- Keep the npm update command simple again ([fc44b7a](https://github.com/usebach/bach/commit/fc44b7a4211033b5904e800d931f0e222321fd26))

All changes: [v0.10...v0.10.1](https://github.com/usebach/bach/compare/v0.10...v0.10.1)

## 0.10.0 (2026-11-21)

### New

- Add the measured results and findings of the installer comparison ([51091b6](https://github.com/usebach/bach/commit/51091b6204ff21cbf825ad1508ee5a45efa2b09d))
- Add a repeatable installer comparison: Bach, Inno Setup, NSIS and WiX on one real app ([16292a4](https://github.com/usebach/bach/commit/16292a439c4ed63aeb99c8d2ea41a1d11fd30be4))
- Add [[services]] and [[firewall]]: Windows services and firewall rules for machine-wide installs, and document them with import, test --here and the new switches ([f6c4b77](https://github.com/usebach/bach/commit/f6c4b7735ac9082164a981c301c6afa1e3bdf877))
- Add compare to the GitHub Action: bach diff against the latest release, in the job summary and one pull request comment ([7f2bb7e](https://github.com/usebach/bach/commit/7f2bb7ef02c8dc3864c7d5fb74e857be05054fb8))
- Add bach import: a first bach.toml from an Inno Setup or NSIS script ([4d43933](https://github.com/usebach/bach/commit/4d43933d29a9599e40641c63b66d0515f44f53ac))
- Add bach test --here: try a setup file on this computer, for CI runners and virtual machines ([0924468](https://github.com/usebach/bach/commit/0924468e67d4b7ca694bd1ea4e019f95de82f1e6))
- Add Home and End to the keyboard, and solid edges and rings in Windows High contrast ([f97bcdc](https://github.com/usebach/bach/commit/f97bcdcd7c328114dc0543acde0a867f15a1fe96))
- Show the install window in English in both themes, and add the screenshot polish script ([4cbb4c1](https://github.com/usebach/bach/commit/4cbb4c1c6a48ae6fcf98353b90b39121aa77b3ee))

### Improvements

- Format the new modules ([0546686](https://github.com/usebach/bach/commit/05466865064d83c7a153ebd573e898c8faa66cd5))
- Mention services, import and test --here in the README and the agent skill ([bf4485c](https://github.com/usebach/bach/commit/bf4485c1191b6b0ef6071e8b83b92f0393333b84))
- Document the compare input of the GitHub Action ([0eda795](https://github.com/usebach/bach/commit/0eda795eb164a59f6c851ed90ccd2f0b138086f2))
- Make the --here guard test ignore line endings ([fa8f8bb](https://github.com/usebach/bach/commit/fa8f8bb21c1abcb4914e1c2d7f3d5a1ff81c8de4))
- Accept Inno Setup, NSIS and Windows Installer silent switches in a setup file ([6e149a0](https://github.com/usebach/bach/commit/6e149a0251d7c7fc9adf6a26ead6b9bef133e1be))
- Rewrite the README: shorter funnel, grouped commands, current features ([74aa381](https://github.com/usebach/bach/commit/74aa381afc3a8fe592be3ec5d54739330c91761a))
- Teach AGENTS.md the docs-site workflow and the screenshot rules ([c8a6356](https://github.com/usebach/bach/commit/c8a63567181a5d7ba6d6d83f47448b57d119c5b3))
- Update: Bump the cargo group with 6 updates ([2143af0](https://github.com/usebach/bach/commit/2143af0daee8c98ded4c492f543262d683a41bf4))
- Update: Bump the actions group with 2 updates ([c21655d](https://github.com/usebach/bach/commit/c21655d5ed30d3ef262ca8140bd37308cffe2515))

All changes: [v0.9...v0.10.0](https://github.com/usebach/bach/compare/v0.9...v0.10.0)

## 0.9.0 (2026-11-20)

### New

- Show Bach's window when an MSI is opened: setup asks the questions and Windows Installer does the work ([6a31e4b](https://github.com/usebach/bach/commit/6a31e4b1dff1d555ac39c77b6406464ec3971e84))
- Add the text for a Windows Installer error ([0e01977](https://github.com/usebach/bach/commit/0e019777c7867664e7303f8b6c70a03a2f16e0d3))
- Add bach build --msi: a Windows Installer package of the app ([e5ead60](https://github.com/usebach/bach/commit/e5ead60c2fed1e009a6b30ed69d0757a8511a659))
- Let a component's files be downloaded when it is ticked: manifest, config and text ([c967137](https://github.com/usebach/bach/commit/c967137c21aacd31c4af8eb92c9961feaee087f4))
- Let the update feed list a signed setup file per architecture, and pick one by machine ([7161cf9](https://github.com/usebach/bach/commit/7161cf93f1c1edcfed9a9f670d9d469d998083aa))
- Add bach test: install, start and remove a setup file in Windows Sandbox ([be2b6fb](https://github.com/usebach/bach/commit/be2b6fb9ae490fa65533976c1ee2357e6250016b))
- Add bach build --profile, --all-profiles and bach profiles, with ZIP builds ([1e05b9d](https://github.com/usebach/bach/commit/1e05b9dd1760f6226bd671ff15b4105d7dd18780))
- Add bach workflow --winget: open the winget update PR after each release ([c3e3080](https://github.com/usebach/bach/commit/c3e3080e1204b57ade804926165ef49b582e6dab))
- Add bach diff: what changed between two setup files ([23d1938](https://github.com/usebach/bach/commit/23d1938efaffea0377a2e8070609e01890b292d1))
- Offer Repair in the installer window when this version is installed ([1b13210](https://github.com/usebach/bach/commit/1b132103810f69186cae44ff5b046d93fb069a88))

### Improvements

- Keep bach-setup the same program alone and built with bach, so the release check passes ([c8ebf0a](https://github.com/usebach/bach/commit/c8ebf0ad91c5684b7a686719c49c3be171658970))
- Format the MSI window code ([188feea](https://github.com/usebach/bach/commit/188feeab4e8d61fdae4a24e02b949297b6f49b2a))
- Document the window of an MSI ([6872fcf](https://github.com/usebach/bach/commit/6872fcf3104da599f9818db6484ba4044fa881dc))
- Put the window into the MSI: its program, the interface sequence and the actions that start it ([d295858](https://github.com/usebach/bach/commit/d2958581f57a7e2ae810712660acedd4559b0e28))
- Make the MSI do what the setup file does: options as features, PATH, variables, autostart, link schemes, file types, right-click entries, run steps ([691d9ca](https://github.com/usebach/bach/commit/691d9ca56367659e9b54f215c3642bad8f4cf8a2))
- Use zip 9 (the 2.x advisories fail cargo deny) and gate Windows-only test helpers ([513bcfb](https://github.com/usebach/bach/commit/513bcfbd9419ef4bf55e07bf251d31f8f4af59d7))
- Put the app icon in the MSI and name its features like the components ([86725d3](https://github.com/usebach/bach/commit/86725d33c4a1c85f1ac0258a085d0a6a20e02902))
- Give bach styles and layouts --json the colours and settings the designer draws ([b350de0](https://github.com/usebach/bach/commit/b350de00ecf3293743aee90889b54bcc0ee76b3f))
- Document Windows Installer packages ([bb1e4f8](https://github.com/usebach/bach/commit/bb1e4f8043e70ebaab38f91475d6fef73b49a256))
- Document binary deltas in update patches ([39cb2c3](https://github.com/usebach/bach/commit/39cb2c39ace5ae1642cedc31681bd228e5d98195))
- Build update patches with binary deltas for changed files, and let verify step over them ([3f9df22](https://github.com/usebach/bach/commit/3f9df22f6f84830e20f9cf5c452bdc026e9ba2b0))
- Expand a binary delta from the installed file when applying an update patch ([bb06193](https://github.com/usebach/bach/commit/bb06193a9ab8506984f27eac6a92e0110391a091))
- Store a changed file as a binary delta from its earlier version in the archive ([91d8b44](https://github.com/usebach/bach/commit/91d8b44b1b1d61a604e65e36ee2db91aaa0b2dc9))
- Document components that are downloaded when ticked ([803df6b](https://github.com/usebach/bach/commit/803df6baeed9653998efbe46fed84a58f55eb707))
- Write each downloaded component to its own .bachpack, and teach verify and inspect about it ([da7f2f0](https://github.com/usebach/bach/commit/da7f2f066ce9bc61dc63a55dddcc409736a7ee37))
- Download a component's archive before installing and read it like the setup file's own ([6e3d6ad](https://github.com/usebach/bach/commit/6e3d6ad632a12b27ee73fb4ff8733c319455f5be))
- Document one download for x64 and ARM64 ([1ee10ac](https://github.com/usebach/bach/commit/1ee10ac446f54abfc13532556c6ec19b8aac2da8))
- Build an ARM64 setup file next to the x64 one and list both in the feed ([3edeea8](https://github.com/usebach/bach/commit/3edeea817f78574aa944074ead00ffd4abe2350a))
- Read [arch.arm64] in bach.toml: the ARM64 build of the app's files ([3e388e8](https://github.com/usebach/bach/commit/3e388e8cd3f8a5f9958cae881c5e1a7382670fc1))
- Document bach test ([e0aad64](https://github.com/usebach/bach/commit/e0aad64eb3575d21198d89d5b639d4ba83cc2eef))
- Document profiles: company versions and portable ZIPs ([b0a4fa5](https://github.com/usebach/bach/commit/b0a4fa580363910ea71a221cc16c50d2174b8c45))
- Read [profiles] in bach.toml and lay a profile over the config ([e16a5e2](https://github.com/usebach/bach/commit/e16a5e2a5c765b113eb56ea1e23bddc24cdeab1a))
- Teach the editor schema the repair_button text ([ec70b0b](https://github.com/usebach/bach/commit/ec70b0b9f8259b77f47d724644693392060ca436))
- Escape every invisible character in bach diff and key entries by position ([13ccb10](https://github.com/usebach/bach/commit/13ccb105e8ba202f2f0c59a93496f122130da4ef))
- Print only safe text in bach diff and compare every field of an entry ([248ab66](https://github.com/usebach/bach/commit/248ab669127674b1fcc9394014c837a5265ee240))

All changes: [v0.8...v0.9.0](https://github.com/usebach/bach/compare/v0.8...v0.9.0)

## 0.8.0 (2026-11-18)

### New

- Add plain rows, and give tiles a full-width folder line and equal heights ([e2aa100](https://github.com/usebach/bach/commit/e2aa1007a7203879b095d2cffa37c191e827558f))
- Add bach layouts: every built-in layout and what it suits ([aecb187](https://github.com/usebach/bach/commit/aecb187a4cc5de38128798aceedb576f284dbdd9))
- Show the window's layout and panel picture in the Look line ([960a33f](https://github.com/usebach/bach/commit/960a33f6c6f6c5a9adcb97778fa3cc3c34df5389))
- Add fade, slide, drop and no page animation, at any speed ([b73fb9c](https://github.com/usebach/bach/commit/b73fb9ca03a93e7dcbb9a0f8f4b8ea34a45392d9))
- Let [style] set the text colour on a side panel ([11b289e](https://github.com/usebach/bach/commit/11b289e3df3d82007721221677b469456166a2d7))
- Let bach preview and bach watch try a style with --style ([6fd7f47](https://github.com/usebach/bach/commit/6fd7f478e6331be05ea5e99043ff88f5622e18fa))
- Add bach styles: every built-in style, what it looks like and suits ([4aca04f](https://github.com/usebach/bach/commit/4aca04f104b6922ec7f5ba0ae252c5c9c6c75217))
- Let a preview try any built-in style with --style ([672cfcb](https://github.com/usebach/bach/commit/672cfcb232336cfcdf26094ff7681ff52a0ecaa3))
- Add fifteen built-in styles for the installer window ([c298e57](https://github.com/usebach/bach/commit/c298e5747f7e6bd2089ea2ff320ba231dce87a8a))

### Improvements

- Start every project in Bach's own style ([2a61184](https://github.com/usebach/bach/commit/2a61184c093576a124b046ba736eaa29f87cf247))
- Document the preview flags of setup files, and what bach watch reloads ([b8f2203](https://github.com/usebach/bach/commit/b8f2203e1b278429549194f4c3131632c066e20a))
- Calm the new layouts: no icon wells, spacing on a 4-point scale ([e1da966](https://github.com/usebach/bach/commit/e1da966aa5e86ee42fe119523217c85897d6490d))
- Replace the compact layout with portrait, and set Inter headings in Inter Display ([5ffb4f3](https://github.com/usebach/bach/commit/5ffb4f3f7695ccb7dd858311fdd6738755c59652))
- Centre the logo on the heading when it sits beside it ([79e8528](https://github.com/usebach/bach/commit/79e8528694bcfdf0704167285fc70961ea6cf8d5))
- Update the README, guide and agent skill for 14 styles and 10 layouts ([55e1763](https://github.com/usebach/bach/commit/55e1763c40b423fbf84db10495b89915e20f4fd9))
- Rework the built-in layouts and styles after the first review ([b45dd0c](https://github.com/usebach/bach/commit/b45dd0c45f22787d137bd1152bc98a10befc52cb))
- Keep a heading on a banner picture readable ([49fd420](https://github.com/usebach/bach/commit/49fd420a3cdd46d72241360b5649c1ec2e787480))
- Document the layout presets and panels, and teach the schema them ([9fcb2f1](https://github.com/usebach/bach/commit/9fcb2f126b26653c2ab81dfced776a8e870b314d))
- Rework [layout] around a panel, with eleven layout presets ([442827e](https://github.com/usebach/bach/commit/442827e7517448bed72b074fa476f62c045de1d1))
- Cut a panel picture to its panel, so rounded corners show ([80d97bb](https://github.com/usebach/bach/commit/80d97bb962a2166997b2337d9f7a4f1f773ac6d3))
- Document [layout] with recipes, and teach the editor schema it ([b0e4f38](https://github.com/usebach/bach/commit/b0e4f38d898efe0863209569b765dac480946bc1))
- Arrange the setup window by [layout] ([da7c8ac](https://github.com/usebach/bach/commit/da7c8ac888a379f13d203610d3f5b4fd4d46790f))
- Read [layout] from bach.toml: size, arrangement, spacing and motion ([093579c](https://github.com/usebach/bach/commit/093579c40e373ffa0009082a5922963914d39126))
- Mention styles in the README, getting started and the agent skill ([5e7927d](https://github.com/usebach/bach/commit/5e7927dea72e551335c7f1cc30e5232ae1846988))
- Document installer.style and [style], and teach the editor schema them ([f314ef0](https://github.com/usebach/bach/commit/f314ef05b328800f705c5faa8cd398695652e753))
- Suggest a style in bach init for the kind of project ([e5b381e](https://github.com/usebach/bach/commit/e5b381ee2d5e716ac90831c055a540d697ee8a2f))
- Recognise Godot and Unity games in bach init ([5777152](https://github.com/usebach/bach/commit/57771526a8f7d8d051a49d75de30675040567238))
- Paint the setup window in the app's style ([880fd6b](https://github.com/usebach/bach/commit/880fd6b8abb26bac84b33db0b44e5c80a2c285b4))
- Ship a style's fonts and picture in the setup file ([6023c3c](https://github.com/usebach/bach/commit/6023c3cb9f35c27785e664e0614e1a0cebad9526))
- Read installer.style and [style] from bach.toml ([0ca2ef4](https://github.com/usebach/bach/commit/0ca2ef4cc3285d76de1e9bdd952d39231fa5b9eb))

All changes: [v0.7...v0.8.0](https://github.com/usebach/bach/compare/v0.7...v0.8.0)

## 0.7.6 (2026-11-18)

### Improvements

- Document what an update keeps, and the journal that stays when a roll-back is stuck ([03945f7](https://github.com/usebach/bach/commit/03945f7657a92a979f666769d30c993b235f0993))
- Never let the install progress bar run backwards ([960ea65](https://github.com/usebach/bach/commit/960ea65dc184dadcc5deb07253c8892fa49e8db8))
- Keep a file the user changed when an update stops shipping it ([98c09f0](https://github.com/usebach/bach/commit/98c09f0c89a4c80a25e3f8c94ad0ae30c5ee2da1))
- Keep the update journal until putting the earlier version back is confirmed ([162a300](https://github.com/usebach/bach/commit/162a300c26a5f0cfad083a6b46afb71479628e21))

All changes: [v0.7...v0.7.6](https://github.com/usebach/bach/compare/v0.7...v0.7.6)

## 0.7.5 (2026-11-17)

No changes since the last release.

All changes: [v0.7...v0.7.5](https://github.com/usebach/bach/compare/v0.7...v0.7.5)

## 0.7.4 (2026-11-17)

### New

- Add the scroll maths: scrollbar thumb and smooth glide ([af674b6](https://github.com/usebach/bach/commit/af674b627c529782ea51553fd981d92e30e7ddd9))

### Improvements

- Check for a newer bach on every command ([49bc146](https://github.com/usebach/bach/commit/49bc146dbef715101d89690633cb37d847afd448))
- Check for a newer bach every hour instead of every day ([a0d0cca](https://github.com/usebach/bach/commit/a0d0cca54e7f4ab0cc37df2259f5f9b9c159917c))
- Format the license page code with rustfmt ([a8d64bc](https://github.com/usebach/bach/commit/a8d64bcc3963c21e10498ad617de2ce474c8a062))
- Document the license page's scrolling and text selection ([3c46581](https://github.com/usebach/bach/commit/3c465816b034f1c92bc0f9ca31ff522baa2e0420))
- Make the license page scroll smoothly, with a scrollbar and selectable text ([95d04da](https://github.com/usebach/bach/commit/95d04da6a0443cc474d9b8cbebbe12f42c576d53))

All changes: [v0.7...v0.7.4](https://github.com/usebach/bach/compare/v0.7...v0.7.4)

## 0.7.3 (2026-11-17)

### New

- Add bach verify and bach page, and --strict for check and build ([157b2e9](https://github.com/usebach/bach/commit/157b2e9987013764be7128ce2a35edbf85a877f2))
- Add bach watch: keep the preview open and reopen it when the config changes ([42f0f88](https://github.com/usebach/bach/commit/42f0f88888ee29037df5f0b739c7da4e4d67dac5))
- Add bach intune: install and uninstall commands, a detection rule and a detection script ([bb988f1](https://github.com/usebach/bach/commit/bb988f125c5c6ec0aa4881c9d1498afc4219e5b1))

### Fixes

- Fix PowerShell quoting so curly single quotes can't end a string early ([20bd8fc](https://github.com/usebach/bach/commit/20bd8fc1445f70f2d54371a57f15ba3070a4a800))

### Improvements

- Refuse Windows paths that leave the project on every system in bach watch ([f0504a8](https://github.com/usebach/bach/commit/f0504a86b09964f450890509521cb0b3b1351181))
- Warn when a signed setup file packs unsigned programs or libraries ([2b5b11c](https://github.com/usebach/bach/commit/2b5b11ca516c07709c2c5d68d6d962b0a2fbd8f4))

All changes: [v0.7...v0.7.3](https://github.com/usebach/bach/compare/v0.7...v0.7.3)

## 0.7.2 (2026-11-17)

### New

- Add bach update and a notice when a newer release is out ([abb52be](https://github.com/usebach/bach/commit/abb52beccf86ad6448faf1a6ef69feff02e73c32))

### Improvements

- Make the update test's installed bach executable on Linux and macOS ([735afb0](https://github.com/usebach/bach/commit/735afb0af96ee028c3a3468847690b65ec74f2cb))
- Document bach update and the update notice ([01096c2](https://github.com/usebach/bach/commit/01096c20fab1da6edcc868482334430f2e6ca815))
- Make agents update the repository before they start ([d899533](https://github.com/usebach/bach/commit/d899533794a3e8afd01694ec33f1b06df4cfb3f7))

All changes: [v0.7...v0.7.2](https://github.com/usebach/bach/compare/v0.7...v0.7.2)

## 0.7.1 (2026-11-17)

### Fixes

- Fix the number of languages in the README, AGENTS.md and bach init's template, and test it (KAN-91) ([cf8a584](https://github.com/usebach/bach/commit/cf8a5841b7cd70944e298e8b669526beb701fba6))
- Fix bach version rewriting a version inside a multi-line TOML string (KAN-96) ([f13149e](https://github.com/usebach/bach/commit/f13149eb1f1f4a61430cce6490d7ea5fdeaa9e8e))

### Improvements

- Warn when the file list is big and explain why its format stays (KAN-93) ([d22f80a](https://github.com/usebach/bach/commit/d22f80a25cea6c02d917efa27a3f403cfda179fd))
- Split the action's args input like a command line so values with spaces work, and document it (KAN-94) ([044aab5](https://github.com/usebach/bach/commit/044aab5448fdf783a3e959b75e93410df12aea2c))
- Check an elevated program against its SHA-256 before asking for rights, warn in bach check, and say what the timeout can and cannot do (KAN-95, KAN-98) ([629d6a8](https://github.com/usebach/bach/commit/629d6a89d6f5d5862109e074bb83ea22da188d8f))
- Refuse tool, browser and profile folders as data folders, warn about unfamiliar ones, and list the real folders on the uninstall page (KAN-88) ([6f3e9d1](https://github.com/usebach/bach/commit/6f3e9d130e3dc725fdfd59e5d213b54f42044552))
- Put shortcuts and integration back when a removal fails, and finish the removal when user data can't be deleted (KAN-85, KAN-87) ([8ae81a7](https://github.com/usebach/bach/commit/8ae81a79061f4c16c258dde7aa4253178df387ba))
- Check [updates] signer on the setup file the online installer downloads (KAN-105) ([1428528](https://github.com/usebach/bach/commit/1428528dd6ded473111b8e375919f10cc1a64de0))
- Sign a publishing time into feed entries with an optional max_age_days, and limit the feed's size and characters (KAN-100, KAN-102) ([1bfdb1d](https://github.com/usebach/bach/commit/1bfdb1d924ff02e6a55aa175973cc49a1f6a505f))
- Keep the installed uninstaller before an update starts, and stop if that fails (KAN-101) ([24038b7](https://github.com/usebach/bach/commit/24038b75e938b2c566970ff5aaca9ebaa0b1d2cf))
- Check every unpacked file against the manifest and refuse a path twice in an archive (KAN-99) ([90afa42](https://github.com/usebach/bach/commit/90afa4250e74222f59de647783e056aca3d16e50))
- Wipe key material with zeroize wherever the key is derived, unwrapped or opened (KAN-107) ([4493d3b](https://github.com/usebach/bach/commit/4493d3b002eee9f3fa7f9fa920e5312a9f51c3f5))
- Warn about paths over the classic Windows limit in bach check and in the setup log (KAN-106) ([0608c0e](https://github.com/usebach/bach/commit/0608c0ef271c2b2e15a6c24c7a95c046c26b272b))
- Take the Tauri program name from mainBinaryName or the crate, not from productName (KAN-104) ([f33821e](https://github.com/usebach/bach/commit/f33821e28d5e44937c4eb49e7f624d0a872e51ea))
- Report signtool in bach doctor only when it is Windows' tool, osslsigncode off Windows (KAN-89) ([cd8055c](https://github.com/usebach/bach/commit/cd8055cff762378332e4e01eb99eccaf10f5d238))
- Warn when app.version is not a version that can be compared (KAN-92) ([38313c4](https://github.com/usebach/bach/commit/38313c4c80f5b71d891dd57bdd5ff469654ac159))
- Say why an app name or id is refused: its length in characters, the character, or the reserved name (KAN-90) ([86d5c48](https://github.com/usebach/bach/commit/86d5c489be58bfbedfe44a7f08dd0b1713d9e0b0))
- Find duplicate file names the way Windows compares them, not with to_lowercase (KAN-103) ([9a701c1](https://github.com/usebach/bach/commit/9a701c179d3013ed879131299afa6ef7dfc29df0))
- Refuse many more credential files and keys or tokens in small text files (KAN-97) ([89681e3](https://github.com/usebach/bach/commit/89681e335b13be0b9495c05714dc2abd8d5f53bd))
- Make cargo test and clippy for the whole workspace pass on Linux and macOS and check it in CI (KAN-86) ([35dfc35](https://github.com/usebach/bach/commit/35dfc35ebfbe7212e3b7c461e6c6f6222e9c34e6))
- Check the ARM64 runtime hash like the x64 build does (KAN-38) ([54d54f7](https://github.com/usebach/bach/commit/54d54f7398fc2fd5c2c32c8d80dcf0ab94561269))

All changes: [v0.7...v0.7.1](https://github.com/usebach/bach/compare/v0.7...v0.7.1)

## 0.7.0 (2026-11-17)

### New

- Add a beta channel: [updates] beta_feed and uninstall.exe --channel (KAN-28) ([64dad7c](https://github.com/usebach/bach/commit/64dad7cbe1ea2543a65f9ea80419be4f6ec03ccd))
- Offer Continue anyway on the requirement page (KAN-76) ([decfc16](https://github.com/usebach/bach/commit/decfc16d0ef75c721310a52ea1e6064a1c785669))
- Add a real Cancel while an install unpacks its files (KAN-75) ([b8ac286](https://github.com/usebach/bach/commit/b8ac28648f62dc990a8eedd0f912153042a6a4d5))
- Add Swedish, Czech, Vietnamese, Indonesian and Hindi (KAN-21) ([40525e1](https://github.com/usebach/bach/commit/40525e1ec19324019bce3147da78132490a63d50))
- Add Traditional Chinese for Taiwan, Hong Kong and Macau; tag key slots only in files with many keys (KAN-62, KAN-61) ([0424d4d](https://github.com/usebach/bach/commit/0424d4d869f30ecd3c87ea78a6f10c6bd8197994))
- Let [[registry]] values go to the 32-bit view, and warn about 32-bit programs in machine-wide installs (KAN-80) ([b702451](https://github.com/usebach/bach/commit/b7024514d22c08ab1d3297c27e1a1a7cab528320))
- Add Save support info to the failed page: one text file without the user's name or any key (KAN-26) ([6c1f311](https://github.com/usebach/bach/commit/6c1f311ef37bc25305a89317916d692b34d6742d))
- Add bach choco to write a Chocolatey package for a setup file (KAN-12) ([51262f0](https://github.com/usebach/bach/commit/51262f04dda6f9e0589f8c570ba54cc16986e672))
- Add an SBOM and signed build provenance to every release file (KAN-19) ([b7dc1c6](https://github.com/usebach/bach/commit/b7dc1c6a080f92e06c4d28c91ce0f29b761e91cf))
- Add --json to check, build, inspect, feed and doctor and let the action read it (KAN-57) ([ab79ef1](https://github.com/usebach/bach/commit/ab79ef17c0c759383a38a7036f54f7bc39bedba5))
- Add --repair to put back missing or changed files of the installed version (KAN-4) ([df8060a](https://github.com/usebach/bach/commit/df8060ae1bb7531e0db15ac4f93d57677a728573))
- Let bach init recognise Flutter, Wails, PyInstaller and Qt projects (KAN-25) ([e973479](https://github.com/usebach/bach/commit/e9734796c4b3a44635156b0959bafaf800b54095))
- Create the feed signing key readable by its owner only on Linux and macOS (KAN-37) ([485baa9](https://github.com/usebach/bach/commit/485baa9f0a4864c4d09bbd5bc3a705b890a24d9d))
- Add an optional config_version to bach.toml and write it in bach init (KAN-24) ([dac3d83](https://github.com/usebach/bach/commit/dac3d833fff6581028dce5d2845c242789b5b358))
- Allow right-click menu texts in Chinese, Japanese, Korean, Cyrillic and Arabic (KAN-70) ([3abff77](https://github.com/usebach/bach/commit/3abff7794f68c5d31bf8283ae0a925c10b34ce72))

### Fixes

- Fix the CI failures: macOS winget test and the signature check on runners ([657dc58](https://github.com/usebach/bach/commit/657dc58944ee3c7bf33e31ca43e755d9113fd59c))
- Fix the [[run]] table in docs/config.md and list [[run]] in the contents (KAN-6) ([a0d88cd](https://github.com/usebach/bach/commit/a0d88cdc3c54806c63c17b5b7394cb9b82f7079d))
- Stop reporting "access denied" as a running program unless a program holds the file (KAN-44) ([1445381](https://github.com/usebach/bach/commit/1445381b1dbff7e2441dc0ea8c42fede0a7d80bc))

### Improvements

- Give the SBOM a serial number so the attest action accepts it (KAN-19) ([63a2b4d](https://github.com/usebach/bach/commit/63a2b4d72e6fa86253c359b994b9778abb18c6f2))
- Build bach-setup the same whether or not bach-cli is built with it (KAN-38) ([7fd1dfc](https://github.com/usebach/bach/commit/7fd1dfc6643c76739e6e5e1eff08f8ca79c7af04))
- End the install.ps1 test with exit 0 so CI doesn't see a leftover exit code (KAN-10) ([4354057](https://github.com/usebach/bach/commit/4354057162bd3996c286cecc6aad6d4cc9318a4b))
- Wrap the footer buttons onto a second row when they don't fit (KAN-76) ([10d36a7](https://github.com/usebach/bach/commit/10d36a7eec896a4ed5aee107542155ea491a7e17))
- Give bach version a --json result (KAN-57) ([036687d](https://github.com/usebach/bach/commit/036687d2a38b0a481e080de5e20a2e7320917688))
- Check the signer of every update setup file, and let signer be a whole certificate subject (KAN-17) ([6ac033a](https://github.com/usebach/bach/commit/6ac033a1669a78a5f52174fcf7ae72206986373e))
- Only install updates signed by the named publisher: [updates] signer, checked with Windows (KAN-17) ([1415f0b](https://github.com/usebach/bach/commit/1415f0bfdf93b37df5d0f59878df0556ac890e1d))
- Give every access key its own slot, so one setup file opens with any of many keys (KAN-61) ([cbff2c6](https://github.com/usebach/bach/commit/cbff2c6feb474833c8730472e09769d2300b432a))
- Trust a second publisher key so the feed key can be changed in two releases (KAN-37) ([2781f01](https://github.com/usebach/bach/commit/2781f01f36a57aa24ff3012b2d27f802b4ede242))
- Keep the registry view import to the test that uses it ([932c506](https://github.com/usebach/bach/commit/932c506cf40f499cb69b6fed93ead0f1d9b3c738))
- Build smaller programs (fat LTO, opt-level s) and fail CI when the setup runtime grows past 9.5 MiB (KAN-27) ([29d87a0](https://github.com/usebach/bach/commit/29d87a005fd7a4f5d82839da25dc7bbe770122a1))
- Test the Tab order of every page and when the window may close; reach Save support info by keyboard (KAN-81, KAN-26) ([b4c6f3b](https://github.com/usebach/bach/commit/b4c6f3b8e6158949737b5d6c2d0e0dad41ec9258))
- Give every shortcut the app's AppUserModelID (KAN-22) ([a458e78](https://github.com/usebach/bach/commit/a458e7878f2b0b3ed9318d1781b545c536e852e2))
- Validate the winget manifests with winget in CI and name the oldest supported Windows (KAN-83) ([f9f8d83](https://github.com/usebach/bach/commit/f9f8d838aeaf6e5858d882d54b6bc76e4808c355))
- Pin Bach's action to a commit in the generated release workflow (KAN-58) ([347e1fe](https://github.com/usebach/bach/commit/347e1fe2d0214a97fff9ec767d9e05c95bbf4b1f))
- Keep compression under a memory budget by limiting threads, with installer.compression_threads and compression_memory (KAN-78) ([561b96b](https://github.com/usebach/bach/commit/561b96bd8ce1dff047f1fb35eab9119bacdfc5c9))
- Test install.sh and install.ps1 in CI against a release made on the spot (KAN-10) ([cb99a17](https://github.com/usebach/bach/commit/cb99a17759b745d04be9328ceaf1d3cc3c8b20fd))
- State the supported Windows versions and warn about a lower windows requirement (KAN-15) ([43a9246](https://github.com/usebach/bach/commit/43a924684f7736b84b64c4c89899bb4bd5bcd77d))
- Test every example and link in the docs (KAN-11) ([d199822](https://github.com/usebach/bach/commit/d1998229f0168c57e15a3572ad3c97b22519a395))
- Open the winget pull request after a normal release when a token is set (KAN-14) ([0facf9c](https://github.com/usebach/bach/commit/0facf9c26ea1d82ace6862a8f101bea381af049d))
- Refuse a version that could leave the output folder in bach choco and bach winget (KAN-12) ([9cec56c](https://github.com/usebach/bach/commit/9cec56c4105387419c6440eb2b56e3af3c496d24))
- Run CI on Windows on ARM and an older Windows image as well (KAN-13) ([2297917](https://github.com/usebach/bach/commit/22979172e84bc3c6e4949bc560ea87775ae2c950))
- Prove builds are reproducible: same bytes whatever the folder, file times or core count (KAN-18) ([858cefa](https://github.com/usebach/bach/commit/858cefaa93cf21f1d6cb1de46cf03dcccb4e0ab5))
- Check the archive's SHA-256 while unpacking instead of reading it twice (KAN-56) ([dffb6da](https://github.com/usebach/bach/commit/dffb6da7d9948c5ae2ea86eec9605b2b68a993bc))
- Read each file once in a build: take the checksums while packing (KAN-50) ([c0dbdef](https://github.com/usebach/bach/commit/c0dbdef8f522329d10e1c7df5e49608457b2eed5))
- Rotate the staged-rollout group from version to version (KAN-51) ([b7412ff](https://github.com/usebach/bach/commit/b7412ffb305ac614877476907557765dbcededc0))
- Say that --repair is for silent runs (KAN-4) ([f2b8d71](https://github.com/usebach/bach/commit/f2b8d71e208d98d17b5fd0ee4b65d22d1efd58e6))
- Escape and check what bach init takes from project files so they can't inject settings (KAN-25) ([a627667](https://github.com/usebach/bach/commit/a6276672a0e8bc4491b6a8f6e34360d7bdb30701))
- Cover registry values, variables, file types, PATH, --delete-data and a Unicode folder in e2e (KAN-79) ([d841cec](https://github.com/usebach/bach/commit/d841cec61733d9fc4c354a765543c94b533ccd87))
- Damage setup files, archives, manifests, feeds, configs and command lines thousands of ways in tests (KAN-63) ([4d21cd4](https://github.com/usebach/bach/commit/4d21cd4e21662a7562bc44edb39e9b3f26612050))
- Update the threat model for signed feeds (KAN-37) ([c922266](https://github.com/usebach/bach/commit/c9222666c4aa1cefd0638eee3c71adb024d8242f))
- Sign update feeds with a publisher key the installed app trusts (KAN-37) ([719fbd9](https://github.com/usebach/bach/commit/719fbd996e134e9b30fa859daf405f7cabc91ce7))
- Check the signature after signtool ran and warn about a missing timestamp (KAN-59) ([083e477](https://github.com/usebach/bach/commit/083e477bcae8cfbb5037a368ec8f69ba14a78fa0))
- Test that docs/config.md and the editor schema name the same settings (KAN-9) ([5fbc3eb](https://github.com/usebach/bach/commit/5fbc3eb12ff5a20e74fd3b7f1566cd9f1ac797e9))
- Format the code with rustfmt and check the formatting in CI (KAN-1) ([75976ae](https://github.com/usebach/bach/commit/75976ae50c72ea7b794961b7e107d3c3130faa51))
- Check dependencies with cargo-deny and test bach on Linux and macOS in CI (KAN-66, KAN-3) ([f9486e3](https://github.com/usebach/bach/commit/f9486e3e858ce805e9bec9419bc0e5c1ef102d84))
- Name the setup exit codes in one place, pin them with a test and list them in the docs (KAN-7) ([f14d57c](https://github.com/usebach/bach/commit/f14d57ca5fd185f182e9611b2f0425b078997502))
- Detect more .NET, Visual C++ and registry installs, and let a silent run skip requirements (KAN-76) ([3a0c53a](https://github.com/usebach/bach/commit/3a0c53a169aacf0848b96dd993a2e41763ebf692))
- Document the threat model: what Bach trusts, protects and doesn't yet (KAN-65) ([924ad07](https://github.com/usebach/bach/commit/924ad073c45b97b6b729d422083087db27bc2ff8))
- Keep winget package ids to 32 characters per part and quote the links (KAN-83) ([4e5af26](https://github.com/usebach/bach/commit/4e5af26e43cc60d4fae3b4ac50eeb84521136f59))
- Say that Bun is for the repository's tooling and npm publishes the npm package (KAN-8) ([9d41502](https://github.com/usebach/bach/commit/9d4150249f3cedff95adb01855f890fb5089c47a))
- Delete the setup file after exit without putting its path into a cmd line (KAN-52) ([ef087f1](https://github.com/usebach/bach/commit/ef087f1dbc8d1293e5392dc13d14315fb8479b96))
- Match exclude patterns in linear time and check the patch base in parallel (KAN-53) ([964a298](https://github.com/usebach/bach/commit/964a2980316013726a8e72f019871342a9526970))
- Skip files an update would only write again unchanged (KAN-77) ([c48ddfe](https://github.com/usebach/bach/commit/c48ddfe31d712e404fedf3041ef03ce94b60b31a))
- Hash setup files in pieces in bach feed and bach winget instead of reading them whole (KAN-55) ([8ffe3eb](https://github.com/usebach/bach/commit/8ffe3ebeb50c792d5818a39af30a35d5bdd553b3))
- Keep the update patch in the feed, sign it and upload it in the GitHub Action (KAN-54) ([f454f69](https://github.com/usebach/bach/commit/f454f695b890c960d00aac16f08ecb373695c06b))
- Document antivirus false positives and what helps (KAN-84) ([1fbd0d2](https://github.com/usebach/bach/commit/1fbd0d27bef0ae69a21816ba301b0619a3ceafac))
- Make removal all or nothing: set every file aside first, delete after (KAN-46) ([9f93450](https://github.com/usebach/bach/commit/9f9345056f524f08713390d10cdbc08aac24714d))
- Treat a locked file as unchanged, not as edited by the user (KAN-73) ([915b8ee](https://github.com/usebach/bach/commit/915b8ee2555f33277e1f5a246a644464a8d858c5))
- Roll an update back whole when a step after the file swap fails (KAN-47) ([c4f8b98](https://github.com/usebach/bach/commit/c4f8b981018d7cbc4e12c11a44660ff01cf18764))
- Treat unreadable owners and entries as weak and check the folders above the install folder (KAN-69) ([33aa4be](https://github.com/usebach/bach/commit/33aa4be2ddd0344e4bbbaddd73693788432ba02f))
- Log what run steps print, end the whole process tree on a timeout, and add timeout_minutes (KAN-45) ([6570238](https://github.com/usebach/bach/commit/65702386939171ea218928845787105dcc4f79d0))
- Make the e2e script say when Defender blocks a freshly built setup file (KAN-84) ([f52cd44](https://github.com/usebach/bach/commit/f52cd44a0e7317e14ac9cda314ef091de7ca4978))
- Remove the empty extension keys a file type leaves behind (KAN-72) ([2694cc6](https://github.com/usebach/bach/commit/2694cc67c961f07fbcb21fcad1e23364a053fdcd))
- Make bach init write a valid app name for scoped and unusual package names (KAN-74) ([b2b97ab](https://github.com/usebach/bach/commit/b2b97abb469b546012bc164cd18b9080a09711e9))
- Refuse device names as app ids and folders and names longer than 255 characters (KAN-71, KAN-42) ([2df80d3](https://github.com/usebach/bach/commit/2df80d323f8a803eb119bda5b853d33f52f6bba4))
- Read curl's HTTP status instead of digits in its error text (KAN-49) ([a55a527](https://github.com/usebach/bach/commit/a55a527970f5d32e77268947ec9a48e52be29da0))
- Give app ids with a space their own scratch folders instead of sharing them with dashed ids (KAN-48) ([132ae04](https://github.com/usebach/bach/commit/132ae047b955110c512cf1e7c722768686866005))
- Read the setup command line strictly: unknown flags, missing values and --flag=value (KAN-43) ([96f1bea](https://github.com/usebach/bach/commit/96f1beafa5dde68a2447fa7ff11f6e65f6d61d63))
- Fail closed in the elevated-folder check and look at every file (KAN-69) ([2ff08fe](https://github.com/usebach/bach/commit/2ff08fef353c44e0214aafcc6ebd17d485583761))
- Check the whole install tree and trust only administrators in the elevated-folder check (KAN-69) ([e0e8570](https://github.com/usebach/bach/commit/e0e8570c67fd6293894fec2373ef58fadab25772))
- Check a machine-wide install folder's access list before running anything elevated (KAN-69) ([531dce1](https://github.com/usebach/bach/commit/531dce16c923ab555076ff5bc3d781ab21251896))
- Refuse whole vendors and code-injection variables in [[registry]] and [[env]] (KAN-68) ([7bada94](https://github.com/usebach/bach/commit/7bada943e9d24c5278f329a9f1e2aa3d0ad7d9a9))
- Harden the runtime download: https only, a hash built into bach and a private cache (KAN-38, KAN-39) ([c325d71](https://github.com/usebach/bach/commit/c325d710c84fb3d3be251f54b579c9cbbdd22cde))
- Refuse to package secrets by accident and warn about build clutter (KAN-67) ([11a763c](https://github.com/usebach/bach/commit/11a763c27ff37b4877f172a1bdeb4dfd3e62feda))
- Document --overwrite in the agent skill (KAN-73) ([5a08a39](https://github.com/usebach/bach/commit/5a08a39d5814dac6206d6d560121ca554eafe254))
- Warn before a first install replaces files that aren't the app's, and keep changed files on removal (KAN-73) ([6406f0d](https://github.com/usebach/bach/commit/6406f0dd156aaf1564c2689d2cbbaf0d3d0f55a1))
- Refuse data folders that are profile folders or hold the install folder (KAN-40) ([42740ae](https://github.com/usebach/bach/commit/42740aee8f890747331d918a621057ceffc050b7))

All changes: [v0.6...v0.7.0](https://github.com/usebach/bach/compare/v0.6...v0.7.0)

## 0.6.0 (2026-11-15)

### New

- Add update patches: bach build --patch-from and bach feed --patch ([b36c11c](https://github.com/usebach/bach/commit/b36c11c0e176cba40237f966bb35fa8f4deaea09))
- Add [[migrate]]: remove an older install made by another installer ([48f52cf](https://github.com/usebach/bach/commit/48f52cf2b82c60ae7ef5f2571ae2e68d8bf5b6ab))
- Add machine-wide installs and elevated run steps ([0c26f4b](https://github.com/usebach/bach/commit/0c26f4b9b790bf6e3fa017659fd781ba80f160b6))
- Add optional signing: [signing], bach sign and bach build --no-sign ([2e07440](https://github.com/usebach/bach/commit/2e074404d71f409625e6287b2deea58e3f002291))
- Add registry values, environment variables and right-click menu entries ([2373f56](https://github.com/usebach/bach/commit/2373f567202842f434c5abef97dba7b45e9e6250))

### Fixes

- Fix the commands in the repo docs for PowerShell, cmd and signtool ([d01c9c9](https://github.com/usebach/bach/commit/d01c9c9d063e33aec2e42ff51518d8d83e4d3cc7))
- Fix winget scope and architecture and the registry hive shown by inspect ([c825644](https://github.com/usebach/bach/commit/c82564477c2a324c2a78c94623f79922dd1ff945))
- Fix clippy findings in the elevation and registry changes code ([c7e8269](https://github.com/usebach/bach/commit/c7e8269336a7126beeae96c252457c60bb058fc5))

### Improvements

- Make key files and signing work from any shell ([2080948](https://github.com/usebach/bach/commit/208094819fe9f2c3d8e487a912c88ebb8f86a5c2))
- Hash files with the sha2 0.11 helpers in the patch code ([75a2ba1](https://github.com/usebach/bach/commit/75a2ba1cd07031c6b1248680ad8887e02b042c0c))
- Keep machine-wide installs inside Program Files ([3cfce7d](https://github.com/usebach/bach/commit/3cfce7dabe2b03cccd5f89cb88dfec5babe9836e))
- Update the safety rules for machine-wide installs, elevated steps and migration ([2ca2639](https://github.com/usebach/bach/commit/2ca263964325a479c2318dd6d25d4743054903ba))
- Document building a setup file for Windows on ARM with the ARM64 runtime ([4a1fb52](https://github.com/usebach/bach/commit/4a1fb52dde3ae246144f2f25feb0300b3e580a5b))
- Apply update patches, and fall back on the full setup file ([b2f045b](https://github.com/usebach/bach/commit/b2f045b9da65de0fb1213ba1c39b95cc02b09e59))
- Limit what [[migrate]] may run with administrator rights ([aee939b](https://github.com/usebach/bach/commit/aee939be7b5a34bbc75c8d9060b9e70bc8685234))
- Harden machine-wide setup: remove in place, load DLLs from System32 only ([75faf09](https://github.com/usebach/bach/commit/75faf097b04b65992e3352b5eabf4c1e6b256281))
- Update: Bump the cargo group across 1 directory with 9 updates (#13) ([6696993](https://github.com/usebach/bach/commit/669699365e323ecf0a99f49256c0618701e82c93))
- Make signing opt-in: only bach build --sign or BACH_SIGN=1 runs the signer ([4a02937](https://github.com/usebach/bach/commit/4a029373b233db14b1fae662b0b77ce77f298786))

All changes: [v0.5...v0.6.0](https://github.com/usebach/bach/compare/v0.5...v0.6.0)

## 0.5.0 (2026-11-14)

### New

- Add run steps, a Windows version requirement and --help for setup files ([f47decb](https://github.com/usebach/bach/commit/f47decbdfbea01243c16ee9b6c92dc82eae22c93))
- Add access keys: encrypted setup files, private updates and staged rollouts ([bd9b4c7](https://github.com/usebach/bach/commit/bd9b4c7e48857fbae46da70abda7e49d998d9103))
- Add the online installer: a small setup that downloads the newest version ([a6bd5ed](https://github.com/usebach/bach/commit/a6bd5eda4ce086b3998733276493b9480dad7d21))

### Improvements

- Send the access key only to the host of the main update feed ([826e769](https://github.com/usebach/bach/commit/826e7692c3659b3a22b8b3dbf52d3fe00b035bdd))
- Document online installers, access keys, run steps, mirrors and rollouts; publish the online installer with the action ([2052b0e](https://github.com/usebach/bach/commit/2052b0ec50e9c844f8ce74586e5bdb928becf4be))
- Cover protected apps and run steps in the end-to-end script ([e3ca3e6](https://github.com/usebach/bach/commit/e3ca3e65bca26ec43e68ee7908957c128a9c5a46))
- Describe run steps and the Windows requirement in the editor schema ([7c8e9c9](https://github.com/usebach/bach/commit/7c8e9c974a5fa83ef4517df7170dd7f598a02e08))
- Remove the version note from the README ([35a8613](https://github.com/usebach/bach/commit/35a8613274caf3d093d943a6696067b200a360e7))

All changes: [v0.4...v0.5.0](https://github.com/usebach/bach/compare/v0.4...v0.5.0)

## 0.4.0 (2026-11-14)

### Improvements

- Print a ready-to-paste PATH line when install.sh can't put bach on PATH ([bdec807](https://github.com/usebach/bach/commit/bdec80715569ab5184b5aa5567a323f1c53984ed))
- Write a real README for the npm packages ([aca09dd](https://github.com/usebach/bach/commit/aca09dd632e388b9c953ab16b5a48f262ef5b771))

All changes: [v0.3...v0.4.0](https://github.com/usebach/bach/compare/v0.3...v0.4.0)

## 0.3.0 (2026-11-14)

### New

- Let the install scripts install for every user, and only when asked ([c4d599c](https://github.com/usebach/bach/commit/c4d599c0f4e3930995772dc73edefa029357149f))
- Add the @usebach/cli npm package and the script that builds it ([b38fd69](https://github.com/usebach/bach/commit/b38fd69efbd618bba6d7084993a416d9aa46b233))
- Add bach version and bach workflow, a one-click GitHub release ([a8ebfe6](https://github.com/usebach/bach/commit/a8ebfe64098cf45e2fef534c617105cc7e37a5f4))

### Fixes

- Fix the CI step that tests the npm package on Node 22 ([835a812](https://github.com/usebach/bach/commit/835a81277fa0229c120d80ad35aead3c55b6925c))

### Improvements

- Hand the elevated install the script's text, not a file to run ([9e87082](https://github.com/usebach/bach/commit/9e870824c8bde577526cc36706da65762e74850f))
- Document one-click releases, the npm install and system-wide installs ([7e03efa](https://github.com/usebach/bach/commit/7e03efadc36fb59cb72aca635bcfe8c6acebfdb2))
- Publish the npm packages from the release workflow ([e2dc145](https://github.com/usebach/bach/commit/e2dc145a91ffb86a6cda9b867a32cfaeaca09575))

All changes: [v0.2...v0.3.0](https://github.com/usebach/bach/compare/v0.2...v0.3.0)

## 0.2.1 (2026-11-14)

### Improvements

- Update action name to 'Bach Installer' ([d4b63d1](https://github.com/usebach/bach/commit/d4b63d1334bd661bfd8b7cc12ff0597dbab46561))

All changes: [v0.2...v0.2.1](https://github.com/usebach/bach/compare/v0.2...v0.2.1)

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

