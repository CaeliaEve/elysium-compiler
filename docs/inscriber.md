# AE2 Inscriber

The current producer uses Source 40 / Catalog 40. The Inscriber process shape is
unchanged in contracts `0.40.0-semantic.66`. This process describes
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

The producer now reads the actual API registry fields before any NEI handler
factory call. It never constructs CachedInscriberRecipe / PositionedStack and
never writes a borrowed NEI recipe cache. The API, registry, native HashSet,
material definitions, optional source, exact source owner and name-press item are
bound by identity. Both native MaterialStackSrc and ItemStackSrc are supported;
an absent optional means the name press is disabled. Arbitrary definition,
recipe, storage and metadata callbacks are rejected before invocation.

Per-batch checks cover roots, count and name-press values. Rows are owned snapshots;
per-row checks cover their original definition and nested item/NBT identities.
Sealing additionally checks the full native HashSet encounter order and every
definition. Same-content replacement is drift. HashSet encounter order is not a
cross-JVM stable identity, and this observation is not a cross-restart recovery
proof. Snapshot content and identity bindings have aggregate budgets; close only
releases owned state and is idempotent.

The current Compiler requires this native Handler to retain the Inscriber process,
even if its precise rules are also removed. NeoNEI enforces the same boundary in
query hydration and HTTP recipe decoding. The material calculator lists results
as conditional because earlier native registrations or name presses may win.
The explicit declared bottom reference remains hydrated even when it is not a
required ingredient. Older Inscriber process rows remain readable; no archived
Source, Catalog or fixture is rewritten to satisfy the new gate.

Native tests execute the pinned recipe/registry, precise Platform comparator and
real TileInscriber.getTask on owned inventory and API fixtures. They cover first
registry selection, swapped plates, top-absent precedence, name-press priority and
the count-greater-than-one guard. Only bootstrap API/MaterialType initialization
is isolated; recipe, stack-source, inventory, comparator and getTask bodies are
native. This is not a live-game registry census, AE network, powered tick, GL or
whole-pack language/pixel validation.
