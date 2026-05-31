# rust-dedupe

A fast, std-only recursive duplicate-file finder. Point it at a folder and it tells you exactly how much space byte-identical copies are wasting, ranked biggest-win first.

This npm package is a thin launcher: on install it downloads the prebuilt native binary (built from the [Rust source](https://github.com/SujalXplores/rust-dedupe)) for your platform. **No Rust toolchain required.**

## Use it without installing

```bash
npx rust-dedupe ~/Downloads
npx rust-dedupe .
npx rust-dedupe "C:\Users\you\Pictures"
```

## Or install it globally

```bash
npm install -g rust-dedupe
rust-dedupe ~/Downloads
```

Supported platforms: Windows x64, macOS (Intel + Apple Silicon), Linux x64. On anything else, install from source: `cargo install --git https://github.com/SujalXplores/rust-dedupe.git`.

Source, algorithm, and license: https://github.com/SujalXplores/rust-dedupe
