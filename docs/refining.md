# BuildCraft refinery

Source 30 / Catalog 29 adds `Process.buildcraftRefinery`, separate from assembly
and Galacticraft refining. It stores per-attempt `energy: i32`, native timer
`delay: decimal signed i64`, the current default per-tank `capacity: i32`,
`earlier: Stack[][]` and `filling: [fluidId[], fluidId[]]`.

The producer is BuildCraft 7.1.44 / Compat 7.1.18. It requires the exact singleton
RefineryRecipeManager to agree with BuildcraftRecipeRegistry.refinery and admits
the exact FlexibleRecipe class. Its snapshot preserves the manager HashMap's
actual iteration order, not sorted recipe names. All preceding preview selectors
are kept, including zero-energy recipes that can shadow executable entries.
Unknown callbacks are rejected before a partial priority interpretation is used.

At native updateRecipe calls, visit that order and cache the first recipe whose
preview returns a result. `earlier` consists of its preceding ordered fluid
requirements; they are selection references, not ingredients of this row.
Preview checks each requirement against the same two input tanks without
decrementing them. Duplicate requirements therefore each see original stock.
Do not sum them into a single preview gate or apply sequential subtraction there.

Completion tries each requirement in order, draining the first and then second
matching tank until satisfied. Each drain can refresh the machine's cached
selection, but the in-progress FlexibleRecipe method continues its original
requirements. If a later requirement cannot be fulfilled, it returns null with
earlier drains retained. Thus two 700 mB requirements can preview successfully
against 1000 mB, consume all 1000 mB, and yield nothing. `allocated` amounts record
these ordered requirements, not guaranteed per-operation deductions or outputs.
The fixed output is made only if the actual completion succeeds.

The machine first verifies output space for the cached result. It then consults
SafeTimeTracker with cached `delay`, and at a due attempt requires
`RFBattery.useEnergy(cost, cost, true) > 0`. If the current recipe preview still
succeeds, it deducts that cost once and attempts actual crafting. No energy is
refunded on failed completion. Zero or negative costs cannot satisfy the native
strict-positive return check: the exporter records
`native_energy_gate_rejects_nonpositive` evidence instead of emitting production.
The NEI visual label treats the field as RF/t and multiplies by time, but the
machine does not. Its native illustration is preserved; typed facts and recipe
details explicitly explain the per-attempt RF cost.

`delay` is a world-time retry interval, not a fixed duration. The timer starts at
Long.MIN_VALUE; it compares lastMark + delay with current time using Java signed
long arithmetic, updates its mark on a due attempt, and resets on backward world
time. Due attempts without enough power still advance this timer; output-space
failure occurs before it. This machine uses no random-delay range. Default battery
capacity/receive are 10000/1500 RF; persisted native battery parameters may differ,
so an energy cost above the default capacity is not globally declared unreachable.

`filling` snapshots the manager's two independent allowed-fluid lists. Membership
uses FluidStack equality without amount. The lists are appended on registration
and are not pruned when a recipe is removed. Rebuilding them only from current
recipes changes machine behavior. Fill tries tank 0 then tank 1. SingleUseTank's
selected fluid filter and existing contents further restrict filling; filter
state is machine state, not a fixed recipe choice. Result draining, filling
(including simulations), input depletion and NBT loading can refresh selection.
Default input and output capacities come from LIQUID_PER_SLOT at capture.

Current admitted requirements and filling predicates have absent NBT, which makes
native fluid comparison exact without replacing identity or stripping tags.
Tagged fluid state, item-bearing recipes, empty/nonpositive requirement lists,
nonpositive outputs and custom recipe classes remain explicit unsupported errors.
These are visible scope limits, not full live-registry acceptance claims. Inputs
beyond the two native NEI fluid positions remain in the semantic list without
inventing UI slots. The native display uses owned fluid/tank copies.

NativeRefiningTest extends the existing BuildCraft runner. It exercises actual
FlexibleRecipe, RefineryRecipeManager, CachedRefineryRecipe and RFBattery on owned
test state: duplicate preflight, partial failure drains, registry order, removed
recipe filling permissions, quantities, native energy gate and display ownership.
It loads only CoFH's integer-only energy API interface dependency from the pinned
installed JAR. No world ticking, game installation or source export is performed.
The shared Java fixture, Rust validators and existing API/browser scenarios check
the contract, state-reference hydration and separation from positive use links.
