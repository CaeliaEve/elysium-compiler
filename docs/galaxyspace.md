# GalaxySpace ordinary assembly

Source 38 / Catalog 37 / contracts 0.38.0 add the process
`{"kind":"galaxyspace-assembly","earlier":[CraftingSelector...]}` for the pinned
GalaxySpace 1.1.121-GTNH assembly machine. Selectors share the native crafting-grid
contract used by Railcraft and QED; they retain registry order, shaped empty
cells/mirroring and native greedy shapeless input order. They are conditions,
not additional consumed ingredients or item-use edges.

This process describes ordinary crafting with stable valid inputs, reset work,
an initially empty output slot and a stable usable supply tier. Native repair
is checked before registry matching. The adapter rejects ordinary recipe
predicates that can be intercepted by that branch; dynamic repair is not
represented by a fixed output sample.

Every occupied input cell consumes one item, with no crafting-container return.
The machine copies the selected recipe's fixed output directly into an empty
output slot, preserving native count and NBT, including counts above 64. It does
not invoke `ShapedRecipes.getCraftingResult`, so that class's copy-NBT flag is
inert for this machine.

`duration` and `energy` remain null. The completion threshold is the native Java
integer expression `400 / (1 + poweredByTierGC)`; arbitrary external tier input
must not be converted to a fixed 200-tick recipe. The source records operating
conditions and native power settings as properties. A tier of -1 cannot complete
through this path because the original calculation divides by zero.

The Compiler validates unit consumption, fixed unconditional output, no invented
container returns or fixed time/energy, grid integrity, prior-rule budgets and
every prior template reference. NeoNEI loads prior-only items for details but
does not add them to consumption or usage indexes. Conditions are shown as
paginated item requirements, including empty cells and metadata matching.

The shared Java fixture and offline native tests are contract and implementation
evidence. They do not certify the complete loaded registry, live graphics or
real-game export performance.
