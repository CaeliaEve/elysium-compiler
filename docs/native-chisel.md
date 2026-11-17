# Chisel native lookup and pickup

The pinned producer reads Chisel 2.16.15-GTNH and ModTweaker 0.12.0 carving fields without constructing recipe GUI state. Ore IDs use native integer HashSet order, named groups use HashMap encounter order, and fallback lookup uses the audited native map, including tree bins. A stale fallback group can remain reachable after the native name table is cleared.

`process:chisel` separates single pickup from whole-stack pickup. Single pickup consumes one offered item and copies the selected native target count and NBT. Whole-stack pickup uses the offered stack count and discards target NBT. Tool wear and returned tool state remain conditional on native configuration, durability, creative mode and enchantments. Fresh tool samples are witnesses, not unconditional returned items.

`nativeWildcard` identifies the default metadata partition. Metadata32767 by itself is a literal value in the carving lookup; an exact32767 partition is valid. Explicit metadata partitions take precedence over the default partition. The producer observes block metadata and item metadata separately and retains ore targets with their original counts/tags. Current production observations reject negative or out-of-scope metadata rather than silently changing it during ItemStack.copy.

The shared query must hydrate the selected original target, every source partition including exclusions, and all tool policies. They can be absent from the visible input/output choices. In whole-stack mode, the original tagged target differs from the untagged output witness.

Actual Java records in `contracts/fixtures/chisel-*-semantic-contract.json` cross the raw identity and complete domain gate. The literal-metadata fixture also verifies that metadata0 is rejected when only exact32767 is admitted. Source40/Catalog39 schema shape is unchanged.
