# Strict consumer table decoding

The contracts runtime selects a Rust-generated table variant by its declared
kind and compiles each used validator once. Fully tagged oneOf definitions use
AJV's discriminator to select the same generated strict branch. Unknown tags,
additional fields and missing required data remain errors. This avoids validating
records against unrelated domains; no fallback validator or permissive cast is
introduced. The generated schema file itself is unchanged by runtime selection.

Error summaries deduplicate path/message pairs and retain at most six entries,
each at most 256 characters, plus the omitted-detail count. Validation still
rejects the complete malformed table. The limit concerns error presentation,
not how many bad rows are accepted. MessagePack byte/string/array/map budgets
and semantic zero-GUI guards remain in place.

Contracts semantic.44 checks the native transport cases and compiled Catalog
path alongside the existing Node suite. A malformed native process previously
produced a 5774-character union error; the bounded/tagged path identifies its
actual action field. No browser or whole-export speed multiplier follows from
this local validation change.
