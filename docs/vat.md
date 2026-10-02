# EnderIO Vat

Source 23 / Catalog 22 add `process.kind = "vat"` and item consumption `upto`.
The evidence is EnderIO 2.9.28: VatRecipe, VatRecipeManager, VatMachineRecipe,
TileVat, PoweredTask and AbstractPoweredTaskEntity. Native test calls use the
installed JAR through the existing isolated native-machine test runner.

## Selection and amounts

Records follow the captured registry order. The machine chooses the first
matching native recipe, then the first matching item predicate at each required
slot. Two-slot recipes require slots 0 and 1; one-slot recipes require slot 0.
Item counts do not participate in matching. Base inputs ignore NBT and optionally
metadata; ore inputs expand to the observed native member predicates. `except`
retains earlier same-slot matching and earlier consumption predicates.

Each record represents a pair in the native precomputed fluid table. For two
reagents, the native float product is `im = left.multiplier * right.multiplier`;
input is `Math.round(1000f * im)`, output is
`Math.round(im * fluid.multiplier * 1000f)`. A single reagent uses its multiplier.
Multiple fluid entries overwrite the same pair, so the last fluid entry wins.
The adapter copies these actual table cells, including typed fluid NBT, rather
than recomputing the independently rounded NEI display. Input and output tanks
each hold 8000 mB. Positive output must fit completely before starting.

## Consumption and optional slot

`upto` is permitted only for Vat item choices. `choice.amount` is the maximum
removed at task start; one present item suffices. The native inventory removes
`min(requested, available)` and retains any remainder. The consumption amount
comes from the first matching item predicate across **all** declared slots,
independent of which predicate selected the fluid table pair. Zero/negative
native consumption becomes `keep` with example amount 1. No container return
or input-NBT inheritance is implied.

For one required slot, slot 1 may be empty or contain an extra item. `extra`
preserves the ordered native consumption predicates as `{id, rule, amount}`;
positive signed amounts remove up to that amount, nonpositive amounts keep it.
If none matches, one item is removed. These are references for explanation,
not mandatory ingredients or additional product edges. Ordinary insertion rules
still apply: an allowed machine state is not a promise that any extra item can
be inserted into an empty machine. Two-required-slot records have empty `extra`.

Fluid input uses exact identity and NBT. `consume` uses its positive table amount.
For a table amount of zero, `keep` means a matching non-null FluidStack is
required but it may contain **zero** mB; amount 1 is only the identity example,
not a minimum quantity. NeoNEI displays this branch as zero and explains presence.

## Zero output and energy

A positive output is one exact, certain fluid result. When native rounding
produces zero, outputs are empty and `zeroOutput` references the original target
fluid. It is not a producible item. Native FluidTank.fill with a zero result may
set an empty tank's fluid identity while adding no volume; this is not modeled
as a positive product. Material and energy use can still occur.

`energy` is the native signed int RF requirement. PoweredTask converts it to float
and accumulates float energy. Actual speed depends on machine power settings,
stored power and redstone, so no fixed tick count or EU/t value is invented.
Task inputs are copied before consumption; completion reselects against that
snapshot. The contract assumes the captured registry remains unchanged.

## Validation and display

Compiler rejects conflicting duration/EU/grid/magic, out-of-capacity quantities,
wrong consumption modes, missing/extra slots, malformed optional predicates,
dangling references, invented zero products and `upto` outside Vat. Unknown native
classes, custom metadata getters and unsupported/oversized tables are explicit
export failures; they are not skipped as successful coverage.

Native background/tank coordinates and multiplier labels are retained. The
handler has no progress animation to synthesize. Fluid links use the existing
details area so an extra 16px icon does not cover the native tank drawing.
NeoNEI hydrates process-only references and shows upper bounds in both selected
slots and candidate dialogs, including after changing the candidate.

The shared Java fixture contains two-slot, optional-slot and zero-yield examples.
Native tests cover understock, first-match shadowing, separate consumption,
last-fluid overwrite, float rounding, zero quantities, fluid NBT and copied
table ownership. They do not execute a world, player, full live registry or GL;
those remain part of the unified external acceptance run.
