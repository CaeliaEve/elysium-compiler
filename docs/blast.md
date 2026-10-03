# IC2 blast furnace

Source 31 / Catalog 30 adds `process: {kind: "ic2Blast", heat: i32}`, `consume: {kind: "staged"}` and the primitive `rule: {kind: "untagged", meta: boolean}`. The rules describe IC2 2.2.828-experimental; no world or machine is run during export.

`heat` is the captured, mutable `TileEntityBlastFurnace.maxHeat` (normally 50000), not temperature in kelvin. There is no fixed EU/t or recipe duration. Each server update heats/cools the furnace, then calls `work()` only if `heat >= maxHeat`. The front-facing adjacent `IHeatSource` receives the native signed-int request `maxHeat - heat + 100` when there is input or positive progress and heat is not above maxHeat; redstone also permits idle heating. No supplied heat, or no positive request, cools by `min(heat, 1)`. Heat-source callbacks, update scheduling and upgrades remain runtime conditions.

The primary input is slot 0. Its amount is the recipe manager's count gate, **not** completion consumption. Native cache precedence is verified against each displayed candidate; unknown callbacks or a different selected source fail capture. NBT is ignored by the supported IC2 item/ore predicates. The adapter accepts only inherited, constant containerless items: custom container predicates, container-bearing inputs and upgrade modules are unsupported, because native `consume(1)` can replace containers or consume nothing. This boundary is explicit rather than declaring those recipes impossible.

Air is input slot 1, with amount 1 and staged consumption. The native ItemStackWrapper for an untagged air item accepts absent or empty root NBT and rejects all nonempty tags. `meta: true` ignores metadata only when the native air item has neither subtypes nor durability; otherwise metadata is literal. The exported air anchor is untagged. Each successful air checkpoint adds the declared return stack to the separate empty-cell output. This return is per checkpoint, not per finished recipe. Tagged configured air items require an additional native tag/hash adapter and currently fail explicitly.

The state transition uses persistent signed-int progress:

- Missing input, no selected recipe, or failed output checks return without clearing progress.
- Progress 0 becomes 1 without consuming air.
- At 1, 1000, 2000, 3000, 4000 and 5000, nonempty air and room for the returned cell are required. A successful checkpoint calls `airSlot.consume(1)`, adds the return stack, and advances once.
- At progress >= 6000, the first product is added, then the second product if present, then `inputSlot.consume(1)` is called and progress resets to 0. Completion returns inactive.
- Other nonzero, non-checkpoint values increment. Native signed overflow/negative saved values are not normalized.

A fresh, uninterrupted, sufficiently hot operation therefore needs 6001 work calls and six air checkpoints. Input removal, pauses and replacing the input retain progress; final products are looked up from the **current** input. It is incorrect to debit six air cells when starting a recipe or advertise 6000 fixed wall-clock ticks. Manually corrupted/NBT-injected invalid inventory can bypass insertion and `consume` acceptance; such states are not canonical recipe inputs.

Both preflight product checks use the **main output slot**, including the second product. This can stall even if the slag slot is empty. Actual completion adds the second product to the slag slot but ignores the amount that did not fit: partial or total slag loss is possible. Main output slot 0 is fixed on completion; slag slot 1 uses the existing quantity `{kind: "potential", stat: "ic2:slagSpace", nominal: "..."}` with bounds 0..nominal. Empty output slots accept the entire offered stack even beyond its normal stack limit; occupied slots respect the native strict-identity capacity. Products after index 1 and recipe metadata are ignored by this machine, and are not exported as reachable products.

The owned IC2 NEI cache supplies the original primary/product coordinates. The native decorative air icon at (15,38) is also an explicit linked input. Its original background and progress animation are retained. No style, layout or effect changes are required.

Lightweight evidence: NativeBlastTest calls the pinned manager, inventory classes and private work() in an isolated bootstrap; it verifies a full 6001-call operation, count gates, six returns, persistence, output-slot defect, partial slag loss, empty-tag air matching and ownership. Shared contract tests reject invented fixed costs, ordinary consumption, six-at-once air and unconditional slag. Live registry/export coverage remains unexecuted until the final unified run.
