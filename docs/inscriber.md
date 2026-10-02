# AE2 Inscriber

Source 21 / Catalog 20, compiler and contracts 0.21.0. This process describes
registered recipes in AE2 rv3-beta-695-GTNH, in the observed registry iteration
order (`recipe.order`). They are conditional branches of one machine, not
independent unconditional crafting recipes. Dynamic name-press operations precede
the registry and are not enumerated by this NEI handler.

`process = {kind: "inscriber", mode: "inscribe" | "press", top, bottom,
namePress}` uses nullable item IDs. The references retain declared templates;
`namePress: null` means its native item definition is disabled. Name identification
uses item and literal metadata, ignoring NBT.

For a nonempty center slot, the machine first checks the name operation: at least
one plate is a name press and all nonempty plates are name presses. Such a state
renames the center item rather than using any registered row. The exporter omits
registered rows provably entirely shadowed by this guard. Remaining rows keep the
guard in their process even when only some possible states are preempted.

The machine then chooses the first matching registered row:

- Every offered nonempty input stack must have count at most one. Normal usable
  inventory inputs are represented as one item; source template counts do not
  determine required amounts.
- When `top` exists, the two plates must match `(top, bottom)` in either orientation.
  A missing `bottom` requires the opposite slot to be empty. Required plate inputs
  occupy slots 0 and 1, center alternatives occupy slot 2.
- When `top` is absent, the pinned native boolean expression accepts either empty
  plate slot, regardless of the other plate or the declared `bottom`. Only center
  input slot 2 is required. `bottom` remains a metadata reference, not a use edge.
- Center and required plates use `kind: "ae"` matching, described below.
- `inscribe` retains both offered plates; `press` clears both, including arbitrary
  optional plates in the top-absent case. Both modes clear the center; no container
  returns. The single result preserves the registered count and NBT.

Output space is required. The machine checks the task again at smash output step
8, and ends the smash at step 16. Work draws `10 * (1 + speed upgrades)` AE per work
call, with progress dependent on the network tick interval and upgrades. Fixed
duration and EU/t fields would be misleading and must be absent. This contract
does not simulate network power, slot insertion, the inventory/world or elapsed time.

## Precise NBT predicate

`ae` compares item registry and **literal** metadata, including 32767. Counts are
irrelevant to this predicate. Null and empty root compounds compare equal. Nested
tag types and compound keys remain significant. Lists compare entries in order;
empty-list element types are ignored. Float/double tags use Java numeric equality,
so signed zeros compare equal. Other primitive and array tags compare exact values.

Facts and exact item IDs keep their original typed NBT, including empty compounds,
float bits and list element types. Normalizing an input predicate must not rewrite
the facts. `ae` is currently permitted only on an Inscriber process.

Exported anchors must have ordinary compound NBT, ordinary metadata getters and no
NaN values. NaN/shared-root equality can depend on object identity, which cannot be
recovered from serialized facts; these cases fail explicitly. An ordinary anchor
compares structurally even against an offered AE shared compound.

## Validation

Compiler checks process references, mode, required slots, plate binding, one-item
amounts, consumption, match rules, fixed result and absence of conflicting timing,
energy, grid or magic fields. It rejects rows entirely preempted by name presses.
NeoNEI loads process-only item references and explains conditions in its existing
details area. Reverse links remain observed input examples, not a general game
matcher or proof that an arbitrary offered inventory can reach a later row.

Native tests execute the pinned recipe/cache constructors and Platform comparator
on owned objects. The isolated loader exposes NBTTagList.tagList to satisfy AE2's
native direct access; it does not replace the comparator. TileInscriber routing is
audited against bytecode; no actual tile ticking, AE network, GL or live registry
coverage has been tested locally.
