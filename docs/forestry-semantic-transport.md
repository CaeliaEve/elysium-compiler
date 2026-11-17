# Forestry GUI-free transport conformance

Contracts `0.40.0-semantic.80` correct the generated Fabricator plan-use schema:
both `keep` and `wax` carry `waxPlans`; `wax` also carries `maxDamage`. Rust
serialization already used these camelCase names. Explicit field renames now
make schema generation agree with serialized Java-owned program records.
Snake-case alternatives are not accepted. Source and Catalog remain revision 40.

Six targeted Java fixtures cover bottler, dynamic bottler, fabricator,
fermenter, moistener and carpenter. Their `-semantic-contract.json` names keep
them distinct from earlier fixtures. They retain real Forestry handler/category
provenance, GUI-free recipe records, source-bound Text notes, shared program
chunks and the original Java recipe identity. No ID rewrite or generic source
substitution is required for Compiler validation.

The Rust conformance test checks the original recipe ID and the complete Domain
with those dependencies. The published MessagePack decoder separately checks
the recipes, strings and program tables. NeoNEI's catalog, backend and frontend
consume the same packaged contract version; historical revision readers remain
separate. These are local transport checks, not installed-pack recipe coverage,
native pixels, bilingual content or live export acceptance.
