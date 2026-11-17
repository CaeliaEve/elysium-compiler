# Vanilla crafting tag-transfer projections

Development Source 40 / Catalog 39 and contracts `0.40.0-semantic.34` add
`process.kind = craftingNbt` and `change.action = {kind: craftingNbt, base}`.
This is specifically Minecraft `ShapedRecipes.field_92101_f`, not GT crafting.

`change.input` is zero, and `bindings` must be absent. Samples consist of the
base stack, followed by each input's candidates in input/choice order. Each is
an isolated native whole-compound transfer, not a matching inventory tuple.
No Cartesian candidate enumeration is required. The base item, metadata and
amount are preserved, and each projected NBT is the complete input compound
if present, otherwise the complete base compound. Empty compounds are present.

At display time the LAST tagged offered row-major cell wins. Horizontal
mirroring reverses each row's cells; grid translation preserves this order.
No tagged cell leaves the base result intact. Arbitrary offered tags follow
this declared rule, while captured examples only cover exported candidates.
The result family cannot gain machine costs, magic, missing cells, extra
results, tuple bindings or a single-input interpretation.

This local native mechanism and decoder proof does not establish whole-pack
counts, crafting-manager first-match reachability, native pixels, translations
or production performance. GT normalization/electric/enchantment behavior
remains a separate release obligation.
