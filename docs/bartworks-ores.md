# Bartworks mineral references

Source 40 / Catalog 39 represent Bartworks ores as non-consuming typed reference
leaves: `bartworksOresDefinition`, `bartworksOreLayer`, `bartworksOreMaterial`,
and `bartworksOreLookup`. Native GUI drawing and transfer rectangles are absent.
The registered handler identity is its class name; `quickanddirtyneihandler` is
a transfer action, not its stable source identity.

The adapter targets the installed GT5 5.09.51.482 Bartworks classes. Enabled
generators are read in the native active-list order. Disabled generators can
still appear in the metadata lookup; these remain explicit, with null active
ordinal. Both Ross generators retain dimension ID, translation key, native name,
provider policy and replacement block/metas. A native null replacement block is
retained as unavailable, never replaced with a guessed block. Unknown generator
subclasses fail. Runtime dimension memoization is not generation configuration.

All four layer roles retain signed metadata, GT/Bartworks ownership mask and
native specimen. These are mineral layer examples, not guaranteed drops.
Public int fields preserve their actual initialized values; the installed
constructor narrows its numerical inputs to short, including size after max(1).
The enabled weight total is checked with Java's integer addition semantics.
It is not normalized into a world-generation probability.

Werkstoff ordinary and small-ore samples remain separate. The installed full
overlay uses the *ordinary ore feature* predicate for both variants, then
deduplicates solely by layer. The contract preserves that display policy,
both actual feature flags and nullable native samples without inheriting the
GUI's lossy deduplication. Registry ordinals and material identity remain.

Metadata lookup entries preserve native member order and multiplicity. The
native map is keyed by signed short metadata, without material-owner type.
Compiler checks complete membership against every layer role, including repeated
roles and metadata collisions. Neither missing members nor invented outputs
are accepted. Material and layer item facts must exist with matching metadata.

Readiness requires Forge AVAILABLE and initialized Werkstoff prefix policy;
ready empty registries are distinct from unfinished initialization. Batch
checks guard sizes cheaply; sealing recomputes the complete semantic snapshot.
NeoNEI reads only public compiled Catalog leaves through shared HTTP/offline
queries and hydrates current-page native specimens.

Local tests use actual installed constructors/getters with explicit vanilla
replacement specimens. They do not prove pack initialization, Ross world
generation, localized pack coverage or native textures in the game.
