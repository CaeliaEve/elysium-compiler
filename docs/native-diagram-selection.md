# Native diagram selection declarations

Development Source 40 / Catalog 39 records preserve the entire finite diagram closure,
raw bindings, original query normalization, and group action destinations. Selection
preferences never remove captured documents or become a GUI capture gate.

Each definition includes `selection`: `showEmptyDiagrams`, the audited `empty`
predicate (`never` or `slotInsertionsBelow` with a bounded `minimum`), the native
`defaultVisibility`, nullable original `configuredVisibility`, and effective
`visibility`. Each diagram includes `nativeSlotInsertions`, the exact native
multiset size. It is independent of logical cell and alternative counts; a direct
Diagram constructor can have useful contents and zero recorded insertions.

The pinned NEICustomDiagram 1.7.5 has seven visibility values: ALWAYS_SHOWN,
ALWAYS_HIDDEN, SHOW_ON_SHIFT, HIDE_ON_SHIFT, SHOW_ON_CTRL, HIDE_ON_CTRL, DISABLED.
Its old config `true` maps to ALWAYS_SHOWN and `false` to DISABLED; unrecognized
config strings map to ALWAYS_SHOWN. The original value is retained. Compiler and
shared reader validate the effective value against this declaration.

Java identifies the actual default and DiagramUtil lambda classes without running
the predicate. Unknown predicate callbacks fail explicitly. Existing configuration
is read without creating properties, migrating legacy values, saving files, or
probing keyboard state. Changes invalidate the reader; world/mod/locale identity
remains subject to the surrounding capture protocol.

NeoNEI defaults to the complete reference content. `selection=native` applies the
native empty rule and query visibility with explicit `shift` / `ctrl` booleans.
The native empty rule applies to all entry points. Group and custom overview actions
bypass visibility, matching DiagramGroup/CustomDiagramGroup. Item/fluid queries
apply visibility. Online, offline worker, and strict frontend reader use the same
pure selector. These preferences are presentation rules, not consumption semantics.

Controlled actual API tests and local compiled samples are distinct from validation
of all five GT generators and running-world capture. Language and native pixel /
animation / production throughput acceptance require separate evidence.
