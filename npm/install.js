"use strict";

// postinstall: fetch the prebuilt binary up front so the first run is instant.
// Failures here are non-fatal — the launcher retries lazily on first use, so a
// transient network hiccup (or `--ignore-scripts`) doesn't break installation.

const { ensureBinary } = require("./lib/binary");

ensureBinary()
  .then(() => {
    console.log(`rust-dedupe: ready for ${process.platform}-${process.arch}`);
  })
  .catch((err) => {
    console.warn(`rust-dedupe: deferred binary download (${err.message})`);
  });
