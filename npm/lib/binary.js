"use strict";

// Resolves the right prebuilt binary for the host, and downloads it from the
// matching GitHub Release on demand. Used by both the postinstall step and the
// launcher (so `npx --ignore-scripts` still works — the launcher lazily fetches).

const fs = require("fs");
const path = require("path");
const https = require("https");
const { version } = require("../package.json");

const REPO = "SujalXplores/rust-dedupe";

// Node platform+arch  ->  release asset name produced by .github/workflows/release.yml
const ASSETS = {
  "win32 x64": "rust-dedupe-win32-x64.exe",
  "darwin x64": "rust-dedupe-darwin-x64",
  "darwin arm64": "rust-dedupe-darwin-arm64",
  "linux x64": "rust-dedupe-linux-x64",
};

function assetName() {
  const key = `${process.platform} ${process.arch}`;
  const name = ASSETS[key];
  if (!name) {
    throw new Error(
      `no prebuilt binary for ${key}. ` +
        `Build from source instead:\n  cargo install --git https://github.com/${REPO}.git`,
    );
  }
  return name;
}

function binaryPath() {
  const ext = process.platform === "win32" ? ".exe" : "";
  return path.join(__dirname, "..", "bin", `rust-dedupe${ext}`);
}

function downloadTo(url, dest) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "rust-dedupe-npm-installer" } }, (res) => {
        // GitHub release downloads redirect to a storage host.
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          res.resume();
          return resolve(downloadTo(res.headers.location, dest));
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error(`download failed (HTTP ${res.statusCode}) for ${url}`));
        }
        const file = fs.createWriteStream(dest);
        res.pipe(file);
        file.on("finish", () => file.close((err) => (err ? reject(err) : resolve())));
        file.on("error", reject);
      })
      .on("error", reject);
  });
}

// Returns the path to a ready-to-run binary, downloading it if missing.
async function ensureBinary() {
  const dest = binaryPath();
  if (fs.existsSync(dest)) return dest;

  const url = `https://github.com/${REPO}/releases/download/v${version}/${assetName()}`;
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  await downloadTo(url, dest);
  if (process.platform !== "win32") fs.chmodSync(dest, 0o755);
  return dest;
}

module.exports = { assetName, binaryPath, ensureBinary };
