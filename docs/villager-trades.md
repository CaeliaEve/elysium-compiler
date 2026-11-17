# Native villager trade observations

Mobs Info 0.5.6 publishes trade-list reference data, separate from consuming recipes.
`TradeItem.sizes` preserves the sorted, unique alternatives as signed 32-bit Java
integers. Native PositionedTradeItem assigns these values directly to stackSize;
negative observations must not be clamped, converted to unsigned values, dropped,
or interpreted as refunds. The item fact itself still needs a valid positive
specimen stack. A null native alternative set uses the positive specimen count;
an empty set remains invalid.

Contracts package 0.40.0-semantic.76 exposes this signed domain in JSON Schema.
The frontend displays alternatives unchanged and identifies negative records as
observations, not executable consumption. Native frequency and random enchantment
power retain their existing meanings. Compiler validation still requires sorted
unique alternatives, item facts, complete professions and correctly bound content
identities.
