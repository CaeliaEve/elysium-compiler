# Floating flower crafting

The current implementation batch introduces `nativeFloatingFlowers` with the
paired `nativeFloatingFlower` edit. It has not yet passed unified validation or
contract version generation. It records native `getString` observations for all
listed final-special-flower examples, rather than restricting them to strings.
String and absent tags are independently checked. Other values retain a native,
source-bound string read; canonical NBT never supplies compound storage order.
Coercions, final choices and output samples align one-to-one, with cumulative
byte/candidate budgets. Results get fresh NBT containing only the coerced type.

The previous process described below retains its historical string-only domain.
The new-release native gate and archived reader boundary are part of the pending
unified contract/version work; the old behavior is not evidence that this batch
has been validated.

`floatingFlowers` describes Botania's ordinary recipe and default crafting-slot
settlement. A 2×2 or 3×3 grid contains exactly the displayed number of occupied
cells, with at least one floating flower and one special flower; other cells
are empty. The 36 count branches cover 2–9 occupied cells and every split
between the two flower types. Each occupied cell consumes one item.

The `special` suffix lists input slots in increasing **physical grid** order.
The last special flower supplies the result's type. It is restricted by an
explicit `string_tag` predicate: the `type` root key is absent or a string,
with any metadata and other tags. Other special flowers ignore metadata and
NBT because their type values are overwritten. This is a partial input-state
adapter: non-string `type` on the final flower is not implemented.

The paired `floatingFlower` edit takes only the registry from its renderable
`base` example, creates metadata 0/count 1 and discards all base/input tags.
It writes only the final special flower's string `type`; a missing key becomes
an empty string. Output samples verify this transformation, not the full NBT
copy shown by the native NEI display.

Minecraft 1.7.10's actual `getString` can stringify non-string NBT. Compound
stringification depends on native map iteration order, which canonical typed
NBT does not preserve. The adapter therefore does not silently turn those
values into empty strings or invent an iteration order. Global CraftingManager
priority, crafting events and arbitrary mod callbacks are outside this
ordinary-recipe/default-settlement model.

The native handler must retain this process, including when its dynamic output
is stripped from a malformed row. Current rules are presented from the typed
process in separate Chinese and English UI text; fixed English scope/boundary
properties no longer enter either language's Source. Material calculations
retain a conditional result rather than a guaranteed batch output. See
[botania-native.md](botania-native.md) for the related pool and altar laws.
