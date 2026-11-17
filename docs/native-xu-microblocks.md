# ExtraUtilities microblock rules

The Source 40 / Catalog 39 development contract models the installed
ExtraUtilities 1.2.12 `RecipeMicroBlocks` with the ForgeMultipart 1.7.2 material
lookup. This is not a shaped recipe inferred from a NEI screenshot.

`xuMaterials`, `xuNames`, `xuAliases` and `xuSettings` chunks share one
content-addressed program. Block rows retain native ID-map order. Name IDs are
signed Java integers. Name aliases are resolved once; missing or unknown names
resolve to zero. XU does not require a cutter or saw strength, so this program
does not invent those FMP fields. A purely literal batch can mark its lookup
`unused` and emit empty material sections, including with an empty native map.

Each `xuMicroblock` procedure binds its native recipe index, nine literal,
integer or empty cells, input layout, output prototype and observed native
material text per input slot. Literal cells preserve raw registry, metadata,
count and typed NBT even though native admission ignores count/NBT. Metadata
32767 is a native wildcard. Concrete visual witnesses must come from the ready
native item list, never a fabricated metadata-zero fallback.

Numeric zero cells require a full block whose first native ID-map match resolves
to a positive ID. Other numeric cells require a micro item with a compound and
a nonzero resolved ID; the native matcher deliberately ignores shape metadata.
All numeric cells must agree on the same material ID. Material lookup is not
required by literal-only templates, which are emitted once.

`scope: nativeRegistryWitnesses` declares concrete, count-one witnesses, rather
than pretending to enumerate arbitrary player NBT, material names or inventory
counts. Two states preserve offered base/mirrored placement, the actual native
matched orientation, the first tagged input slot and resulting stack. Matching
tries mirrored first and visits columns; output separately visits physical
rows and copies the complete NBT of the first tagged micro item, including a
literal cell. If none exists, the output prototype remains intact.

`xuResult` samples follow those states. Identical candidate bindings do not
identify a state: the offered mirror flag determines the selected output.
Compiler validates the entire rule, program and both physical states. NeoNEI
uses the same cached program query online and offline, hydrates only current
inputs/results/prototype, displays nine physical cells and links mirror state
to output selection. The materials tool treats the output as conditional.

Native dimensions greater than three have no matching loop and receive an
explicit scoped exclusion. Narrow templates expose the installed native
`recipeItems[-1]` bug and fail explicitly. Unused array tails remain source-bound
but do not become input cells. Missing readiness, unknown callbacks, source
replacement, drift and closed access remain errors. Historical contracts and
captures are not converted into new successful data by this reader.

Actual Java fixtures exercise ordinary numeric, tagged literal-first, mirrored
different output, literal-only empty lookup, untagged literal followed by a
tagged micro and untagged-only prototype branches. These controlled transports
do not establish whole-pack native pixels, complete English or production
latency acceptance.
