# Soul Binder

Source 27 / Catalog 26, EnderIO 2.9.28. This is a native process contract;
it does not execute a Minecraft world or claim arbitrary NBT simulation.

The exporter reads the existing `MachineRecipeRegistry.machineRecipes`
LinkedHashMap for `blockSoulBinder`, without calling the map-creating getter.
Only exact BasicSoulBinderRecipe, five built-in crystal/controller classes and
SoulBinderSpawnerRecipe are admitted. Unknown subclasses fail explicitly.
Registry order, including UID replacements in their original positions, is
preserved. Each row retains the earlier two-slot selectors. A row applies only
if both of its input predicates pass and no earlier selector passes both slots.
XP is checked after recipe selection, so an earlier recipe lacking XP still
blocks a later recipe. Extra material does not enable a fallback selector.

## Matching and consumption

`Match.soul.filter` contains an empty native vessel reference, sorted unique
nullable names (null first, then Java UTF-16 order), and an `exclude` flag. Extraction returns null unless the item
is that vessel and its root compound contains `id`. Metadata and all other NBT
are irrelevant. Otherwise extract **native Minecraft 1.7.10 getString(id)**.
This is not a typed string-only check: the pinned native test maps integer 7 to
`"7"`. Other tag types use native text coercion; a consumer must not pretend
that sorted typed-NBT JSON reproduces Java compound iteration or float text.

For a fixed recipe, membership in names admits the input; names can contain
null. In that case non-vessel items and vessels without id also match. For a
spawner recipe use the complement, with null and the actual powered-spawner
blacklist in names. Unregistered non-null strings are allowed when not blocked.
Entity registration and boss capture settings affect NEI examples, not this
spawner predicate. Canonical examples use string ids (or a tagless vessel for
null); compiler validation computes those examples only.

Material slot 1 uses item and metadata equality for fixed recipes, item identity
alone for spawners. Input NBT and declared example stack size do not restrict
admission. Each physical slot consumes **one** item at successful task start;
no container callback is used. Normal inventory capacity is one per slot.
Insertion also consults the current other input, through the native registry.

## Completion

PoweredTask copies the two offered inputs at start. Fixed completion scans them
in order; every vessel with an id overwrites the current mob type. The last
such vessel controls the final allow-list test. The material can itself be a
vessel, so a recipe can match at start and yield an empty result array.
`Quantity.soul` makes both outputs share this same gate. When true, attempt one
empty vessel and the fixed product's nominal count; when false, produce neither.
Zero-to-nominal bounds are conservative, not an independent probability.

Spawner material is a distinct native broken-spawner item, so it cannot
overwrite the soul. Output 0 is one empty vial. `Edit.soul` on output 1 takes
input 0's extracted string and constructs a fresh base-registry item, metadata
0, count 1, and a compound containing only `mobType` as that string. Input
entity data, prior spawner tags and metadata are discarded. Per-choice samples
must agree with this construction; the default output is the first sample.
As with other EnderIO tasks, destination space and output-stack changes can
reduce actual delivery; merging uses native item/meta rules and retains an
existing destination's tags. The examples describe fresh recipe results.

## XP and energy

Process fields retain native signed int RF, display levels, required raw XP,
configured tank capacity, and whether XP fluid exists (`drains`). Required XP:

- level 0: 0;
- levels 1..15: level × 17;
- levels 16..30: int(1.5 × level² − 29.5 × level + 360);
- otherwise: int(3.5 × level² − 151.5 × level + 2220).

Java double-to-int truncation/saturation applies. Native XP level requirements
are stored at recipe construction; RF may read the active config. Capacity is
min(configured maximum XP, INT_MAX / 20). Start requires stored raw XP >=
required raw XP. It does not subtract that many player levels. Only after a
successful task start does ExperienceContainer perform the native drain.

When XP fluid is absent the drain returns null and leaves XP unchanged, while
the gate still applies. Otherwise, for stored `x` and required `r`, all following
operations use signed Java int arithmetic: available = x × 20;
requested = r × 20; drainedLiquid = (min(available, requested) / 20) × 20;
debited = drainedLiquid / 20. The container resets then adds x − debited,
capped to its configured maximum. Division truncates toward zero, multiplications
and subtraction wrap. In ordinary bounded nonnegative tank states the debit is
r. Saved tank NBT can contain out-of-range state; do not silently replace the
native formula with an unconditional level subtraction or fixed fluid recipe.

RF per tick depends on capacitor and power. No fixed duration is invented.
Source/Catalog consumers retain this process even when no native view exists.

## Evidence

NativeSoulTest uses actual remapped EnderIO and EnderCore method bodies. The
test-only transformer isolates EnderIO startup fields and spawner configuration
initialization; it does not replace item predicates, recipes, XP methods or UI
handlers. Tests cover quantity/NBT, null/numeric ids, dual-vessel empty output,
blacklist complement, fresh spawner tags, raw XP debit, absent-fluid behavior,
read-only registry capture, native mob examples and unknown-class rejection.
The shared Java fixture, Rust validator, API scenario and existing online/offline
homepage scenario cover both fixed and spawner outputs. These are local
conformance checks, not installation, GL capture or full-registry acceptance.
