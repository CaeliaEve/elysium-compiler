# Railcraft rolling machine

Source 28 / Catalog 27. The pinned native implementations are Railcraft
9.16.33 and NEIIntegration 1.5.0. Both shaped and shapeless NEI categories read
the same ordered RollingMachineCraftingManager registry. The exporter snapshots
that list and owns its input, output and display copies; it never runs a world
or calls the machine update loop.

`Process.rolling` retains `powered` from `machinesRequirePower()` and `earlier`,
the preceding native selectors across **both** categories. Each selector has
ordered `inputs: MatchCase[][]` and an optional `grid`. These are exclusion
references, not positive ingredient edges. The current row applies only when
it matches and no earlier selector matches. Unreachable duplicate entries can
therefore remain described without claiming that they win native selection.

For a shaped selector the grid is row-major, including explicit empty cells.
It may be translated within the 3x3 inventory. Horizontal reflection is allowed
only when `mirror` is true. All cells outside the translated grid must be empty.
For a shapeless selector there must be exactly one occupied inventory cell per
requirement. Visit offered cells row-major; each consumes the first remaining
matching requirement. This is greedy native matching, not backtracking. A broad
wildcard followed by a narrow requirement can fail for one offered order and
succeed for another.

The four admitted exact classes are Minecraft ShapedRecipes/ShapelessRecipes
and Forge ShapedOreRecipe/ShapelessOreRecipe. Their predicates compare native
item and metadata, accept wildcard metadata 32767, and ignore input NBT and
example stack counts. Forge ore lists are snapshotted as their actual item
predicates, without inventing membership in a named ore group. Empty ore lists
are impossible and can be omitted with evidence. Unknown subclasses/selectors
are rejected because they could shadow any later entry. Custom item metadata
getters are not flattened to raw metadata. A ShapedRecipes copy-NBT flag is
explicitly rejected at capture, not presented as a fixed output. This flag
requires a future input-dependent output contract if encountered in live data.
All these limits remain visible failures; handler implementation is not a claim
of full live-registry acceptance.

After selection, every occupied grid cell loses one item. No crafting container
return callback is invoked. The machine normally requires more than one item
in every occupied cell, reserving the last item. `useLast` bypasses that gate
for one operation and resets after the completion attempt with available output
space. Thus the declared consumption is one, not two, and `Keep` is incorrect.
Slot balancing and adjacent inventory pulls can change the offered counts.
Automatic insertion rejects container items and unstackable items; manually
placed ingredients still use the native matching and completion rules.

The machine refreshes its cached result every eight ticks. Every active tick
also calls `canMakeMore`, which checks the current registry match and reserve
gate. Loss of a valid recipe or reserve resets progress. Pause is controlled by
the action update every sixteen ticks. With power enabled, a progress increment
requires and extracts 50 RF; otherwise it advances without RF. Normal progress
needs 100 increments, then a separate completion branch. Storage is 5000 RF and
the maximum receive call is 1000 RF. No fixed wall-clock duration or EU/t is
claimed by the recipe.

At progress >=100, the machine checks destination space for its cached result,
recomputes the current matching result, decrements all crafting cells by one,
and attempts delivery. It then resets progress and `useLast`. Input changes,
output space, pause and power can delay processing or alter completion. The
fixed result is a fresh recipe result, not a guarantee of destination state.

NativeRollingTest extends the existing Railcraft runner, using the real native
manager, four recipe implementations and both NEI cached-layout constructors.
It verifies registry precedence, shifted grids and holes, ignored tags/counts,
owned copies, lack of container returns, greedy wildcard order, and rejection
of unknown overrides and input-dependent NBT copies. The shared Java fixture,
Rust contract, API scenario and existing online/offline homepage scenario cover
both power modes and prevent priority references becoming ingredient links.
These are offline conformance checks. Installation, GL capture and complete
live registry enumeration remain part of the final unified acceptance.
