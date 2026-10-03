# Shared machine rules

Compiler/contracts 0.33.0 read Source revision 33 and publish Catalog revision
32. Both formats require a `programs` collection/table, which can be empty.
An optional category `program` references a shared content-addressed context.
The initial context is Forestry Squeezer rules. It does not register a recipe
adapter, execute native callbacks, or change the existing recipe UI.

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

A future recipe process must bind the program ID and native rule selector into
recipe identity and agree with its category reference. The transport fixture
does not prove this unfinished process or full Squeezer registry coverage.
