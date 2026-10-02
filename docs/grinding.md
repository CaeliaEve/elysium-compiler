# EnderIO SAG grinding

Source 26 / Catalog 25 add the `sag` shared process, optional `reserve`
consumption and `grinding` output quantity. This captures EnderIO 2.9.28's
registered Recipe / CrusherMachineRecipe / TileCrusher path. Capture verifies
the exact NEI handler and sole native machine selector; unknown implementations
fail explicitly. Nothing runs a machine, world tick or random production task
during export.

## Selection and stock

The manager matches only material slot 0 against recipes in registration order.
One offered stack subtracts its whole count from the first matching positive
requirement. Two positive requirements cannot match that single offered stack,
so such records are excluded by proof. This adapter accepts positive item
requirements; unknown, nonpositive, fluid or malformed registrations fail.
`earlier` contains overlapping preceding predicates and their minimum amounts.
Any satisfied entry takes priority over this row. NBT is ignored by these
native item/metadata predicates, including expanded ore entries.

The material input has `allocated` consumption. The native consumer traverses
both physical input slots separately from matching: `process.slot` is -1 for
either slot or 0/1 for one particular slot. It deducts at most the remaining
requirement and available count. It does not require full debit to have occurred
and does not return containers. Slot 1 may thus also be used by a declared
material consumption rule when its contents match.

Input slot 1, if present, is optional ball stock with `reserve` consumption. Its
choices are the union of `balls[*].choices`, whose encounter order is retained:
the first matching registered ball is selected. Amount one denotes loading one
ball, not a per-recipe requirement. Admission permits registered balls, but an
active ball may exist with an empty reserve slot. `grinding`, `chance`, `power`
are round-trip binary32 strings; `duration` is the signed native energy budget.
Captured registered parameters are finite, grinding is 0..16777218, chance is
positive and power is nonnegative. Other registrations require explicit support;
they are not clamped into this range.

## Exclusion and task creation

`blocked` predicates test material 0 only. `oreBlocked` predicates test both
material and reserve slots, independently of whether the reserve is the active
ball. The latter is a finite snapshot of the native deprecated `getOreID` check:
choose the first wildcard ore registration; only if none exists, choose the
first exact-metadata registration. Names beginning `ingot`, `block` or `nugget`
exclude bonuses. Having any matching ore tag is not equivalent. Capture queries
owned examples, not the native `isExcludedFromBallBonus` method, whose ineffective
identity-based copy cache grows on repeated calls.

Output-space admission uses the original shared Java nextFloat draw. A failed
start retains that draw. At creation, if an active ball exists and neither
exclusion applies, the stored task draw is `original / active.chance`, clamped by
PoweredTask to [0,1], and native float energy is multiplied by `active.power`.
Otherwise both remain unmodified. These initial multipliers apply to all three
native bonus types. `bonus` means specifically MULTIPLY_OUTPUT; NONE and
CHANCE_ONLY both have false and identical behavior in this tile path.

## Lifetime and correlated completion

Power use charges an active, enabled ball by `(int)(currUse + actualPowerUsed)`.
At its duration threshold it is cleared. Whenever no ball remains, the same
power step may load one from reserve slot 1 and consume one stock item. It may
therefore replace a ball during the final power step. This replacement does not
recompute the stored task chance or energy.

Completion first attempts the task results once. If a ball still exists, the
task's use-ball flag is true, and `bonus` is true, the tile draws one more
nextFloat value `r` and executes the native binary32 loop:

```
for (float mul = completionBall.grinding - 1; mul > 0; mul -= 1)
    if (r <= mul) complete_the_same_task_again();
```

All passes use the original task and its stored draw. Within every pass an
output is selected iff `threshold >= storedDraw`, and attempts `nominal` items.
There are no independent output rolls. For example, grinding 2.5 yields two
passes or three when the shared completion draw is <=0.5. The active ball at
completion may differ from the one at creation.

Saved `GrindingMultiplierNBT` values need not match today's registry. Records
describe these native state transitions, not a sampled tile or guaranteed
delivery. Nonfinite/pathological saved values retain native IEEE behavior;
large positive float decrement loops can fail to terminate. No consumer executes
such loops. For terminating completions, a conservative integer bound is
`[0, nominal * 2^25]` when bonus is true, otherwise `[0, nominal]`: above 2^24,
binary32 spacing is at least two, and decrement either reaches that boundary
or stalls; below it, fewer than 2^24 decrements reach zero. This loose bound is
independent of the current ball registry. It is a safety ceiling, not an
attainable prediction; the UI shows nominal per pass and explains repetition.

Native output insertion compares item and metadata, keeps NBT of existing
stacks, and can lose residual output if space changes or repeated passes do not
fit. Source records describe requested results rather than an inventory snapshot.

## Verification

The existing native runner exercises the real selection, task creation and
completion code with owned inventory/RNG. It skips only unrelated client sound
initialization using an allocated tile; no world, tick loop or GL context runs.
Checks cover initial chance/energy, three correlated passes, reserve exclusion,
wildcard-before-exact ore precedence, NONE/CHANCE_ONLY, quantity-sensitive
earlier matches, cache ownership and the production selector gate.

One shared fixture extends the existing compiler, API and online/offline browser
scenarios. Validation rejects missing referenced items, incompatible consumption,
ball-stock mismatch, invalid parameters and independent output probabilities.
Native background coordinates and two progress tracks are retained. The homepage,
styles, layout and effects are unchanged. Full loaded registries and real visual
capture remain for unified game acceptance.
