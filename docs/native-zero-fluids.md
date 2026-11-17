# Native zero-volume fluid inputs

The semantic development contract preserves the native `"amount": "0"` with
`consume.kind = keep`, no returns, and an explicit `nativeZeroFluid` matching policy.
Ordinary fluid inputs remain positive. Negative inputs and unclassified zero inputs fail;
they are never changed to one millibucket. Physical density/viscosity fields are independent.

| Policy | Native authority | Meaning |
| --- | --- | --- |
| `gregtechCost` | Installed GT 5.09.51.482 `GTRecipe.maxParallelCalculatedByInputs` | A zero fluid cost is skipped. An empty provided array can pass this cost calculation; a null array retains the native early guard. Registry lookup and machine-specific conditions are separate. Fluid cost calculation ignores NBT. |
| `gregtechPresence` | Industrial Rock Breaker native `ProcessingLogic.validateRecipe` | Requires Water and Lava fluid objects; its identity checks do not require positive amounts and ignore NBT. Both zero-volume conditions must be retained. |
| `vatMatch` | Installed EnderIO `VatRecipe.matchRecipe` and `FluidStack.containsFluid` | Requires the same fluid and NBT. A zero-volume offered object can match; null or another fluid cannot. There is no debit. |

The Compiler validates family provenance, consumption, amount, returns and native machine
shape. Rock Breaker also requires the actual Water/Lava identities and both slots. VAT
retains its table process and conditional zero output; a keep-one surrogate is rejected.
The schema does not permit this policy as an item matcher or primitive priority filter.

NeoNEI shares the policy validator between online/offline queries and material calculations.
The recipe displays zero explicitly. GT zero-cost entries have a separate explanation and
are not listed as required catalysts; native presence entries remain reusable conditions.
The helper has Chinese/English explanations; this is not evidence that the full UI/content
language collection is finished.

Small Java samples use the actual installed native API in an owned fixture. Their recipe
records are replayed through complete Rust domain validation, generated TS table decoding,
material calculations and an isolated stopped-backend persistent offline browser reopen.
Names, absent icons and shared baseline ore memberships are fixture scaffolding. This does
not certify production pixels, all machine state conditions, language coverage or throughput.

This change is in the unreleased semantic implementation (Source 40, Catalog 39,
contracts `0.40.0-semantic.32`). Unified compatible candidates and their final release
identity are still required before the export-agent handoff.
