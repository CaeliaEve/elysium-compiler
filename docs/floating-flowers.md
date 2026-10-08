# Floating flower crafting

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
