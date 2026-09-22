# Output quantities and native views

Revision 12 separates a fixed output `amount` from a computed `quantity`. Exactly one is present. Fixed amounts remain positive decimal strings. Computed quantities have `amount: null`; zero is not a sentinel quantity. Item changes require a fixed sample amount and cannot also carry a quantity rule.

The first two quantity rules preserve the GTNH sparging algorithm. They apply to fluid outputs and refer to a fluid input slot with one consumed, fixed alternative:

- `draw`: `input`, `after` and a positive decimal `limit`. After the ordered preceding draws in `after`, draw an integer uniformly from 1 through `min(limit, remaining input - 1)`, inclusive.
- `remainder`: `input` and `after`. Return the input amount minus those ordered draws.

All computed outputs sharing an input form one group. Each group has exactly one remainder, and its ordered `after` list includes every draw in the group. Omitting a draw or resetting the budget in a second branch is invalid.

Each referenced preceding output must itself be a draw from the same input, with exactly the preceding prefix as its `after` list. References must be unique and cannot include the current output. A chain has at most 128 draws. The compiler rejects a chain if any reachable remaining amount cannot supply a further positive draw while reserving one unit. It also checks that computed outputs have probability one and no item change. An empty `after` list is valid.

The rule stores the correlation, not only separate output ranges. For a 1000 mB input, five draws capped at 200 mB can each range from 1 to 200, but their sum cannot reach 1000; the remainder is always at least 1. A consumer must not treat the five ranges as independent unrestricted quantities. Displayed bounds use integer arithmetic and sequentially propagate remaining bounds. Bounds are descriptive; the browser does not simulate random game runs.

The exporter derives these rules from the pinned sparging implementation even when the registered output stacks contain results left by a previous machine run. It does not sample a random result during export. Other dynamic machines require their own explicit models; their zero placeholders remain errors until supported.

A native recipe view is a presentation, not an exhaustive inventory. Its slot references must exist in the recipe, but a view may omit real inputs or outputs. Exporters must still capture all supported semantic slots; an omitted source value cannot be discarded to satisfy a layout. NeoNEI shows unbound inputs and outputs beside the native view and includes them in details and recipe lookup. No invented native coordinates are assigned.
