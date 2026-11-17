# Native item documentation

Source 40 / Catalog 39 development contracts represent `InformationHandler` and Extra Utilities
`InfoHandler` as read-only `neiInformationDefinition` / `neiInformationPage` records. They are
not inputs or outputs in the recipe graph. New datasets contain no documentation GUI screenshots,
font wrapping, native viewport dimensions or decorative layers.

The definition proves native readiness and counts registrations/scanned items, pages, listed
pages and item links separately. Registered NEI pages with no matched items are retained and
marked `listed: false`; zero pages produce a ready definition rather than an absent domain.
Each link preserves quantity and native order, including duplicate links. Extra Utilities
uses the complete ready NEI item catalogue and its installed `isValidItem` predicate. Its
specific-key-before-base-key and single-key-before-sequential-paragraph policies are preserved.
Only its display quantity is normalized to one, as in the installed native constructor.

Text sources retain keys, actual `StatCollector` translation results and translation availability.
NEI's literal `\\n` expansion and actual paragraph breaks are preserved without font-specific
wrapping. Extra Utilities text is read before its native GUI pagination; the native font constructor
and `splitString` are not invoked. Null `precise` / `filterOwner` values are mandatory properties,
with the same presence/null semantics in serde, JSON Schema and the frontend reader.

Compiler checks complete order/counts, source/category ownership, item/quantity closure and exact
paragraph reconstruction. It rejects missing pages, altered paragraphs, illegal quantities and
unready success. NeoNEI `/information` uses the same bounded query online and from offline storage,
hydrates only the current page of visible item links (eight per documentation entry by default),
and exposes search/source filtering and optional unmatched pages. Full native link records are
retained for integrity checks; pagination does not truncate semantics or silently omit quantities.

The local fixture uses native registration and installed Extra Utilities predicate code with an
isolated language map. An unused font-only class initializer is disabled in the test environment;
the predicate body is unchanged. Fixture names and missing item icons are not full-pack language,
native pixel or live-game evidence. Formal language collection remains an Export Agent task.
