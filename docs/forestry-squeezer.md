# Forestry factories: native rule foundation

This is an implementation checkpoint, not a supported-handler or live-acceptance
claim. `domain::forestry` replays owned Forestry 4.10.17 inventory and selection
values. NESQL's `SqueezerRules` snapshots the exact native recipe classes and
detects registry drift. Source revision 35 and catalog revision 34 carry
shared rule chunks through a category's optional `program` reference; see
[shared programs](programs.md). Ordinary/fixed-container production cursors and
their process are connected. Dynamic-container coverage remains partial; no
live acceptance is implied.

## Verified behavior

The optional native runners record 26 stock, 15 selection, 19 squeezer work,
16 powered-machine and 22 still observations. Six compiler tests replay these
observations and validate a production rule snapshot. Expected results come
from the pinned native methods, not the Rust implementation. These small
runners take seconds and do not launch Minecraft or perform an export.

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
- A registered `IFluidContainerItem` is omitted from the fixed-container table.
  Audited null readers and Forge/IC2 callbacks with disjoint container keys have
  explicit descriptors. IC2 reads create an empty compound on untagged original
  stock before fresh ordinary lookup; retained recipes bypass that effect.
  Unknown callbacks still fail. Snapshot tests use throwing read/drain overrides
  to ensure no callback executes during extraction. The seven callback observations
  use the original Forge, IC2 and EnderStorage methods on owned fixture stacks.

All input stack values are independent copies. Shared Java object graphs are
not modeled. The retention predicate is exercised through the actual native
`containsSets` method and the branch read from `TileSqueezer.checkRecipe`; no
world, tile tick, player inventory or live game is invoked.

## Squeezer work and powered arithmetic

`forestry::work::observe_work` checks a selected recipe against the physical
stock, product tank and remnant slot, then applies native completion helpers.
The 10,000 mB tank must fit the entire output. A zero output can pass even with
an incompatible full tank. Remnant-space preflight applies even at zero or NaN
probability. Empty slots accept an oversized remnant; occupied slots require
the native stackability, stack limit and tag equality. Probability is rolled
only after successful input removal and only for a non-null remnant.

The native runner calls `TileSqueezer.hasWork`, `InventorySqueezer` and
`StandardTank` methods. It does **not** call `TileSqueezer.workCycle`, which
would resolve a player from a world. The completion observation uses that
method's native inventory/tank helper sequence with an explicit Random seed.

`observe_power` describes one Squeezer `EnergyManager` call, not a game clock:

- Work steps: `Math.round((float) time / speed)`.
- RF: signed int `time * 200`, difficulty scaling via `Math.round`, followed by
  power scaling via `Math.round`.
- Storage capacity: `Math.round(5000f * difficulty)`.
- Per-step consumption: `(int) ceil((float) energy / (float) steps)`.
- Native subtraction and CoFH capacity-clamp order are retained, including
  overflow. All multipliers retain raw binary32 bits.

`TilePowered` checks every five game ticks. It only calls the energy manager
when its work counter is below the required steps, and resets the counter after
successful completion. Zero/negative-step direct energy observations are not
claims that the scheduler would consume that energy. Speed upgrade setters and
recipe selection can reset the counter; container packaging is separate.

The existing NESQL Still/Centrifuge rows now leave fixed `duration` unset and
provide `forestry:workSteps`, `forestry:stepTicks` and `forestry:energyRF` as
metadata. The RF value is the signed native parameter before difficulty, not
EU/t or a guaranteed complete-cycle charge.

## Still input reservation and truncated output

`forestry::still::observe` runs the owned equivalent of native `hasWork`, then
`workCycle` if ready. The independent native runner calls those two actual
methods on an unticked tile. It observes:

- The buffered liquid takes priority over the resource tank for selection.
  A still-valid retained recipe wins; fresh lookup follows observed HashSet
  order and matches one input unit with native fluid/tag equality.
- Without a buffer, `hasWork` reserves `cycles * input.amount` immediately,
  **even when the output tank is blocked**. This occurs before the power gate.
  A blocked attempt keeps its buffer; completion clears it.
- Output preflight checks one unit, while completion fills the full batch and
  ignores the amount that did not fit. With 3 cycles, 10 mB input/unit, 3 mB
  output/unit and only 5 mB free space, the native tile consumes 30 mB, produces
  5 mB (not 9 mB), reports success and clears the buffer.
- Zero/negative quantities, signed multiplication overflow and the tank's
  copied filter are represented by the model rather than converted to positive
  quantities. A zero output can complete without storing any fluid.

NESQL's existing positive-batch projection now uses the already-versioned
`potential` quantity with `stat: "forestry:stillTankSpace"`, preserving the
nominal batch and a 0..nominal conservative bound. Its native UI projection
still shows the nominal batch. This fixes the false fixed-output claim without
changing the web layout.

This is **not** a completed Still process integration. Its source recipe still
needs explicit selection/reservation context; signed/zero batch handling and
the Centrifuge pending-product lifecycle remain open. The new model and native
fixtures are not a live dataset or evidence of full registry coverage.

## Remaining integration

1. Connect the owned `SqueezerEntries` cursor to production recipe capture.
   It now enumerates ordinary rules and registered fixed-container variants,
   copies each returned recipe, and constructs the native cached layout through
   the ordinary-recipe constructor. The container constructor, which enumerates
   arbitrary item callbacks, is not used. Six fixture entries compare to native
   lookup results; signed/zero fields and null input holes are retained.
   Compiler `SqueezerSelector` resolves the same positions and refuses a fixed
   container that bypasses the first matching native map key or is dynamic.
   These internal selectors are not yet a source recipe process or handler route.
2. Define and validate the source process without duplicating the entire global
   fluid registry in every recipe. Keep physical allocation, retained-state and
   fresh-selection requirements explicit.
3. Connect the verified work/tank/energy model to a versioned process, preserving
   scheduler/reset semantics. Complete Still and Centrifuge process integration.
   Container packaging is a separate operation.
4. Preserve native UI coordinates, fluid tank, remnant slot and animated
   progress; connect only data descriptions to the existing web design.
5. Run focused adapter/contract/browser checks, then count the handler as locally
   implemented. Full live registry coverage remains a later unified export gate.

The frozen files under `contracts/fixtures/forestry-*.json` are native
observations, not a GTNH source dataset. HashSet/map order is captured per native
run rather than assumed deterministic across JVM starts. Re-running the native
runner writes only `NESQL++/build/native-tests/`; it does not silently refresh the
frozen compiler fixtures or historical acceptance evidence.
