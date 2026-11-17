# Mobs Info drop references

Source 40 / Catalog 39 development contracts carry Mobs Info as read-only references,
not consuming recipes. `MobHandler.getResult()` is empty and is not a drop catalogue.

`mobDropsDefinition` binds the native handler origin, counts, readiness and hidden-mode
policy. `mobDefinition` binds the native rule key and display alias, entity class,
native name, mod, health, infernal eligibility, peaceful/vial flags, discovery,
informational item representations, spawn descriptions and registration-event notes.
`mobDrop` retains each original rule ordinal, quantity, item/NBT identity, type,
base chance integer and scale, looting/player restrictions, enchantment power,
damage-map traversal order and weights, chance modifiers and native additional notes.

Base chances are not clamped. A chance modifier includes its original class,
priority, native tooltip lines and `writeToByteBuf` payload as lowercase hexadecimal.
The serialized state is provenance, not executable webpage code. Neither Compiler
nor NeoNEI evaluates world, victim or attacker callbacks. Native descriptions remain
the display authority for dynamic conditions. A raw value above 10000 is not a
fixed probability; Mobs Info itself displays zero records as below its 0.01% resolution.

Mobs Info's sample generator uses a mob-wide damage weight total, whereas the NEI
display permutation selects damage metadata differently. Preserve the native total
and weights; do not invent independent normalized damage probabilities. Keep duplicate
drop-rule provenance and entity aliases instead of merging visually identical items.

The Java adapter snapshots published pages and their authoritative `MobRecipe` map,
uses inexpensive identity/count guards per batch and re-reads the complete rule and
discovery state before sealing. Modifier buffers have a maximum capacity and are
released on both success and failure, preserving original errors. Unknown callbacks,
cache disagreement or inconsistent rule weights fail explicitly.

NeoNEI's `/mobs` route returns a paged summary and only visible member facts. A locked
summary retains the native name and mod but omits rule counts and details. The
`/mobs/:entity/drops` route hydrates only the current visible page. Online and offline
workers share the same bounded index and do not return hidden drops.

Current local evidence covers real installed `MobDrop` and `BaseChance` classes,
Java-to-Rust full-domain validation, generated TypeScript decoding, shared API,
actual Compiler publication, a browser journey and backend-stopped persistent offline
reopening. Fixture item names and absent icons are explicit test scaffolding.

Outstanding before claiming complete live Handler acceptance: readiness proof for
legitimate empty/disabled client caches, entity model/renderer content and any boss
information previously generated only by native drawing, native bilingual extraction,
and a fresh complete game session. Infernal pools have their own native distribution
and still require a separate adapter; this module does not simulate or absorb them.
