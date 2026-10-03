# BuildCraft integration: native model

This is the native model and its cross-language conformance evidence, **not a completed
recipe adapter**. No source/catalog revision, current catalog, or frontend UI changes
are made by this step. The integration-table identity remains pending in the repair
ledger. Do not include it in implemented-adapter or live-acceptance counts yet.

## Evidence and ownership

The model is derived from BuildCraft 7.1.44 and BuildCraft Compat 7.1.18. NESQL's
`NativeIntegrationTest` invokes the actual locally remapped native methods on an
isolated registry; no mod lifecycle, world update, game export, or creative-subitem
enumeration runs. Its output is `build/native-tests/integration-observations.json`.
The byte-identical fixture is `contracts/fixtures/integration-observations.json`.

There are 25 observations: 23 can be replayed against an immutable rule, and two
exercise mutable registry drift at the Java boundary. Success comparisons include
the entire primary stack, eight physical expansion slots (including holes), and
the output, with exact typed NBT. The fixture also contains expected native failure
cases. Additional assertions reject unknown recipe and expansion callbacks, custom
item subtype callbacks, and writes to borrowed stacks. These are representative
conformance cases, not full registry coverage.

`IntegrationRules` checks the exact three audited recipe classes. It rejects
non-inherited metadata/subtype callbacks and board serializers it cannot describe.
Gate identifiers must inherit `GateExpansionBuildcraft.getUniqueIdentifier`;
robot boards must inherit the base `createBoard` and a known constant/field ID getter.
Native preview and completion receive an owned copy of the full stack graph. Even
native exceptions after partial consumption cannot mutate the caller's inventory.
Recipe costs, chipset mappings and board mappings are checked again before each
observation. The immutable block registry is captured once, not hashed per sample.

The Rust model accepts independent physical slots with canonical string registry
and board names. It deliberately rejects non-string name examples whose Java NBT
text conversion is not implemented. The native predicate still has those coercion
semantics; rejection must remain explicit rather than broadening a sample into a
claim about every possible NBT value. Java-only object aliases also require an
explicit representation or rejection before entering the value-based contract.

## Transformations

- **Gates:** copy the primary item, metadata and tags, force output count one, then
  visit occupied expansion slots in physical order. Each RED chip toggles logic,
  including repeated toggles, without being consumed. Other matching chipsets
  install the first not-yet-installed registry expansion and consume one on
  completion. Duplicate installed expansions and unrelated items are skipped.
  Registry order and BuildCraft wildcard/NBT equality are significant. A nonempty
  list with the wrong element type is replaced by a new string list. A typed empty
  list is returned by `getTagList` even with another element type; its append is
  refused, but the recipe still counts the change and consumes the chipset. An
  even number of RED toggles likewise still produces an output.
- **Facades:** the first wire and first eligible facade are selected in slot order.
  The first wire is not also the facade; a later wire can become the facade and
  then fail because it has no facade state. The GUI accepts plugs, but the native
  craft selector never selects them. It is incorrect to invent a plug-to-transparent
  recipe. Legacy facade tags migrate even during preview. Unknown block names
  resolve through Minecraft's defaulted registry to air. The selected facade's
  first state replaces the first state using the selected wire, or is appended.
  Hollow is retained, transparency is cleared, metadata is narrowed to a byte,
  and invalid wire ordinals default to RED. The output is a fresh global facade,
  metadata zero, count one, with only `type` and `states` tags.
- **Robots:** use the first occupied expansion, decrement it only on completion,
  and resolve its board ID (unknown or absent IDs use the empty board). Automation
  can insert non-board items; the native static parser still reads their tags.
  A new output is made from the global robot item, metadata zero, count one, with
  only a freshly serialized `board` and an integer `energy`. Exactly zero energy
  becomes 20000; negative values remain negative. Existing extra board parameters
  and unrelated primary NBT are not copied. The missing-board path writes an empty
  board ID at the primary root even during preview; this mutation is also tested.

## Machine process to bind next

The table has primary slot 0, physical expansion slots 1–8, and output slot 9.
`getExpansions` omits holes but preserves order. An existing selected recipe remains
selected while its primary predicate still matches; otherwise the first matching
registered recipe wins. The recipe's maximum expansion count constrains admission,
not the native craft loop over already occupied slots. Automation checks slot
availability but does not call the GUI expansion predicate.

Preview/output-space checks run every 16 ticks. This is a check cadence, not a fixed
craft duration. Invalid/no work and blocked output discard stored energy; a craft
also clears the entire energy buffer, not only its cost. The machine reduces the
primary by the returned output count and removes zero-count expansion stacks only
after successful output. The observation primitive models `craft`, not these world
updates, and must not be presented as a complete machine simulator.

Remaining work before marking this entrance implemented:

1. Bind the model to a versioned process/output contract and validate correlated
   input tuples. A single-primary-axis `Change.samples` array cannot represent it.
2. Enumerate the three native rule families and bounded display examples using an
   owned projection. Do not call the exponential gate `generateExampleOutput`, or
   substitute the NEI one-expansion example for the machine's eight slots.
3. Bind input choices and outputs to the same observed tuple in the existing UI,
   retaining native positions/background and optimize styling. Then run one small
   source/compiler/browser conformance case before the final unified export batch.

No interim candidate or live export is authorized by this model-only checkpoint.
