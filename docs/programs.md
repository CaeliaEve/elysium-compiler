# Shared machine rules

Compiler/contracts 0.34.0 read Source revision 34 and publish Catalog revision
33. Both formats require a `programs` collection/table, which can be empty.
An optional category `program` references a shared content-addressed context.
The initial context is Forestry Squeezer rules. NESQL 0.37.0 registers ordinary
and Forge fixed-container recipe projections. Dynamic item callbacks remain
explicit unsupported entries; no full runtime coverage is claimed.

The whole ID hashes `{kind: "forestrySqueezer", rules}`. Each row contains
`id`, `program`, `offset` and `data: {kind, rows}`. The four section kinds are
`squeezerRecipes`, `squeezerContainers`, `squeezerFluids` and
`squeezerCallbacks`, retaining native ordinary/container/fixed-fluid/callback
registry order respectively. Every section has a chunk, even when empty.
Chunks contain at most 128 rows and must fit the 1 MiB Source record limit.
NESQL splits at 900,000 payload bytes. The reconstructed context is limited
to 16 MiB (32 MiB including transport overhead).

Chunk IDs are `<program-id>.chunk_<content-hash>`; the hash covers program,
offset and data. Compilation checks individual IDs, continuous section
offsets, complete sections, native rule validity and the reconstructed whole
hash. Duplicate chunks, extra empty chunks, missing references and unreferenced
programs are errors. Chunk storage order may change; native rule order may not.

Catalog keeps these typed chunks in sorted MessagePack partitions. NeoNEI's
shared `programs/<id>` API finds the matching key range with binary search and
loads only intersecting partitions through its existing verified cache. The
same reader serves HTTP and offline-worker queries. It returns chunks, not
executable scripts or a simulated machine result.

Each Squeezer recipe carries `process: {kind: "forestrySqueezer", program,
selector, time, chance}`. Selectors are `{kind: "ordinary", index}` or
`{kind: "container", container, filled}`. Both the program and selector enter
recipe identity. Validation resolves the selector, checks first-key precedence,
and requires agreement with the category program, native work steps and exact
discrete `Random.nextFloat() < threshold` probability.

Inputs retain native signed i32 counts and use `rule: {kind: "forestry"}` with
allocated consumption. This predicate is meaningful only in the shared process;
it is not an independent item predicate and cannot be nested in one. Display
examples retain registry/metadata/typed NBT. Null requirements remain holes in
the shared rule and keep their original slot indices in the projection.

Outputs use `quantity: {kind: "squeezer", nominal}` with signed i32 parameters,
no fixed amount and no independent roll. Fluid bounds are `[0, max(nominal, 0)]`;
remnant bounds include zero and the signed nominal count, except a zero native
probability gives `[0, 0]`. Zero/NaN native chance still has remnant-space
preflight checks. Conditional stock removal, possible partial consumption,
fluid-tank space, remnant allocation and power cadence remain shared semantics,
not fixed ticks or EU/t. A recipe cannot attach a grid, magic cost or fixed
duration/energy to this process.

Native helper observations and production adapter rows cover ordinary/fixed
selectors, signed/zero/null fields, registry drift and rejected altered rows.
Dynamic `IFluidContainerItem` behavior, real full registry coverage and game
visual capture require further work and unified live validation.
