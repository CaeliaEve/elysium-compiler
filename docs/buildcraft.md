# BuildCraft assembly

Source 29 / Catalog 28 adds `Process.buildcraftAssembly { energy: i32 }` and
`Match.buildcraft { wildcard: bool, subtypes: bool }`. The pinned producers are
BuildCraft 7.1.44 and BuildCraft Compat 7.1.18, with the exact
`RecipeHandlerAssemblyTable`, `AssemblyRecipeManager` and `FlexibleRecipe` classes.
This contract does not describe the integration table or refinery.

The player selects assembly plans. The machine retains a craftable selected plan
or selects another plan; registry priority is not an unconditional exclusion.
Inputs are ordered requirements: all native fixed inputs first, then alternative
groups in their native order. Every choice in a group has the amount of the
first native alternative, even if later source examples have zero or different
counts. Each choice uses `allocated`, with no crafting container returns.

Visit the twelve inventory slots in ascending order for each requirement and
consume matching stacks until its amount is satisfied. Different alternatives
may contribute to the same requirement. Later requirements see the remaining
stock. This is greedy allocation, not backtracking; overlapping predicates may
fail for one inventory order despite another valid arrangement. Extra stock is
allowed and leftovers remain. Preview uses the native owned fake inventory.
The exported input list preserves requirements beyond the first twelve; the NEI
layout displays only twelve, so later requirements have no invented UI slot.
No-input plans remain expressible.

Matching always requires the same registered item. `wildcard` records whether
the original source template had metadata -1 or 32767, even when its displayed
sample was expanded to ordinary metadata. That flag OR offered metadata -1 or
32767 accepts immediately, skipping metadata and NBT. Otherwise compare metadata
only when `subtypes` is true and compare NBT with native typed equality: missing
and empty compounds differ, list element types matter even for empty lists,
floating signed zeros compare numerically, and all other fields remain intact.
NaN input predicates relying on native object identity are rejected unless the
source wildcard bypasses NBT. This does not normalize fact identities or output
NBT. Custom item metadata getters and unknown recipe overrides remain explicit
unsupported errors instead of being guessed from NEI examples.

On a craftable selected plan, current laser energy must be >= `energy`. The
machine then crafts against the current inventory, sets energy to
`max(0, currentEnergy - energy)` using native signed-int arithmetic, copies the
fixed result into the native output helper, and selects its next plan. Energy
may be zero or negative in the native registry, so the contract preserves it as
signed. `craftingTime` is unused by this machine: no fixed duration or EU/t is
invented. Pauses, stock, chosen plans and supply determine completion time.
The fixed output describes the newly crafted stack, not destination inventory
state; delivery follows native adjacent-output/drop behavior.

Assembly has no fluid slots. Positive fluid requirements cannot craft and are
excluded with per-row `native_machine_has_no_fluid_slots` evidence and original
registry indices. Empty alternative groups, nonpositive first item requirements,
invalid fixed outputs, nonpositive fluid templates and unmodeled classes remain
explicit errors. The native registry is authoritative: the adapter does not
copy NEI's facade display suppression or accidental early return at an invalid
recipe. Full live-registry coverage is still unverified.

The exporter snapshots the registry list, copies all facts and supplies an owned
view to the native cached-layout constructor. It does not execute unknown
callbacks, machine ticks, world actions, or live inventory operations. The native
test runs FlexibleRecipe/StackHelper against a small owned inventory, comparing
mixed consumption, ordering, wildcard/NBT boundaries, no-input plans, ownership
and the no-fluid proof. The existing shared fixture and Rust tests check contract
round trips, guard against invented timing/returns/probabilities and retain reverse
links. The existing homepage scenario checks the details and choices online and
offline; no frontend layout or styling changes are needed.
