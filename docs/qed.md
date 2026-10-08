# Extra Utilities QED

The pinned Extra Utilities 1.2.12 `EnderConstructorRecipesHandler` scans one ordered
registry and returns the first matching recipe's `getCraftingResult`. All standard
vanilla/Forge shaped and shapeless registrations participate in this selection;
the native NEI `EnderConstructorHandler` only lists shaped and shaped-ore entries.

`Process.qed` retains `earlier: CraftingSelector[]` in native registry order,
including hidden shapeless predecessors. A selector with a grid retains explicit
empty cells, dimensions and the native mirror flag; it can be translated in the
3×3 inventory. A null grid means native greedy shapeless matching in row-major
offered-cell order. Item/meta matching ignores NBT; only a native wildcard
template ignores metadata. The four reviewed exact recipe classes are supported;
unknown subclasses or copy-input-NBT shaped outputs need a separate adapter.

The exported visible recipe is shaped, each occupied input consumes one item,
and container returns are empty. `TileEnderConstructor.updateEntity` decrements
all nine physical matrix cells by one on completion and does not invoke crafting
container callbacks. The output retains the native result stack and typed NBT.

`enderFlux: "20000"` records the default native buffer capacity required for a
complete craft. The buffer must be full and the output must fit; completion
resets stored energy to zero. A compatible existing output must fit both its
stack limit and the machine's 64-item limit. Supply-dependent time means `duration`
and `energy` (EU/t) are null. This contract describes recipes under the default
machine configuration, not a prediction of modified saved machine buffers.

Source revision 38 / Catalog revision 37 introduce this process; compiler and
contract package version 0.38.0. Selector preservation and consumption bounds are
validated; the catalog does not execute a new live QED inventory simulator.
