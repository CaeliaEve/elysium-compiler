# Native usage observations and documents

Compiler 0.40.0-semantic.5 owns the closed `usageDefinition`, `usageObservation`,
`usageDocument` and `usageResource` reference variants. The generated SDK is
0.40.0-semantic.71; Source and Catalog stay at revision 40. Old SDKs do not
understand these new tagged variants and must not be used for a new dataset.

The four pinned backends are `aeItemCell`, `aeFluidCell`, `ocManual` and
`ocCallbacks`. Definitions identify the exact native source/version, locale and
`catalogObservations` scope. Counts partition scanned inputs into distinct
queries and duplicates, and distinct queries into matched/unmatched inputs.
None of these observations consumes a recipe input.

The Compiler validates content-addressed domain/row/document/resource identities,
native row order, closed item/fluid/document links, signed 64-bit storage counters,
positive long entry amounts, PNG encoding/digest/dimensions, explicit nullable
provenance, redirects and marked native language fallback. Definition locale must
equal the Source environment. Callback documentation retains annotation text and
metadata without implementing an API execution service.

Manual PNG resources travel inside the references table as bounded original bytes
with a SHA256, location and dimensions. They are decoded and validated before
publication, then served through the same immutable Catalog used online and by the
offline worker. No native recipe GUI images, slot frames or progress images are added.

The actual-Java fixture exercises both cell channels, Chinese and English manuals,
and native EEPROM annotations. Its display facts in local integration specimens
are synthetic and are not evidence of a completed live export.
