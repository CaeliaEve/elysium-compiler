# Elysium Compiler

`elysium-compiler` validates NESQL game snapshots and publishes immutable NeoNEI catalogs. Source records use canonical JSON Lines in gzip shards. Catalog tables use standard MessagePack maps, with lossless WebP texture atlases.

## Stable CLI

```bash
elysium-compiler inspect --input <source-snapshot>
elysium-compiler compile --input <source-snapshot> --output <catalog-root> [--report <receipt.json>]
elysium-compiler check --input <catalog-root>
elysium-compiler schema --output <schema.json>
```

All commands return one JSON result on stdout and diagnostics on stderr. A receipt must be outside both source and catalog directories. Catalog output must be outside source; output paths containing `..` or symbolic links/junctions are rejected before creating directories. Failed validation leaves the current catalog unchanged. There is no old-format reader or migration path.

`inspect` returns `id`, `environment`, `scope` and a `counts` map covering every verified source collection. The old handful of top-level domain counts has been removed; consumers should read `counts`.

Compiler and contract package version 0.10.0 use source/catalog revision 10. Matching rules belong to individual input alternatives, crafting grids survive exports without views, and magic costs retain typed aspect and research references. The environment declares a sorted set of construction parameters; every structure records one successful build or explicit failure per set. Construction palettes bind each placement's block state to a model or explicit appearance failure. Models preserve native faces, sprite UVs, vertex tint and transparency passes, while textures share the existing asset and atlas pipeline. Complete sources cannot omit requested models. The generated schema pins format and revision to the Rust constants; the contract exports those values as `formats`. No migration reader is provided.

`current.json` selects `catalogs/<content-id>/manifest.json`. Manifest descriptors pin every table and image by path, byte length and SHA-256. Tables have at most 4096 rows and 16 MiB per shard; atlas pages have at most 4096 × 4096 pixels and 80 MiB encoded. Partial exports retain their `selection` scope. The compact `index` table supports recipe filtering and category counts without decoding all full recipes. `links` preserve category and recipe order reported by NEI.

## Repository layout

Native recipe progress is represented by `clip` view elements referencing shared content-addressed `tracks`. A track contains discrete tick durations and non-overlapping normalized clipping regions; texture and destination use the same fractions. Empty regions hide an element, consecutive equal states are merged, and tracks never execute game code. Validation rejects missing or unused tracks, invalid float coordinates, overlaps and oversized timelines. The source has 20 domain collections and the catalog has 25 tables. Tracks describe UI demonstration cycles, not actual recipe duration.

```text
crates/elysium-compiler-core/  # source, identity, domain validation, catalog publication
crates/elysium-compiler-cli/   # thin binary entrypoint
contracts/                     # generated JSON Schema, TypeScript bindings, JS decoder, shared fixture
scripts/                       # release/artifact helper scripts
.github/workflows/             # CI and release workflows
```

## Local verification

```bash
cargo fmt --check
cargo test
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo run -p elysium-compiler -- schema --output contracts/schema.json
npm ci --prefix contracts
npm run generate --prefix contracts
```

## NeoNEI integration contract

Input-dependent outputs have an explicit `change` with a source slot, a patch/merge operation and one native sample per input choice. A patch replaces named root tags and applies integer limits while preserving the item, metadata, count and all other tags. Limits follow Minecraft's numeric NBT conversion, including missing tags, Java narrowing and float flooring. The default output ID and quantity denote the first sample. The compiler recomputes every sample; samples are excluded from semantic recipe identity but remain covered by snapshot file hashes. All samples enter reverse recipe lookup. Armor is an explicit item fact, independent of durability.

Tag matching declares equal, present and absent root keys separately. A present `sceptre` tag is significant even when its value is zero. Salis wand replacements describe separate material slots, native state changes and optional self-payment through `MagicRecipe.payment`. Payment uses the old wand and player discounts before retaining/capping or clearing charge; quantities in charge tags and payment capacity are hundredths of Vis. Displayed samples assume external power. The `creative` flag records the configured creative-mode Vis waiver; an empty cost list requires that waiver, whereas explicit zero costs remain payable in survival. No arbitrary-NBT search index or paid-result simulation is implied.

NeoNEI consumes a pinned compiler executable and the packaged `@elysium/contracts` bindings. It reads catalogs, and never interprets game source snapshots or compiler internals. Quantities remain decimal strings; item identities include the full typed NBT. Every alternative input carries its own consumption and returned stacks.

The five integration tests exercise source integrity, precise identities and quantities, recipe references, actual atlas pixels, deterministic publication, corruption rejection, and the real CLI. The shared fixture is written by the Java exporter. Game capture coverage and full GTNH performance still require in-game validation; passing the fixture checks does not establish that coverage.

Material and circuit records preserve registry composition, reduced rational material content, real item/fluid forms and the target diagram provider's circuit families. The `topics` index supports paginated exploration and item associations without reading every full material record.

Species and mutation records preserve Forestry defaults, product semantics, climate and visibility flags, descriptive conditions and each mutation's result genome. Base rates use reduced fractions; tree possibilities have no fabricated rate. Identical mutation registrations remain separate occurrences. The `lineage` index reads only the requested parents or offspring page, and `topics` covers bees and trees. These adapters still require real GTNH capture acceptance.

## Release

Thaumcraft aspect definitions, item tags and research prerequisite metadata use explicit identities. Research links distinguish registered studies from the game's `@` knowledge flags. Unsynced knowledge is null, not false. Aspect cycles, missing references and contradictory research observations fail validation. Additional magic recipe adapters, complete research-book pages and actual game acceptance remain unfinished.

Structure records preserve actual StructureLib piece coordinates, controller markers and advisory placement items. Geometry is content-addressed in bounded `shapes` records; validation checks positions, rules, references, bounds and declared counts. A piece is not an inferred assembly or an executable world predicate. Unavailable definitions are explicit and cannot appear in a complete source. Real GTNH capture and assembled multiblock coverage remain outstanding.

`node scripts/release.mjs [directory]` builds for the current host, runs the actual release executable through schema comparison, fixture compilation, repeat publication and catalog verification, then copies the binary, contract package files and documentation into a fresh directory. The default is `release/<platform>-<architecture>`. `files.json` records the actual version, platform and file hashes. Existing destinations are never overwritten.

`node scripts/check-release.mjs <binary> [sha256]` repeats the same artifact check without rebuilding. Contract files under `contracts/` can be packaged independently with `npm pack`; NeoNEI consumes the resulting pinned tarball.
