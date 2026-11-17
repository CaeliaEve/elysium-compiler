# Targeted native returns

Contracts `0.40.0-semantic.85`, Source/Catalog revision 40.

`wirelessTerminalUpgrade` describes AE2FC 1.4.120-gtnh's
`WirelessTerminalRecipe`, with WCT 1.12.7. Despite inheriting a shapeless
display recipe, it only matches physical inventory cells 0 and 1. They must
contain a native wireless terminal without the boolean `infinityBoosterCard`
flag and a native booster. The process carries the registered class members.
Other cells are ignored by matching; the workbench still consumes one item
and applies containers in every occupied cell. Grid nulls here mean ignored
cells, not required emptiness.

Its output uses the existing patch action to copy the offered terminal,
metadata, whole NBT and stack count, then set the flag to byte 1. Exported
one-item outputs are specimens. Quantity bounds are conservative; material
calculators must not treat the specimen as a fixed yield per craft.

Native infusion optionally carries `randomAspects`. Gadomancy 1.4.8 stores
an aspect-to-base table and a `halfCap` flag, and refreshes its random cache
on access after an elapsed time strictly greater than 500 ms. Without the
flag, each amount is `nextInt(base)`; with it, the amount is
`floor(base/2) + nextInt(floor(base/2))`. Zero entries are omitted. The rule
requires positive RNG bounds. A randomized recipe has empty `baseAspects`
and `magic.aspects`: those arrays do not imply a free or fixed-cost recipe.
The random bases retain validated aspect references for online and offline
display. Capture reads the definition without invoking RNG-backed getters;
derived caches do not participate in definition drift checks.

Botania 1.12.28-GTNH's `botaniaCosmetic` container callback admits the audited
diva charm and tiny planet. `botaniaBlackHole` separately describes the
black-hole talisman. Exactly zero stored blocks produces no return.
Otherwise it copies the offered stack, debits up to 64 blocks with the
native signed-int behavior, clamps the resulting count to zero, and resets
metadata 1 to 0. Complete NBT and stack count are preserved. These dynamic
containers cannot be replaced with a fixed sampled remainder.

These contracts are verified with isolated native-JAR fixtures. They do not
constitute a successful game-session check or visual export.
