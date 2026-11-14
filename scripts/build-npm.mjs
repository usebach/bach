// Builds the npm packages of the bach command from a release's programs:
// @usebach/cli (the `bach` shim in npm/cli) and one @usebach/cli-<platform>
// package per computer type that holds the native program.
//
//   bun scripts/build-npm.mjs --version 0.2.1 --binaries dist/bin --out dist/npm
//   bun scripts/build-npm.mjs --version 0.2.1 --binaries dist/bin --only win32-x64
//
// --binaries is a folder with one subfolder per platform (win32-x64,
// linux-arm64, ...), each holding `bach` (`bach.exe` on Windows); Windows
// ones also hold bach-setup.exe, which `bach build` looks for next to bach.
// Elsewhere the runtime is downloaded the first time it's needed.
//
// The packages are written to <out>/<name>, and packed into <out>/tarballs
// with `npm pack`, which is also what publishes them (provenance needs
// npm's own client). Without --only every platform must be present, so a
// release can't go out missing one.

import { chmodSync, copyFileSync, cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { join, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const PLATFORMS = ["darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64", "win32-arm64", "win32-x64"];

const args = process.argv.slice(2);
function option(name) {
  const at = args.indexOf(`--${name}`);
  return at === -1 ? undefined : args[at + 1];
}
function fail(message) {
  console.error(message);
  process.exit(1);
}

const version = option("version")?.replace(/^v/, "");
if (!version || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) fail("Give the release's version: --version 0.2.1");
const binaries = resolve(option("binaries") ?? join(root, "dist", "bin"));
const out = resolve(option("out") ?? join(root, "dist", "npm"));
const only = option("only")?.split(",");
const platforms = only ?? PLATFORMS;
for (const key of platforms) if (!PLATFORMS.includes(key)) fail(`${key} isn't a platform. They are ${PLATFORMS.join(", ")}.`);

const manifest = JSON.parse(readFileSync(join(root, "npm", "cli", "package.json"), "utf8"));
const base = {
  license: manifest.license,
  homepage: manifest.homepage,
  bugs: manifest.bugs,
  repository: { type: "git", url: manifest.repository.url, directory: "npm/cli" },
  publishConfig: manifest.publishConfig,
};

rmSync(out, { recursive: true, force: true });
mkdirSync(join(out, "tarballs"), { recursive: true });

// One package per platform, holding the program.
for (const key of platforms) {
  const [os, cpu] = key.split("-");
  const windows = os === "win32";
  const from = join(binaries, key);
  const program = windows ? "bach.exe" : "bach";
  if (!existsSync(join(from, program))) fail(`${join(from, program)} is missing.`);
  if (windows && !existsSync(join(from, "bach-setup.exe"))) fail(`${join(from, "bach-setup.exe")} is missing.`);
  const dir = join(out, `cli-${key}`);
  mkdirSync(join(dir, "bin"), { recursive: true });
  copyFileSync(join(from, program), join(dir, "bin", program));
  // npm keeps the executable bit it finds. (A Windows machine can't set it, so
  // build the release on Linux; the shim also fixes it at run time.)
  if (!windows) chmodSync(join(dir, "bin", program), 0o755);
  if (windows) copyFileSync(join(from, "bach-setup.exe"), join(dir, "bin", "bach-setup.exe"));
  copyFileSync(join(root, "LICENSE"), join(dir, "LICENSE"));
  const native = {
    name: `@usebach/cli-${key}`,
    version,
    description: `The bach command for ${key}. Installed by @usebach/cli; install that instead.`,
    ...base,
    os: [os],
    cpu: [cpu],
    files: ["bin", "README.md", "LICENSE"],
  };
  writeFileSync(join(dir, "package.json"), `${JSON.stringify(native, null, 2)}\n`);
  const readme = [
    `# @usebach/cli-${key}`,
    "",
    `The bach program for ${key}. **You don't install this package yourself:** it is pulled in by`,
    "[@usebach/cli](https://www.npmjs.com/package/@usebach/cli), which picks the one that matches your computer.",
    "",
    "```bash",
    "npm install --global @usebach/cli",
    "```",
    "",
    "Docs: https://bach.mintlify.app/docs - Source: https://github.com/usebach/bach",
    "",
  ].join("\n");
  writeFileSync(join(dir, "README.md"), readme);
}

// The package people install, pinned to exactly these native packages.
const main = join(out, "cli");
cpSync(join(root, "npm", "cli"), main, { recursive: true, filter: (path) => !/[\\/]test([\\/]|$)/.test(path) });
copyFileSync(join(root, "LICENSE"), join(main, "LICENSE"));
manifest.version = version;
manifest.optionalDependencies = Object.fromEntries(platforms.map((key) => [`@usebach/cli-${key}`, version]));
writeFileSync(join(main, "package.json"), `${JSON.stringify(manifest, null, 2)}\n`);

for (const name of [...platforms.map((key) => `cli-${key}`), "cli"]) {
  execFileSync("npm", ["pack", join(out, name), "--pack-destination", join(out, "tarballs"), "--silent"], {
    stdio: ["ignore", "inherit", "inherit"],
    shell: process.platform === "win32",
  });
}
console.log(`Packed ${platforms.length + 1} packages for ${version} into ${join(out, "tarballs")}`);
