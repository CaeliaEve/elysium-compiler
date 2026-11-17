# MagicBees mana infusion: ordered snapshot contract

The installed `MagicBees 2.9.4-GTNH` adds
`magicbees.main.utils.compat.botania.SpeciesRecipeManaInfusion` to Botania's mana
infusion registry. The pool adapter now publishes a mixed-registry
`nativeBotaniaPool` process backed by one `magicBeesPool` shared program.
Accepting this class as an ordinary fixed-output recipe would be incorrect.
Production integration has local native/contract evidence; full-pack live
diagnostics and export acceptance remain pending.

Read-only inspection of the installed bytecode and isolated native comparisons
establish these behaviors:

* Matching first checks the selected bee form through `BeeHelper.isMember`.
  Its native item identity identifies the form; offered metadata is ignored.
* `BeeHelper.getMember` reads the offered NBT. The native primary species is the
  active allele after dominance, registry resolution and genome repair rules,
  not simply the first `UID0` string.
* On a successful match `ForestryHelper.replaceSpecies` copies the offered stack
  and changes `UID0` and `UID1` on physical chromosome-list entry **zero**. It does
  not search for the chromosome whose `Slot` is zero. Metadata and unrelated
  tags survive; the result count is set to one.
* `getOutput` returns that cached result and resets the recipe cache to its
  constructor prototype. Export discovery must not modify the live recipe cache.

`NativeMagicBeesPoolLawTest` runs the installed callback and helper on owned
fixtures. Its reordered-chromosome case verifies that matching follows `Slot`
while rewriting follows list position. `magicbees-pool-native-law.json` is only
isolated behavior evidence, not a production adapter or an exported recipe.
The fixture allocates a recipe and supplies the fields normally set by its
constructor; it does not claim to test full mod initialization.

One prerequisite is now implemented in `ForestryMembers`: bee reads use the
native private `BeeGenome(NBTTagCompound)` constructor on owned tags instead of
`BeeGenome.fromNBT`, which inserts entries into a process-global LoadingCache.
Bee's normal reader retains scalar state; genome and mate are then bound to the
owned native reads. The installed native reader supplies the comparison oracle
for health, maximum health, generation, natural/analyzed flags, mate/default
state, duplicate slots and invalid-allele repair. The local fixture verifies
unchanged borrowed NBT and unchanged global cache ownership. This does not
alone admit the production pool callback, and it does not assert equivalence of every
preexisting global cache history.

`MagicBeesPoolSource` now binds the actual native FOREST template storage,
registry lookup and deprecated aliases, default/template contents, dominance,
slot compatibility, bee item roles and repair configuration. Getter-produced
array copies are not identity anchors. Template-only objects retain descriptors
without acquiring invented registry lookup entries. Equal-content storage
replacement and nested source changes fail verification.

`MagicBeesPoolLaw` is a snapshot evaluator, not production callback
admission. Each `Genome`/`Mate` read takes an explicit ordered `miss` or `hit`
observation. A miss follows the bound registry/repair law on owned NBT. A hit
uses its actual primary/secondary object descriptors and does not reparse or
repair the offered tag. The native allele identity comparison remains separate
from UID equality: an old object with the same UID cannot match a currently
bound recipe species. An unused cached Mate is assigned without dereferencing
its contents; an uncached Mate can still fail before species comparison.

The isolated native comparison covers historical species disagreement,
same-UID/different-object mismatch, null/wrong-type cached alleles, repair
suppression and malformed cached Mate. `magicbees-pool-cache-law.json` records
these explicit-read examples with their offered stacks. It is **not** a snapshot
of the live cache. Rust `botania_bee` and NeoNEI `botania-bee.ts` now replay the
same source/trace fixtures and offered-stack-dependent result law. These modules
remain explicit-trace comparison APIs; the integrated session below supplies
cache transitions to the published recipe path.

The three implementations now return the repaired offered stack independently
of the result copy. `magicbees-pool-transition-law.json` records four native
observations: successful result copying, repair followed by species mismatch,
repair retained before a later Mate parse fails, and a historical hit that
suppresses repair. Original inputs remain unchanged. These are stack
transitions with resolved read traces, not cache transitions.

`MagicBeesPoolSelection`, Rust `botania_selection`, and NeoNEI
`botania-selection.ts` now replay ordered ordinary/MiniFlower/bee callbacks
with explicit per-callback read traces. Five native observations in
`magicbees-pool-selection-law.json` cover ordinary and MiniFlower preemption,
an ineligible bee callback that repairs the offered stack before an ordinary
entry, low-mana blocking, and absence of an eligible entry. MiniFlower's
non-string key coercion is explicitly unsupported in this precursor. These
modules retain explicit trace APIs for comparison. The integrated session
described below separately carries output-cache state across invocations.

`MagicBeesPoolCache` adds a separate, read-only observation of the pinned
strong-key/value, 128-entry, one-minute access-expiring Guava cache. It binds the
segment routing/counts, bucket-chain order, insertion hashes, copied NBT keys,
access nanoseconds and actual chromosome-zero allele identities. It uses bounded
try-lock reads; it never invokes native get/getIfPresent, cleanup or entrySet.
Verification rejects changed identities, contents, counts and routing. This is
not a claim that segment locks exclude concurrent native lock-free readers or
that a caller's supplied clock is the live native ticker.

`magicbees-pool-cache-lookup.json` records isolated cache snapshots and native
comparisons for collisions, keys mutated after insertion, foreign same-UID
objects, exact expiry boundaries and signed-long nano wraparound. Rust
`botania_cache` and NeoNEI `botania-cache.ts` replay those single-instant reads.
The supporting native NBT hash implementation is checked against 84 typed/nested
Java observations, including signed zeros, canonical NaNs, UTF-16 and overflow.
Native hashing is for routing only: it must not replace lossless content IDs or
native equality. In particular, native signed-zero equality can agree while the
native hashes differ.

`MagicBeesPoolCache.bound` now checks the installed Guava 17 artifact SHA-256,
the dependency CodeSource/classloader and the exact native system ticker. The
initial observation is bracketed by two real native ticker reads. A custom test
ticker is rejected at this boundary. The observation also retains segment
thresholds, weights, read counts, access/recency queues and key-root aliases.
Shared mutable compound/list descendants belonging to distinct keys are rejected
rather than flattened. Segment try-locks do not exclude lock-free native readers;
the observer retains drift checks and makes no atomicity promise for concurrent
readers.

`MagicBeesPoolCacheMachine`, Rust `botania_cache_machine` and NeoNEI
`NativePoolCacheMachine` now implement serial owned access, expiry, load/failure,
64-read cleanup, expansion, unit-weight eviction, insertion hashes and stale
recency-entry identities. Every ticker read is an explicit input. Ten sequences
are compared to the verified installed Guava 17 and actual Forestry loader,
including collisions, capacity, repaired mutable keys, failures, NaN identity,
multiple segments and signed nano wrap. These are serial tests, not a proof of
arbitrary concurrent cache execution.

`MagicBeesPoolSession`, Rust `botania_session` and NeoNEI
`NativeBeePoolSession` integrate that cache with ordered callbacks and persistent
per-recipe output caches. Nine successive native invocations compare selection,
repaired offered NBT, exact ticker consumption, cached outputs and the complete
cache state. Matching runs before catalyst checks; an ineligible bee recipe can
still repair NBT and set its output cache. The first eligible match stops registry
selection even if mana is insufficient. `getOutput` returns the cached result and
resets to the constructor prototype. An unsupported or interrupted transition
invalidates the detached session; it cannot be reused as a successful cache
snapshot. An explicitly modeled native callback error retains its earlier side
effects and is distinct from an unsupported transition. MiniFlower non-string
keys use source-bound observations of the exact item/metadata/canonical NBT.
Absent/string keys can be independently evaluated, including across item
identities. Arbitrary unobserved non-string values remain unavailable; the
canonical model does not invent native compound iteration order.

`MagicBeesPoolRecipe` binds the actual recipe species/form fields, input,
constructor output and current output-cache identities without invoking
`matches`, `getOutput` or a live cache read. Equal-content replacement of the
output cache fails verification. `BotaniaPoolRecipes` now uses this binding in
the production reader, with full source/cache drift verification before sealing.

The `magicBeesPool` shared program is now typed in `ProgramData`, transported by
Java `Programs.beePool`, reconstructed and validated by Compiler
`botania_program`, and reconstructed by NeoNEI `reconstructNativeBeePool`.
Thirteen required sections split the source, registry order and cache state,
including explicit empty sections. Each chunk has at most 128 rows and 1 MiB;
the reconstructed rules have a 16 MiB limit. Per-chunk and whole-program hashes
reject missing, duplicated, mixed-family or altered chunks. Required nullable
observations are distinct from missing fields. The public contracts package
records this transport and the `nativePoolInput`/`nativePoolResult` operations
in contracts `0.40.0-semantic.83`. Native program fixtures replay the integrated
session after reconstruction.

When a registry contains the bee callback, every pool definition references the
same ordered program, including entries ineligible for the displayed catalyst.
Rows retain constructor input/output specimens. The output has an explicit
`nativePoolResult` action with **empty samples**: it is a constructor prototype,
not an observed successful conversion. Compiler and Catalog bind native class,
mana, catalyst, registry index, item/meta/NBT/count and category program.
Malformed cross-family or missing bindings fail validation.

`NativeBeePoolQuery` serves `/recipes/:id/pool-preview` through the shared
online/offline Catalog API. The item must come from the selected Catalog.
Each request owns a new serial session initialized from the exported cache.
The default explicit clock is the snapshot's `afterNanos`; it does not predict
a future running-world cache. Selection distinguishes no match, native callback
error, insufficient mana and the selected registry index. A selected different
index is not success for the displayed row.

NeoNEI labels the texture as a constructor prototype and displays the calculated
stack data separately. Failed/unavailable preview is explicit. Candidate changes
cancel stale requests, and material totals retain conditional input/output
instead of multiplying prototypes into guaranteed yield. Authored Chinese and
English notes are selected independently.

No global cache is replaced or mutated by production source discovery. All
oracle get/load operations use isolated test caches. Fixture runtime timestamps
describe the local native observation, not a live export or the user's future
recipe execution.

The following explicit limits remain:

* A mutable key can make a load's final lookup select another active entry,
  retaining an unresolved native loading reference. This corner is rejected;
  loading references are not silently converted into completed cache entries.
* Unobserved MiniFlower non-string subtile keys fail preview rather than being
  coerced from an invented compound iteration order.
* Sessions own a detached offered tree per invocation. External object aliasing
  and arbitrary concurrent cache users are outside this serial contract.
* Source discovery snapshots must pass final registry, cache and source drift
  checks. No concurrent atomic snapshot guarantee is made.

A complete implementation must retain the bound native allele registry,
deprecated-UID resolution, dominance, default-template and
`clearInvalidChromosomes` behavior, including the native NBT repair side effect.
It must also preserve first-eligible-registry-entry priority relative to ordinary
pool and MiniFlower predicates, and express the offered-stack-dependent output
through Java, Compiler contracts and NeoNEI together. Missing/malformed genomes,
reordered or duplicate `Slot` entries and earlier recipe preemption need precise
outcomes. A sample-only fixed conversion, generic tag patch or blanket callback
whitelist does not close this gap.

The production-admission fixture uses the real mixed registry reader and asserts
no live `matches`/`getOutput`/cache-load calls. Four Java recipe fixtures cross
the Compiler and public decoder; detached native sessions are replayed in Java,
Rust and TypeScript. This local evidence does not overwrite historical failed
handlers or establish full-pack export readiness.
