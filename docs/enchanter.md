# EnderIO enchanter

Pinned EnderIO 2.9.28 manual enchanter semantics. Source 22 / Catalog 21 add
`process: {kind: "enchanter", level, maxLevel, itemsPerLevel, cost}`.
`cost` is a signed 32-bit count of player XP **levels**, not RF, EU or XP points.
Other integers are positive; `level <= maxLevel <= 32767` and
`level * itemsPerLevel <= 64`.

Slot 0 holds one book and quill (`minecraft:writable_book`), ignoring metadata
and NBT. Slot 1 holds a single material stack. The native registry chooses its
first valid input match before checking quantity. A later record cannot become
eligible merely because the earlier record needs more material. The exporter
preserves this with per-choice `except` predicates; fully shadowed rows are
excluded. Base inputs compare item identity and optionally literal metadata,
including 32767, and ignore NBT. Native ore inputs expand the snapshot's ore
membership into item/metadata predicates, with wildcard ore entries preserved.
Unknown input subclasses and custom metadata getters are rejected.

The offered material count `n` selects `min(n / itemsPerLevel, maxLevel)` using
integer division. A row at a nonmaximum level therefore requires
`level * itemsPerLevel <= n < (level + 1) * itemsPerLevel`; at maximum level only
the lower bound applies. Ordinary slot capacity and the offered item's own
stack limit still apply. Input amounts record **consumption**, not an assertion
that every larger offered stack produces this row's output.

Pickup clears the book slot, consumes `level * itemsPerLevel` materials and
creates one fresh enchanted book containing only `StoredEnchantments` with one
short `id` and short `lvl`. Extra material remains, input NBT is not copied, and
container items are not returned. No duration or energy cost is synthesized.
The user needs at least `cost` levels, then native `addExperienceLevel(-cost)`
runs. Creative mode bypasses the XP gate/payment, but still consumes materials.
`cost` is captured from `baseCost + costPerLevel * level * level`, with Java
signed-int wraparound. Negative costs are retained, never clamped by export.

Each registered level has its own row and exact output fact. Compiler checks
input consumption, the writable-book predicate, output NBT/level, and rejects
conflicting fixed time, energy, crafting or magic fields. NeoNEI explains the
offered-count interval and XP gate in the existing conditions area.

The owned native NEI cache retains its background and text. Capture temporarily
pins its cycle clock to the row's level and restores it even on a draw error.
Native recipe, matcher, manager, cost and cache tests are offline conformance
checks; they are not a live player/container/GL acceptance result.
