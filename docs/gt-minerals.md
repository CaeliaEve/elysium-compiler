# GT mineral reference contract

Source 40 and Catalog 39 carry `gtMineralsDefinition`, `gtMineralEntry` and
`gtMineralDimensionShare` as non-consuming reference leaves. The two native
handlers have separate domains and source identities. They never produce a
consuming recipe or native GUI view.

The adapter targets the installed GT5 5.09.51.482 API, not newer upstream
`IOreMaterial`/`SMALL_ORES_BY_NAME` APIs. It reads initialized helper registries
after Forge `AVAILABLE`, verifies coverage against native builders and active
generators, and checks the dimension helper projection before sealing.
Batch checks are inexpensive size/readiness guards; full semantic verification
occurs before the source is sealed.

An entry preserves both the builder definition and the initialized generator.
Native short narrowing and constructor adjustments are not silently merged:
small ore max height is at least min+1 and amount at least 1 before narrowing;
vein max height is at least min+9 and size at least 1. Disabled vein definitions
can appear in NEI while having no active generator. Original dimension settings
including `false`, provider rules, biome restrictions and generator flags remain
separate from NEI's enabled dimension projection. `GTWorldgen.mDimensionMap` is a
runtime memoization cache and is not exported as generation configuration.

`getMaterialDrops(7)` supplies seven **host samples**, not drop products. Vein
host samples retain four semantic layer roles. The host policy explicitly
matches native dimension navigation's seven-host path; it does not claim the
item-navigation GUI's localized five/seven-host probe has the same display.
Actual small-ore drop candidates come from `mapOreMetaToOreDrops`, preserving
order, sample quantity, registry/meta/NBT and the native reverse lookup map.
This display list has no per-candidate probabilities; the webpage does not
invent them or call the samples guaranteed outputs. Native helper deduplication
by unlocalized item name is retained, so this is not a simulation of every
possible in-world drop, fortune level or material callback.

Dimension shares are native relative `amountPerChunk/sum` or `randomWeight/sum`,
not occurrence probabilities. The integer numerator/denominator, native list
ordinal and exact decimal representation of the native Double are retained.
NaN and infinities use explicit `nonFinite` tags plus null `nativeShare`; they
are not clamped, omitted or presented as 0/100 percent.

Compiler validates family/source identity, ordered coverage, role and host
metadata, item facts, constructor adjustments and dimension share closure.
NeoNEI consumes only the public Catalog contract through the same online/offline
query. Only the current page hydrates items and native textures; additional host
icons mount after disclosure. Old NEI slot positions, frames and backgrounds
are absent.

Local fixtures exercise actual installed builder/generator/host/share callbacks
with explicit vanilla specimen registries. They are contract evidence, not
proof of installed-pack item textures or full world-generation acceptance.
