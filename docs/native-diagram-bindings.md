# Native diagram query destinations

Contracts 0.40.0-semantic.77 stores one `neiDiagramBinding` for each exact native
normalized component and direction. `diagrams` is a nonempty list of unique native
destination indices, in matcher order. It replaces the repeated single `diagram`
field. Each original query still records every binding index, with independently
verified damage/NBT normalization. Binding indices retain first-registration order.

The producer, Compiler and Catalog validate all destinations, full query coverage,
uniqueness, declared counts, sources, item/fluid facts and content identities.
The 32 MiB aggregate byte limit and 262144 relationship limit are unchanged. Grouping
avoids duplicating a large typed NBT component for each destination. Catalog pages
return each matching binding once even when it covers several visible diagrams.

The native controlled fixture uses one 32 KiB NBT query reaching 120 actual Diagram
instances in both directions. The production matcher is exercised without game
startup or GUI drawing. It is local conformance evidence, not full-pack acceptance.

Contracts `0.40.0-semantic.84` allow concrete query components to use
`{"kind":"itemFact","fact":"item_<sha256>"}`. The referenced item retains the
complete registry, physical metadata and typed NBT. Lookup normalization expands
the original fact before applying native damage/NBT rules; semantic duplicates
are rejected even if one component is literal and another is a reference.
Wildcard templates retain their literal component and null fact.

This removes a second duplication: distinct queried tools with large NBT no
longer repeat those compounds in both directions and again in original lookups.
All byte, row and destination limits remain enforced. Online and offline pages
include the original and normalized binding facts, even when those items are
outside the currently visible cells. EnderStorage retains literal stored contents
and rejects this query-only representation.
