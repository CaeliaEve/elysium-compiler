# Native CreativeCore crafting

The installed LittleTiles 1.5.14-GTNH bundles this CreativeCore API. New records use
`creativeCoreCrafting` and `creativeCore` input rules; ordinary unordered matching
cannot express its behavior.

`minimum` is a signed stock threshold. The workbench debits one item from each
occupied cell, independently of that threshold. The reviewed per-item container
returns remain separate. Item predicates ignore metadata and NBT; itemstack
predicates retain literal metadata (including 32767), nullable typed NBT, and
`needNBT`. The stored NBT must remain even when `needNBT` is false: another
predicate's equality can inspect it.

Shaped matching scans column offsets, then row offsets, trying original placement
before mirror. Native height truncation can leave an unused tail. Zero dimensions
can match an empty grid and are not a recipe-GUI error. Shapeless matching reads
physical cells row by row, picks the first accepting remaining predicate, and calls
`ArrayList.remove(Object)`. Removal invokes the selected predicate's equality on
earlier entries and can remove a different instance. Equality is not necessarily
symmetric.

Ore selectors describe the actual baked Forge index, rather than its display stack
table. Code zero is the item-wide branch; other codes compare the wrapped 16-bit
`metadata + 1`. Block/material membership is a reviewed item census with stable
reachable block identities. This is a native family, not a general expression DSL.

Validation checks native source, dimensions/order, active versus unused predicates,
compact grid holes, complete canonical sample census, unit debit, and the fixed
output. It does not turn a sample into evidence of arbitrary unknown callbacks.
The producer explicitly rejects unaudited fuel callbacks, identity-sensitive NBT,
unknown containers/material getters and missing reachability proofs. These remain
visible capability boundaries until separately reviewed.

Actual Java transports are replayed through raw recipe identity and full Domain
validation. Tests also exercise first-equal removal independently in Rust and TS.
This controlled local evidence is not full modpack pixels, English or server-world
acceptance.
