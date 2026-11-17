# Galacticraft circuit fabricator native law

The Source40/Catalog39 development contract `circuitFabricator` represents the
GalacticraftCore 3.3.13-GTNH machine. AmunRa 0.8.2's ARCircuitFab registrations use
the same machine; their category membership intersects the real GC registry.
Both handler names require this process. The old generic timed recipe shape is
rejected even if a consumer recalculates its recipe hash.

`slots` contains exactly five nullable item IDs, mapping to native inventory
indices 1–5. Null means required empty. Nonnull slots consume one item, ignore
count and NBT and match literal metadata; 32767 is not a wildcard. An entirely
empty definition is valid under this dedicated law. The native HashMap encounter
order claims duplicate predicates before AmunRa category filtering. Source
observations bind that order within the current native session; it is not a
cross-restart ordering guarantee. Every GC source row has an explicit captured,
shadowed, category-excluded or slot-inaccessible disposition.

`rawOutputAmount` is the original native count used in capacity admission.
Admission accepts an empty output immediately. Otherwise the output must have
matching item/metadata, and its existing count plus the raw count (signed Java
int addition) must fit both the inventory ceiling and prototype stack limit.
Only afterward does quick mode replace the basic item's metadata13 count with5
or metadata14 with2. `basicItem` binds the actual registered native role.

The prototype occupies native output slot6. An empty output copies its tags and
has no separate cap. A merge retains existing tags. Merged excess above64 causes
unit world-spawn attempts without prototype NBT, with pickup delay10; the spawn
boolean is ignored and the retained output becomes64. It is not guaranteed that
the world accepts every drop. The quantity stays conditional; a prototype cannot
be counted as a fixed final output stack.

Normal processing requires300 consecutive eligible ticks, resetting on a blocked
tick. `maxExtractGj` is20 or40 under hard mode, an extraction ceiling rather than
a measured fixed energy debit. There is no generic wall-clock duration. Normal
ticking gates completion; direct `compressItems` invocation still debits all five
input slots even if output admission fails. There are no crafting containers.

Input exemplars have no NBT, one-per-slot consumption and no returns. Notes keep
source-bound explanations outside semantic identity. English and Chinese Java
transport share the same raw recipe ID. No GUI coordinates, background, slot
textures or progress animation are part of this process.

The producer explicitly rejects negative raw metadata because vanilla
ItemStack.copy would clamp it. Unknown source implementations/callbacks, invalid
result counts, unavailable roots and observation-budget excess also fail rather
than synthesize or drop a successful record. These guards are not evidence that
every game registry has passed. The local native runner executes the merge and
world-drop branches in owned state and restores borrowed roots; actual game
collection, full native pixels, complete English content and production latency
remain separate export-agent gates.
