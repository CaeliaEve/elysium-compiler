# Railcraft literal metadata sentinels

Source 40 / Catalog 39 keep the existing `metadata` match shape. Contract
package `0.40.0-semantic.37` explicitly permits `value >= -1`, retaining -1 as
the only valid negative literal. This represents the actual pinned Railcraft
InvTools wildcard entry condition and keeps the rendered item concrete.
Values below -1 are rejected by both domain validation and generated schema.

This is exact literal metadata matching, not a general wildcard operator.
Registry identity, optional native NBT equality and absent-root-tag rules are
unchanged. A concrete display sample does not gain a usage link merely because
it illustrates a -1 or 32767 branch. Ordinary visual item identity and native
negative durability remain separate contracts.

The actual Java Railcraft row is `contracts/fixtures/native-rail-literal-wildcards-contract.json`.
The native transport conformance suite, matching tests, shared decoder and an
actual CLI-produced Catalog validate it. No game export, native pixel or full
pack acceptance is implied by this local fixture.
