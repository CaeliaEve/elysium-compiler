# Native CustomDiagram semantic content (development)

The finite GT adapter reads the existing DiagramGroup matcher and immutable diagrams. It does
not regenerate layouts, invoke `loadDiagrams`, create a recipe GUI, calculate pixel dimensions,
draw labels, resolve component descriptions through third-party display-name callbacks, or call
interaction consumers. These records are read-only associations, not consuming recipes.

`neiDiagramDefinition`, `neiDiagram`, `neiDiagramBinding`, `neiDiagramLookup` and `neiDiagramAction` retain the native
group identity, full logical cell closure (including empty cells), alternatives, optional display
quantities, notes, content labels, tooltips and crafting/usage bindings. Explicit null properties
are mandatory. Wildcard metadata 32767 remains a symbolic item component with a null appearance
fact; it is never assigned an invented meta-zero icon. Concrete item/fluid facts retain their
normal native capture path. Formatting that conveys warnings is retained; font size, pixel
spacing, border/arrow geometry and drawing callbacks are not presentation data.

Circuit recipe, lens recipe and ore stage associations come from the audited factory code and
native insertion keys, rather than pixel arrow geometry. They do not assert inventory consumption
or inferred quantities. The Compiler independently checks these associations against the complete
logical content. Material parts/tools retain their native material-related groups and tool NBT.

ComponentDiagramMatcher bindings are read in native multimap insertion order; the unordered
`all()` set is not used to invent stable order. Circuits include both the complete matcher diagrams
and the separately captured overview action. Only the pinned overview supplier's owned collection
is read; it is not called. Native material matchers are confined to the two audited generator
classes. Their read-only callbacks are queried over the ready NEI item list, registered fluids and
configured components from the diagrams. This query scope is explicit; it is not a claim that
every possible external NBT state was enumerated. Unmatched browse entries are not retained as
unreferenced icon facts. Every group also scans this ready query scope. One lookup per matched
original identity retains raw metadata/NBT and all ordered crafting/usage binding indices. The
normalization policy comes from the actual native `Item.isDamageable()` result and the native
component constructors; the browser never infers it from stack durability. Ignored NBT is retained
in the original fact. Metadata is the native nonnegative int, including 65535; 32767 remains
symbolic. A direct negative matcher key whose display stack becomes zero fails explicitly.

Snapshots own their JSON and specimen copies. Batch checks guard registry identity/readiness;
sealing re-reads the full semantic content and bindings. Cancellation, unknown callbacks,
ambiguous slot ownership, missing registry facts, drift and closed cursors fail explicitly.
Entry, recursion, UTF-8, per-leaf and aggregate byte budgets are checked before accumulating
unbounded content/bindings. Logical slots bind by the native position object's identity; value
equality is accepted only when unambiguous. Coordinates never enter the transport.

The old `neiInteractiveGroup` dimensions-only shape is explicitly rejected by the current
Compiler. Historical archives remain untouched. This is an unpublished development contract:
Source 40 / Catalog 39 with generated TypeScript package 0.40.0-semantic.30, not a unified final
candidate or a pack acceptance result.

Local evidence uses actual installed NEICustomDiagram 1.7.5 APIs through a derived SRG-name test
JAR with unchanged method instructions. The controlled diagram combines alternatives, a 1250 mB
fluid, a wildcard, a condition and navigation. Full Domain and actual Compiler Source/Catalog
validation are separate evidence from schema-only parsing. Specimen names/missing icons remain
fixture scaffolding. No game, installation, formal export or active preview replacement occurs.

The shared `/diagrams` query, strict TypeScript reader and own semantic browser page are connected
online and offline. The complete native leaves retain all alternatives, while only visible cells,
alternative slices and note pages hydrate item/fluid textures. The index retains one selected
group rather than every group indefinitely. Native action destinations are applied explicitly;
the circuit overview is not silently replaced with all recipe diagrams. The strict reader checks
leaf hashes, origin and cell closure, optional quantities, action/binding/lookup provenance, displayed
slices and the exact item/fluid identities. Undefined optional URL parameters are omitted.
The shared index groups bindings by normalized component and validates complete lookup targets,
coverage and per-registry damage-policy consistency. Original fact IDs select those lookups;
responses include the query proof and only selected binding leaves. No all-bindings scan is needed
for each original query, and hidden candidate textures are not hydrated.

Local tests reuse the existing Node reference checks and combined stopped-backend browser journey.
The controlled fixture verifies quantities 7 and 1250 mB, absent quantities, a logical empty cell,
the symbolic wildcard, native conditions, navigation, deep reload and offline reopening. Existing
fuel/firework/lathe/Forestry/information checks stay in the same journey. Fixture icons/names and
the absent native browsing package do not prove pixels, bilingual content or production speed.

Required work still in progress: native overview/material-family runtime coverage beyond the
controlled API fixture. Query lookup
supports the declared ready-list/registry/configured-component scope; it must not be advertised
as enumeration of every possible externally created NBT state. EnderStorage uses its dedicated
owned server snapshot protocol in [ender-storage.md](ender-storage.md), with unloaded persisted
entries, loaded contents and explicit owner/freshness policy. It does not reuse finite GT capture
or invoke native refresh/lazy enumeration. `saveToTag` is not a read-only shortcut: EnderItemStorage
may resize and mark inventory dirty inside that method.
