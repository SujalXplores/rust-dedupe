#!/usr/bin/env node
"use strict";

// Thin launcher: make sure the native binary is present, then exec it with the
// user's arguments and forward its exit code.

const { spawnSync } = require("child_process");
const { binaryPath, ensureBinary } = require("../lib/binary");

(async () => {
  try {
    await ensureBinary();
  } catch (err) {
    console.error(`rust-dedupe: ${err.message}`);
    process.exit(1);
  }

  const res = spawnSync(binaryPath(), process.argv.slice(2), { stdio: "inherit" });
  if (res.error) {
    console.error(`rust-dedupe: ${res.error.message}`);
    process.exit(1);
  }
  process.exit(res.status === null ? 1 : res.status);
})();
