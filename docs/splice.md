# EnderIO Slice and Splice

Source 25 / Catalog 24 add `process.kind = "splice"` and `consume.kind = "wear"`.
The registered EnderIO 2.9.28 machine shares ordered requirement matching,
separate consumption allocation and shared-roll output selection with alloy
operations (see [alloy.md](alloy.md)). The common implementation is
`EnderAssemblyRecipes`; this does not add a second matcher or an interpreter.

## Ingredients and tools

The machine offers six ingredient slots, 0 through 5, to its recipe matcher.
Ordered requirement indices occupy `inputs[0..process.slots.length]`. The
parallel process slot vector retains native consumption restrictions -1..5.
The normal inventory stack limit is one. Ingredient admission also consults
the already occupied slots through `isValidRecipeComponents`; an arbitrary
placement that passes the completed-recipe predicate is not a guarantee that
every intermediate insertion order is admitted.

Tools are separate item inputs with slot 6 (axe) and 7 (shears), amount 1,
wildcard metadata/NBT and `wear` consumption. The exporter enumerates every
registered Item whose actual class is an ItemAxe or ItemShears, respectively;
NEI's iron axe/shears are examples, not the entire accepted set. The compiler
validates the structure, slots and predicates; class membership is a captured
runtime fact, not inferred from translated names, tool tags or durability.

Normal insertion enforces these tool classes. Task start itself only tests that
both tool slots are non-null; it does not recheck class, durability or enchantment.
Inventories forced to contain unrelated items outside normal admission are not
additional recipe alternatives. Tools never enter the six-ingredient matcher.

## Completion wear

After native result completion, the tile looks up the tools currently in slots
6 and 7. For each non-null, damageable stack it calls
`ItemStack.damageItem(1, fakePlayer)`, then clears that slot if its resulting
damage is at least its current maximum damage. This includes the tile's
exact-maximum clearing rule, earlier than ordinary `damage > maximum` breakage.

`wear` describes this conditional native operation, not a guaranteed decrement.
Unbreaking, unbreakable NBT, item-specific damage hooks, fake-player state and
break behavior remain relevant. The contract does not claim a deterministic
post-tool stack or enumerate random enchantment outcomes. Removing/replacing a
tool while a task is running changes the target of completion wear; completion
does not require the original tool object. No container-return rule is applied.

Energy, result threshold and output destination caveats match the alloy process.
Source outputs retain `sharedRoll`, including threshold-zero behavior. All tools
and ingredients survive compilation into queryable input groups. Existing recipe
details explain the distinctions; homepage/layout/effects remain unchanged.

## Verification boundary

The native test runner uses the actual pinned recipe manager, matching/consumption
classes, tile admission and damageTool method. Its minimal player supplies only
creative status and deterministic RNG; no world, tick loop or OpenGL is run.
It verifies ordinary wear, Unbreaking prevention and exact-max clearing, as well
as isolated production registry construction and rejection of unknown selectors.
The Java fixture adds one row to existing compiler/API/online/offline scenarios.
Full loaded registries, native visual capture and real game export remain for
unified acceptance.
