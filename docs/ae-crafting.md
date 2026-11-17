# AE2 and Wireless Crafting Terminal

Source40 / Catalog40 development contract `aeCrafting` describes the pinned AE2
`rv3-beta-695-GTNH` and WCT `1.12.7` native recipe definitions. It carries the
original item/metadata predicates, logical cells, optional mirror and native
selection order; it does not describe a NEI screen or a running job.

Shaped matching tries column offsets, row offsets and then the permitted mirror.
Every cell outside the pattern and every null cell must be empty. Shapeless
matching reads physical cells in row-major order and removes the first matching
remaining native predicate. It does not perform bipartite matching or search for
a better assignment. For `[{apple, carrot}, {apple}]`, offered `[apple, carrot]`
fails, whereas `[carrot, apple]` succeeds. Counts and NBT are ignored by these
matchers; metadata 32767 is a wildcard. Each occupied workbench cell consumes one
item, with separately audited container/tool rules.

The Java reader opens the actual crafting registry before the NEI handler factory.
It observes native Ingredient / IngredientSet / GroupIngredient fields without
calling expansion methods that alter cached/source stacks. Baked candidates take
precedence. Missing group members can be skipped, but a RegistrationError rejects
the whole group. The compiled-away `IngredientSet.isInside` constant is not a
runtime guard. Unadapted interface implementations and recipe subclasses are
rejected before calling them. WCT's native shaped ArrayList candidate path is
supported; this is not blanket support for arbitrary third-party IIngredient
callbacks.

Native disabled entries retain an explicit exclusion and count. A missing registry
is not an empty success. Same-process source identity, member order, candidate
values, metadata representatives and final sealing checks are retained. Closing
releases owned state, never borrowed `arecipes`. This does not authorize reuse
across sessions, versions or languages.

NeoNEI draws its own cells, supports candidate selection and offered-order changes,
and hides the result when the current arrangement fails this native definition.
The material calculator keeps outputs conditional because it does not receive the
actual player's workbench. Earlier workbench registrations can still take
priority: this contract represents the selected native recipe definition, not a
proof that all other mods' recipes reject the same grid.

The lightweight native runner supplies four actual Java transport records and
648 real shapeless workbench probes. Rust validates the original Java recipe IDs
and native process; TS independently replays the probes. These are local fixtures,
not an installed-game language, pixel, animation or performance acceptance.

Open boundaries before unified export include complete workbench priority/lifecycle
review, zero-width/height native recipes if actually registered, additional native
ingredient callback owners if discovered, and the full remaining Handler audit.
