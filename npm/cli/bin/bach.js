#!/usr/bin/env node
"use strict";

// The `bach` command: hands everything to the native program, unchanged.

const { spawnSync } = require("node:child_process");
const { findBinary } = require("../lib/resolve.js");

const binary = findBinary();
if (binary instanceof Error) {
  console.error(`error: ${binary.message}`);
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit", windowsHide: true });
if (result.error) {
  console.error(`error: couldn't start ${binary}: ${result.error.message}`);
  process.exit(1);
}
if (result.signal) {
  // Ctrl+C and the like: end the same way bach did.
  process.kill(process.pid, result.signal);
} else {
  process.exit(result.status ?? 1);
}
