# Forestry squeezer: native rule foundation

This is an implementation checkpoint, not a supported-handler or live-acceptance
claim. `domain::forestry` replays owned Forestry 4.10.17 inventory and selection
values. NESQL's `SqueezerRules` snapshots the exact native recipe classes and
detects registry drift. Neither component is connected to a released source
recipe process yet. Source revision 32 and catalog revision 31 remain unchanged.

## Verified behavior

The existing optional Forestry native runner records 26 stock observations and
15 selection observations. Three compiler tests replay those observations and
validate a context produced by the production snapshot code. Expected results
come from the pinned native methods, not the Rust implementation.

- Condensation skips nonpositive original stacks, sums identical item/meta/tag
  stacks with Java integer overflow, and does not merge different ore variants.
- Null NBT and an empty compound are **different for stock condensation** in the
  pinned Minecraft runtime. An untagged or empty-tag *requirement* nevertheless
  ignores offered tags in Forestry's direct matching predicate.
- `containsSets` takes the maximum matching condensed stack ratio per demand,
  using native float division; it does not allocate inventory across overlapping
  demands. The required count 16,777,217 can pass with 16,777,216 offered items
  because of float rounding.
- Removal preflights once, attempts exact matching before ore matching, and
  retains partial mutations on failure. `removeSets` can report success because
  its result array has enough entries even when one entry is null. No crafting
  container return is stowed by the squeezer.
- Current-recipe retention uses ore matching. Fresh selection searches each
  physical slot for a container recipe before traversing ordinary recipes in
  their observed registry order, with ore matching disabled.
- Container map stack keys compare exact metadata and native tag equality;
  Item keys compare item identity; ore-name keys compare against actual ore
  members. Ore-key matching itself does not turn metadata into a wildcard.
- Forge fixed-container lookup ignores offered NBT; the selected recipe still
  copies that NBT into its one-item demand. The native fixed fluid amount and
  tags are retained. Remnant probability preserves raw IEEE-754 bits.
- A registered `IFluidContainerItem` is marked dynamic and omitted from the
  fixed-container table. Selection refuses to simulate that callback unless an
  already-valid retained recipe short-circuits container lookup first. Snapshot
  tests use a throwing callback to ensure it never executes during extraction.

All input stack values are independent copies. Shared Java object graphs are
not modeled. The retention predicate is exercised through the actual native
`containsSets` method and the branch read from `TileSqueezer.checkRecipe`; no
world, tile tick, player inventory or live game is invoked.

## Remaining integration

1. Add the recipe cursor and owned native cached projection, including container
   variants. Ordinary and container rules must both be represented.
2. Define and validate the source process without duplicating the entire global
   fluid registry in every recipe. Keep physical allocation, retained-state and
   fresh-selection requirements explicit.
3. Complete the native work-cycle boundary: fluid tank preflight, remnant-space
   preflight even at zero probability, energy/time integer arithmetic and
   upgrade semantics. Container packaging is a separate operation.
4. Preserve native UI coordinates, fluid tank, remnant slot and animated
   progress; connect only data descriptions to the existing web design.
5. Run focused adapter/contract/browser checks, then count the handler as locally
   implemented. Full live registry coverage remains a later unified export gate.

The three frozen files under `contracts/fixtures/forestry-*.json` are native
observations, not a GTNH source dataset. HashSet/map order is captured per native
run rather than assumed deterministic across JVM starts. Re-running the native
runner writes only `NESQL++/build/native-tests/`; it does not silently refresh the
frozen compiler fixtures or historical acceptance evidence.
