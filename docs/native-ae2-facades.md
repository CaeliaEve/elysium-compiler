# AE2 facade texture metadata

Source 40 / Catalog 39 development contracts add optional
`ae2Facade.textureMetadata` (contracts package 0.40.0-semantic.38). It records
native item-to-block metadata normalization when the output differs from the
selected texture's metadata. Without the field, output metadata equals input
metadata, preserving existing facade records.

A Galacticraft enclosed-item observation demonstrates a real non-idempotent
mapping: the shown center texture has metadata 0, while native
ItemFacade.createFacadeForItem produces canonical facade NBT x[1]=4. The input
entity and facadeTexture admission rule remain unchanged. Output NBT must still
use the selected texture registry's modid/itemname and the declared effective
metadata; negative and wildcard effective metadata are rejected.

The actual Java sample is `contracts/fixtures/ae-facade-normalized-contract.json`.
The existing native transport runner validates it and rejects modified
normalization values. The shared TS decoder retains the field; NeoNEI uses the
ordinary output item identity/icon. This is local native API conformance, not a
whole-pack pixel, English-content or game-session acceptance result.
