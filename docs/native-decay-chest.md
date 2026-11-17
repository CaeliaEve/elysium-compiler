# GT++ decay chest

Source40 / Catalog39 development process `decayChest` models the installed
gregtech_nh 5.09.51.482 definition. New definitions declare `scope: nativeStateLaw`
and `stateLaw: tickableItemBatchV1`, admitting arbitrary metadata and native NBT.
The result is conditional on batch completion. Historical development
`freshUntagged` definitions keep their narrower untagged predicate and certain
eventual result; they cannot silently acquire saved-state admission. Ordinary
native dust inventory updates do not advance the chest's progress.

The native recipe, item and tile classes, registry index, input/result IDs,
native item threshold and original displayed time are separate facts. The
scalar process holds these rules; the four source-bound explanatory notes stay
outside recipe identity. Their text references may differ by locale without
changing the rule's ID. Explanations are not translated property values that
silently become new semantic recipes.
The displayed time is not a fixed recipe duration or total energy. The actual loaded
server tile increments a signed 32-bit counter and runs a batch at modulo 20
boundaries. Overflow can produce a 16-update interval. Initial phase, unloading
and lag prevent conversion into a fixed duration.

Each batch executes at most 20 steps, reading the ticking flag before executing
the next item step. For fresh positive thresholds, completion requires
`(maxTicks + 21) / 20` integer batches, using 64-bit arithmetic. The stored
informational maxTick tag does not replace the item's real threshold. The native
item uses signed long tick arithmetic, including overflow in both threshold-minus-
Tick and increment. A saved inactive state completes without a step. An empty
root or missing TickableItem is initially inactive and completes without adding
progress tags. A non-compound TickableItem throws the native ReportedException;
that is a fault and never a successful conversion. Other keys survive updates.
Numeric Tick and isActive use the installed Minecraft NBT primitive coercions:
float-to-long truncates, double-to-long floors, numeric boolean reads the low
byte of its native integer conversion, and nonnumeric primitives become zero.
The stored maxTick is informational. Java/Rust/TS owned batch evaluators preserve
these distinctions, including NaN/infinity and signed-long overflow.

Completion replaces the entire stored input stack with exactly one native result,
preserving that result's typed metadata/NBT and using no crafting container rule.
Normal insertion is limited to one, but NBT inventory loading can retain larger
stored counts. The reader copies the native result and never calls the mutating
getDecayResult/tile replacement path on borrowed data.

Compiler allows whole-stack consumption here only with the validated dedicated
process. It checks owner, scope/law pairing, positive threshold, exact fresh
batch formula, scheduler/replacement constants, no fixed duration/energy/grid/GUI
view, the correct input predicate, no containers and one native result. New
all-state definitions must declare a potential result with `nativeBatchCompletes`;
a certain result or missing law is rejected. It does not
reinterpret arbitrary `consume: stack` as ordinary crafting or permit unrelated
handlers to use this law. NeoNEI renders the native batch facts as text, without
a running-progress animation, and reports whole-stack inventory demand as
uncertain in materials totals. The web page's explicitly labelled single-batch
rule inspector shows fresh/saved/inactive/empty/fault branches and never plays
a simulated running-progress animation.

The current reader exits the NEI factory/GUI-cache/coordinate path, binds native
source/list/recipe/stack/item/tag identities and facts, checks drift and owns its
snapshots/close. The actual Java transport preserves its original content ID
through Rust deserialization; the conformance test restores the full unmodified
recipe/properties before domain validation. Whole-stack output previously failed
the real Source59 inspect gate; that failure remains evidence rather than being
fixed by relaxing the generic consumption gate.

The actual native runner supplies 336 owned batch comparisons spanning the
full NBT primitive types at TickableItem, saved Tick/active variants, numeric
coercions, overflow and threshold boundaries. Rust and TS replay those actual
observations; recipe-level tests separately reject changed native laws and
guaranteed results. This local contract does not establish whole-pack export
readiness, complete English content, native pixel/animation fidelity or
production performance.
