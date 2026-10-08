# Ore dictionary registrations

Source revision 38 and Catalog revision 37 carry the independent `ore-groups` and `ore-members` collections. These records describe Forge's native registry, including named empty groups. They do not create crafting recipes or ingredient/product edges.

Each group preserves its exact name, Java UTF-16 alphabetical order, and original member count. Its origin is `{owner:"Forge",handler:"net.minecraftforge.oredict.OreDictionary",key:name}` and its identity uses the `oregroup` prefix. Each original list position has its own member identity: `<group>.member_<eight lowercase hexadecimal index digits>`. Positions remain contiguous, including duplicate templates, so a query can retrieve one page with exact keys without scanning the whole member table.

The member template preserves registry name, raw stored metadata, typed NBT and signed decimal `stackSize` as `amount`. Zero and negative amounts are valid native registration metadata. They are not manufacturing quantities. Metadata 32767 remains a wildcard in the raw template.

`display` is a nullable concrete Item reference observed in NEI's item map. The exporter checks it with native `OreDictionary.itemMatches` using copies. Exact templates additionally require the same metadata and NBT. Wildcard templates can use a concrete example with different NBT; the original NBT remains in the template. The exporter never calls name, tooltip or `Facts.item` on a wildcard and never invents a metadata-zero fallback. A null display remains inspectable through registry, metadata, NBT and raw count.

Capture yields after at most sixteen rows or two milliseconds between native operations. Per-group counts are limited to one million and the display-example cache to 262144 entries in total. A bounded second pass verifies group templates and example lists, detecting same-size replacements and in-place NBT edits. Native calls themselves cannot be preempted. Full-domain capture includes these tables; a recipe-only capture leaves both empty. Independent `ore-groups` diagnostics run the same cursor without producing a Source.

The compiler validates origins, identities, sorted group order, complete position sets, int32 count strings and concrete display relationships. Topics and item-related links provide reference navigation. `/ore-groups/:id/members?offset&limit` serves at most 100 records and loads only the requested member shards; the web page requests twenty at a time. Online and offline requests share this query implementation.
