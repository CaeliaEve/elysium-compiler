# AdvancedSolarPanel molecular transformer

The Source40/Catalog39 development process `molecularTransformer` pins the
installed AdvancedSolarPanel `1.7.10-3.5.1` native record and tile. The installed
JAR filename ends in `3.5.1a`, while its own mcmod.info declares `1.7.10-3.5.1`.

The reader uses the actual ordered MTRecipeManager.transformerRecipes list,
opening before the NEI handler factory. It neither creates CachedMTRecipe nor
PositionedStack geometry nor modifies arecipes. Exact source/member/raw stack/
item/tag instances and field facts are checked through batch verification and
sealing; ready-empty, unavailable source, drift and closed access differ. Unknown
list, record, stack and metadata callback implementations are rejected before
executing their callbacks.

The native matcher compares item identity and literal metadata, including 32767,
ignoring NBT. The first matching identity claims the input even if quantity or
output capacity prevents work; later duplicates are unreachable. Those duplicate
entries have explicit exclusion evidence. Positive input counts are consumed
when work begins, without crafting containers. A zero-count definition still
requires a present input but consumes none. Counts greater than 64 are not
discarded: the native machine can match such stored stacks. Negative input or
energy and nonpositive results remain explicit unsupported definitions, never
silently clamped to valid recipes.

energyPerOperation is a total EU requirement, not EU/t or duration. Completion
depends on supplied energy and output space. An empty output copies the native
result prototype; merging into an item/meta-matching existing output preserves
that stack's NBT and adds the recipe count. Different tags therefore do not
necessarily block work, and the prototype's tags are not a fixed final result.
The output uses a potential quantity conditioned on nativeEnergyAndOutputAccepts;
material totals do not count that prototype as a guaranteed final item identity.

The typed process retains native index, raw counts, energy, first-match priority,
start debit and output/energy policies. An independently sourced note stays
outside recipe identity. Actual Java English/Chinese transports keep the same
raw recipe ID; they are not a whole-pack translation certificate. Compiler
rejects a missing process, fixed duration/rate, altered counts/predicate, missing
note or unconditional output. NeoNEI draws its own semantic layout and explains
native total energy, zero-presence and tag merging, without a progress animation.

The isolated native suite checks real canSmelt and gainFuel, including blocked
first recipes, zero/maximum energy, supplied-energy remainder, existing-output
NBT retention, larger stored counts, source drift, unknown callbacks, shared
registrations and production cursor ownership. The actual Java fixtures cross
Rust domain validation and the strict TS contract. Whole-pack pixel/animation,
complete English and production performance acceptance remain separate gates.
