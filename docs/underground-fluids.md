# GT underground-fluid references

Source 40 / Catalog 39 development contracts. `PluginGT5UndergroundFluid` is world-generation information, not an operation consuming dimension icons to produce fluid. No NEI GUI is created, drawn, captured or published.

The pinned target is the installed GT5 5.09.51.482 bytecode. Its API differs from current upstream code. NESQL++ waits for Forge AVAILABLE, reads `GTMod.proxy.mUndergroundOil.getDimensionList()` and each `GTUODimension.getFluids()` in its actual native selection iteration order, then checks the initialized `GT5UndergroundFluidHelper.getAllEntries()` projection. A ready empty configuration is valid; an unready or stale helper is an explicit failure.

Four typed reference leaves retain:

- `undergroundFluidsDefinition`: source, readiness, non-consuming semantics, exact policy names and counts.
- `undergroundFluidDimension`: raw configuration key, optional native display abbreviation, native accumulated weight, entry count and null-selection tickets.
- `undergroundFluidEntry`: per-dimension native ordinal, registry, optional registered Fluid identity, raw weight, Min/MaxAmount, DecreasePerOperationAmount and selected tickets.
- `undergroundFluidDisplay`: the separate native helper rows and their per-fluid ordering, displayed chance and range.

The installed native selector draws `nextInt(1000)`, computes `weight * 1000 / maxChance` with integer division, tests `remaining <= threshold`, and subtracts each rejected threshold. Thus normalized weights and the NEI `weight * 10000 / totalWeight` display are not the exact selected probability. Zero-weight first entries can receive a boundary ticket; floor division may leave null outcomes. NESQL++ preserves this law without "correcting" it. Production uses the equivalent O(entries) interval calculation, checked against all 1000 actual native callback inputs in a small fixture. Compiler independently checks selected and null ticket counts. It rejects overflowing/invalid configurations instead of clamping them.

Unknown dimension mappings and unresolved FluidRegistry names retain the original configuration and weight. Unresolved entries have no fabricated native icon. They remain in the native selection distribution; their weight is not redistributed. Helper projection equality is checked as a multiset including dimensions, registry, displayed chance and amount range. Changing helper order or configuration changes the capture signature.

Min/MaxAmount are configuration parameters for `GTUOFluid.getRandomAmount`, not guaranteed per-pump mB output. The pinned amount policy is:

```
upper = floor((MaxAmount * 100 * 5000)^0.2)
minimum = (MinAmount * 100 * 5000)^0.2
veinData = int(max(minimum, nextInt(upper) + nextDouble())^5 / 100)
```

Actual pumping yield also depends on current vein data and machine behavior. The page labels selection probabilities as configured-entry selection, not successful extraction probabilities.

Shared `/underground-fluids` queries accept raw dimension, registry/abbreviation search, optional fluid identity and bounded pagination. Only visible rows hydrate Fluid facts and textures. The immutable lazy index has count and 64 MiB serialized-data budgets; formal combined worker/GPU/process budgets are separate implementation work. The strict client checks row content identities, source/category binding, projected display and native-fluid references. Online and persistent offline use the same API.

Local fixtures prove contract and UI behavior, not the whole installed pack's live initialization, native pixels or real-world generation. Those remain in the unified Export Agent acceptance batch.
