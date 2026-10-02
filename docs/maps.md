# Vanilla map expansion

Source 20 / Catalog 19 introduce `process: {kind: "mapScaling"}` and a matching
output `change.action`. This is the native Minecraft 1.7.10 `RecipesMapExtending`
operation. A zero-count empty map returned by `getRecipeOutput` is only its
registration placeholder, never a zero-cost or fixed empty-map result.

The 3×3 grid consumes eight paper items (metadata 0, ignores NBT) surrounding one
filled map (any metadata/NBT). Ordinary shaped matching permits mirroring.
MapData obtained by the native world lookup must have scale < 4. If data is
missing, a client lookup returns null; a server lookup can allocate an ID, change
the offered stack metadata and initialize scale 3 with spawn-derived center and
current dimension. Missing data is therefore not a universal rejection rule.

The output examples are **pending crafting results**, before `ItemMap.onCreated`:
copy the offered filled map, retain metadata and all other tags, force count 1,
and replace `map_is_scaling` with byte 1. The compiler validates these observations
against the input samples; they do not claim that a sample map exists in the world.

When crafting finishes, `onCreated` obtains the old MapData, allocates a new world
map ID, changes result metadata, creates new MapData at old scale + 1 (byte
conversion followed by cap 4), and retains the old center/dimension. It does not
copy explored pixels or remove the scaling tag. Final IDs/data cannot be computed
from a source catalog alone. A consumer must label the output as pending and must
not treat its sample ID as the final map identity.

NESQL retains each original IRecipe alongside the native NEI cache entry. Special
handling applies only to the exact native RecipesMapExtending class; no display
picture heuristic or blanket zero-output correction is used. Capturing calls only
getCraftingResult on owned inventory/stacks, never world lookup or onCreated.
Local tests exercise that actual native method and cache ownership. World
allocation and GL display remain part of unified live acceptance.
