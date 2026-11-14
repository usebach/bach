"use strict";

// Finds the native `bach` program that belongs to this computer. It comes in
// a package of its own (@usebach/cli-<platform>-<cpu>), which npm installs
// next to this one because that package's `os` and `cpu` match; the others
// are skipped. No install script and no download happen, so this works with
// --ignore-scripts, behind proxies and from an offline cache.

const fs = require("node:fs");
const path = require("node:path");

/** The platforms a native package is published for. */
const PLATFORMS = ["darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64", "win32-arm64", "win32-x64"];

function platformKey(platform = process.platform, arch = process.arch) {
  return `${platform}-${arch}`;
}

function packageName(key) {
  return `@usebach/cli-${key}`;
}

/**
 * The path of the native bach, or an Error that says how to get one.
 * `env`, `platform`, `arch` and `resolve` can be replaced in tests.
 */
function findBinary({
  env = process.env,
  platform = process.platform,
  arch = process.arch,
  resolve = require.resolve,
} = {}) {
  const given = env.BACH_BINARY_PATH;
  if (given) {
    if (!fs.existsSync(given)) {
      return new Error(`BACH_BINARY_PATH is set to ${given}, but there is no such file.`);
    }
    return given;
  }

  const key = platformKey(platform, arch);
  if (!PLATFORMS.includes(key)) {
    return new Error(
      `There is no prebuilt bach for ${key}. Build it with: cargo install --git https://github.com/usebach/bach bach-cli`,
    );
  }
  const name = packageName(key);
  let manifest;
  try {
    manifest = resolve(`${name}/package.json`);
  } catch {
    return new Error(
      `The bach program for ${key} (${name}) isn't installed. It is an optional dependency, so it is missing when ` +
        `npm ran with --omit=optional or --no-optional. Install again without that flag: npm install -g @usebach/cli`,
    );
  }
  const file = path.join(path.dirname(manifest), "bin", platform === "win32" ? "bach.exe" : "bach");
  if (!fs.existsSync(file)) {
    return new Error(`${name} is installed, but ${file} is missing. Reinstall it: npm install -g @usebach/cli`);
  }
  if (platform !== "win32") {
    try {
      fs.accessSync(file, fs.constants.X_OK);
    } catch {
      // Some tools unpack without the executable bit.
      try {
        fs.chmodSync(file, 0o755);
      } catch {
        return new Error(`${file} isn't executable and couldn't be made so. Reinstall: npm install -g @usebach/cli`);
      }
    }
  }
  return file;
}

module.exports = { PLATFORMS, platformKey, packageName, findBinary };
