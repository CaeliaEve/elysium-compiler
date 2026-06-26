# Elysium Compiler

`elysium-compiler` is the native Rust compiler for NESQL++ Raw Export data. It produces NeoNEI/Elysium `dist-data` runtime packs, Native UI layout assets, recipe UI payload indexes, texture packs, search indexes, and validation reports.

This repository was extracted from `NeoNEI/tools/neonei-compiler-rs` after the in-repo compiler split reached extraction readiness.

## Stable CLI

```bash
elysium-compiler inspect  --input <raw-export> --report <report.json>
elysium-compiler validate --input <raw-export> --report <report.json> [--output <dist-data>]
elysium-compiler schemas  [--output <schema-catalog.json>]
elysium-compiler compile  --input <raw-export> --output <dist-data> --report <report.json> [--scope all|native-ui|search|browser|recipes|ui|textures] [--strict]
```

The retired `baseline` command is intentionally not supported.

## Repository layout

```text
crates/elysium-compiler-core/  # compiler library, schemas, packs, fixtures, tests
crates/elysium-compiler-cli/   # thin binary entrypoint
schemas/                       # exported schema artifacts
scripts/                       # release/artifact helper scripts
.github/workflows/             # CI and release workflows
```

## Local verification

```bash
cargo fmt --check
cargo test
cargo build --release
cargo run -p elysium-compiler -- schemas --output schemas/schema-catalog.json
```

## NeoNEI integration contract

NeoNEI should call a pinned `elysium-compiler` release artifact or an explicit local development binary. NeoNEI runtime code should consume compiled `dist-data` only and must not import compiler source or parse NESQL++ raw-export data directly.
