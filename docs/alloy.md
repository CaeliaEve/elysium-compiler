# Registered EnderIO alloy operations

Source 24 / Catalog 23 add `process.kind = "alloy"`, `Consumption::Allocated`
and `Quantity::SharedRoll`. These describe the installed EnderIO 2.9.28
BasicManyToOneRecipe / Recipe / AbstractMachineRecipe path, without pretending
that it is ordinary shapeless crafting.

## Runtime scope

The target GTNH `config/enderio/AlloySmelterRecipes_User.xml` disables the
independent vanilla-furnace path; registered GTNH alloy recipes remain enabled.
Capture checks the actual VanillaSmeltingRecipe.enabled field, the native
manager, and its machine registry selector. An enabled furnace path or unknown
selector is an explicit export error, not silent omission. This adapter does
not implement other configurations' dynamic furnace batches or their ore
preference cache. The captured recipe applies in ALL or ALLOY machine mode.

## Ordered requirements and separate consumption

`inputs` are requirement groups in native registration order. Their `slot`
is a requirement index, not an inventory coordinate. All alternatives of a
group have one positive signed-int amount and base item/metadata predicates
that ignore NBT. Ore members expand into the captured native predicate set.

Native matching visits the three machine input slots from left to right. For
each nonempty offered stack, it finds the first matching requirement whose
remaining item count is positive, and subtracts the **whole offered count**
from that one requirement. No match rejects the recipe. At the end, all
requirements must have nonpositive remaining counts. One offered stack cannot
satisfy two groups even when its total count is sufficient. Registration order
chooses the first matching recipe. More than three positive requirement groups
cannot match and are excluded by proof; empty predicate sets also cannot match.

Consumption makes a separate copy of requirements and available inventory. It
visits requirements first, then available physical slots. `process.slots[i]`
is the corresponding requirement's original consumption slot restriction:
`-1` permits any slot, `0..2` requires that physical slot. Matching itself does
not enforce this restriction. The consumer removes the minimum of remaining
requirement and available count, potentially continuing across slots. Native
start does not demand that every requirement was fully consumed. No container
return is applied. Thus `allocated` must not be reinterpreted as ordinary
fixed consumption from the displayed slot.

## Energy and correlated output

`process.energy` retains signed native RF. PoweredTask converts that int to
float and accumulates float work; power settings and stored energy determine
speed, rather than a fabricated fixed duration or EU/t.

Each output retains its registered order and `sharedRoll` quantity:
`{nominal: positive int string, threshold: native float string}`. The machine
draws one Java Random.nextFloat value for the task and accepts **every** output
whose threshold is greater than or equal to that same value. A failed start
keeps that draw for retry. With the native 24-bit grid, cutoff zero still has a
1/16777216 hit probability. Lower-cutoff outputs imply all higher-cutoff outputs;
their probabilities must not be treated as independent. The ordinary `chance`
field is neutral 1/1 because the shared rule owns the selection semantics.

Displayed bounds are zero through nominal. Native output admission tests the
current rolled results and the output inventory; a full inventory blocks work.
Merging compares item and metadata, not NBT, and retains existing stack data.
An empty slot takes the copied result. Partial merges can mutate the trial list
even if the native helper returns zero. At completion, changed output state can
leave some nominal output unretained; do not promise full delivery independently
of that state. The recipe records the requested native result, not a world-state
snapshot of successful insertion.

## Contract and verification

Compiler rejects unordered requirement indices, mismatched slot vectors, invalid
slots or quantities, unsupported predicates, fixed consumption, independent
output chance, fixed output amounts, missing shared processes, nonfinite or
out-of-range float thresholds, and conflicting grid/magic/duration/EU fields.
The current capture accepts positive requirements and item outputs with finite
cutoffs in [0,1]; other registrations fail explicitly. No unsupported case is
counted as a successful empty export.

The existing native test runner exercises the actual matcher, consumer, shared
roll endpoints and production selector validation using isolated native
registries. The shared Java fixture and existing API/browser scenarios preserve
the process online and offline. Native GL, a real world, and the entire loaded
registry remain unexecuted until unified live acceptance. Background, two native
progress tracks and item coordinates are retained; frontend explanations extend
the existing details area without changing the homepage or recipe layout.
