## What changed and why

<!-- One or two sentences. Link the issue it closes: "Closes #123". -->

## How I checked it

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Looked at changed UI with `--preview`, in light and dark
- [ ] Docs, `docs/config.md` and the schema updated, if a setting or command changed
- [ ] Any new window text is in every language

<!-- Say what you couldn't check, e.g. "not tried on Windows ARM64". -->

## Safety

<!-- Does this touch paths.rs, work/, launch.rs, the payload format or the
update feed? If so, say what stops a bad setup file from doing harm, and
which test covers it. Otherwise write "No". -->
