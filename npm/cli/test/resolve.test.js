"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { findBinary, packageName, platformKey, PLATFORMS } = require("../lib/resolve.js");

function fakePackage(platform) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "bach-npm-"));
  fs.mkdirSync(path.join(dir, "bin"));
  fs.writeFileSync(path.join(dir, "package.json"), "{}");
  const file = path.join(dir, "bin", platform === "win32" ? "bach.exe" : "bach");
  fs.writeFileSync(file, "x");
  return { dir, file };
}

test("every platform has its own package name", () => {
  assert.equal(packageName(platformKey("win32", "x64")), "@usebach/cli-win32-x64");
  assert.equal(PLATFORMS.length, 6);
});

test("the optional dependencies of the package are exactly the platforms", () => {
  const manifest = require("../package.json");
  assert.deepEqual(Object.keys(manifest.optionalDependencies).sort(), PLATFORMS.map(packageName).sort());
  for (const version of Object.values(manifest.optionalDependencies)) {
    assert.equal(version, manifest.version, "native packages are pinned to this exact version");
  }
});

test("the installed native package is found", () => {
  const { dir, file } = fakePackage("linux");
  const found = findBinary({ env: {}, platform: "linux", arch: "x64", resolve: () => path.join(dir, "package.json") });
  assert.equal(found, file);
  fs.rmSync(dir, { recursive: true });
});

test("a missing native package says how to fix it", () => {
  const found = findBinary({
    env: {},
    platform: "darwin",
    arch: "arm64",
    resolve: () => {
      throw new Error("not found");
    },
  });
  assert.ok(found instanceof Error);
  assert.match(found.message, /--omit=optional/);
  assert.match(found.message, /@usebach\/cli-darwin-arm64/);
});

test("an unsupported computer is named", () => {
  const found = findBinary({ env: {}, platform: "freebsd", arch: "x64" });
  assert.ok(found instanceof Error);
  assert.match(found.message, /freebsd-x64/);
});

test("BACH_BINARY_PATH wins, and must exist", () => {
  const { dir, file } = fakePackage("win32");
  assert.equal(findBinary({ env: { BACH_BINARY_PATH: file } }), file);
  assert.ok(findBinary({ env: { BACH_BINARY_PATH: path.join(dir, "nope") } }) instanceof Error);
  fs.rmSync(dir, { recursive: true });
});

test("the command reports a missing program with exit code 1", () => {
  const script = path.join(__dirname, "..", "bin", "bach.js");
  const run = spawnSync(process.execPath, [script, "--version"], {
    env: { ...process.env, BACH_BINARY_PATH: path.join(os.tmpdir(), "bach-does-not-exist") },
    encoding: "utf8",
  });
  assert.equal(run.status, 1);
  assert.match(run.stderr, /BACH_BINARY_PATH/);
});
