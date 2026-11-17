# Salis wand replacement

Implementation status: current batch, not compiled or validated. The released
semantic.67 contracts do not contain this process; regenerate all contracts only
after the unified implementation is complete.

`salisReplacement` is required for `Edit::SalisWand`. It binds the pinned Salis
native owner, occupied-cell roles, signed cost, optional core materials, staff
attributes, charge preservation and two separate capacity observations. It does
not infer semantics from a display property. Components use literal metadata,
including 32767. The native component/wand/conductor/ore classification precedence
is expressed through finite identity predicates and their priority exclusions.
Repeated part/screw/conductor roles must have identical domains. A first-nine-cell
match counts occupied cells rather than stack quantities and rejects extra cells.

For rod replacement, preserved Vis is clamped before metadata resets. Cap changes
do not have that first clamp. Staff rods replace the complete native attribute
list. Metadata resets to `max(cost, 0)`, even for metadata-indexed casting bracelets.
Output samples are owned, unpaid state projections. Negative costs are only
payable under the configured creative waiver; six zero primal costs are distinct
from an empty cost list.

Optional self-payment is conditional on the loaded Salis workbench mixin, an empty
supply slot, no earlier normal output, research and native player payment. OLD
components and player discounts determine the debit. The payment record's capacity
is the new RESULT capacity after metadata reset; it must not be confused with the
earlier rod clamp. Preserve/clear is then applied to the paid wand. No fixed paid
output is invented and configuration is not proof of mixin execution.
