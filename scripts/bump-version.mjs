// Raises Bach's version in Cargo.toml ([workspace.package] version) and
// prints the new version, nothing else, so a workflow can capture it.
//
//   bun scripts/bump-version.mjs patch|minor|major
//   bun scripts/bump-version.mjs 1.0.0
//   bun scripts/bump-version.mjs minor --dry-run   # print only, don't write
//
// An exact version may be the current one (to release it as it is, like the
// very first release) or higher, never lower; the release workflow refuses a
// version that already has a tag. Line endings and everything else in
// Cargo.toml stay as they are.

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const manifest = join(import.meta.dir, "..", "Cargo.toml");
const args = process.argv.slice(2);
const target = args.find((arg) => !arg.startsWith("--"));
const dryRun = args.includes("--dry-run");

function fail(message) {
  console.error(message);
  process.exit(1);
}

function parse(version) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  return match ? match.slice(1, 4).map(Number) : null;
}

function isLower(next, current) {
  for (let i = 0; i < 3; i++) {
    if (next[i] !== current[i]) return next[i] < current[i];
  }
  return false;
}

if (!target) fail("Say how to raise the version: patch, minor, major, or an exact version like 1.0.0.");

const text = readFileSync(manifest, "utf8");
// The version line inside [workspace.package], up to the next table.
const section = /(\[workspace\.package\][^[]*?\bversion\s*=\s*")(\d+\.\d+\.\d+)(")/;
const found = section.exec(text);
if (!found) fail("Couldn't find the version under [workspace.package] in Cargo.toml.");

const current = parse(found[2]);
let next;
if (target === "patch") next = [current[0], current[1], current[2] + 1];
else if (target === "minor") next = [current[0], current[1] + 1, 0];
else if (target === "major") next = [current[0] + 1, 0, 0];
else {
  next = parse(target.replace(/^v/, ""));
  if (!next) fail(`"${target}" isn't a version. Use patch, minor, major, or something like 1.0.0.`);
  if (isLower(next, current)) fail(`${next.join(".")} is lower than the current version, ${found[2]}.`);
}

const version = next.join(".");
if (!dryRun) {
  writeFileSync(manifest, text.replace(section, `$1${version}$3`));
}
console.log(version);
