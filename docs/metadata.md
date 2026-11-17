# Literal metadata and display examples

Source 37 / Catalog 36 add the item-only `metadata` match rule:

```json
{"kind":"metadata","value":32767,"nbt":false,"absent":["synthetic"]}
```

Registry and reference NBT come from the choice's concrete item. Raw metadata
must equal `value` (a nonnegative i32, or the native Railcraft `-1` sentinel),
independently of that example's metadata. Values below `-1` are rejected.
`nbt: true` ignores NBT; false compares it exactly, including null versus empty.
Every key in `absent` must be absent from the offered root compound, regardless
of its stored type/value. Keys are sorted, unique and bounded like tag rules.
This is an independent predicate and can appear in existing priority exclusions.

Railcraft accepts a literal 32767 offered stack for a concrete subtype recipe.
Exporting that synthetic stack as an item breaks AmunRa name/icon callbacks.
Use the real concrete input for display and retain 32767 explicitly in the rule.
Replacing it with an unrestricted wildcard or the display metadata would change
matching and two-pass recipe priority. Direct wildcard templates continue using
the existing wildcard/tag rules with a concrete NEI example.

The display is not necessarily an accepted input for this individual branch.
Do not generate a positive usage link from the display-only branch; another
concrete alternative may still provide the valid usage link. Recipe identity
includes the rule. The existing MessagePack tables carry it without a new table
or additional runtime interpreter. Older source/catalog revisions are rejected.

Contracts `0.40.0-semantic.84` add two explicit item predicates within revision 40:

* `metadataSet` carries sorted unique `values` (at most 65536), `exclude`, and
  `nbt`. It compares physical metadata in the same item registry. `exclude`
  selects the complement of that set. `nbt: false` retains exact native tag
  equality. Et Futurum partitions may merge metadata only when result,
  disposition and native XP bits are identical.
* `nativeItemMetadata` carries `callback: "botaniaManaMirror"`, signed `value`
  and `nbt`. It applies only to `Botania:manaMirror`; all fact IDs and pixels
  retain physical metadata. The getter is
  `1000 - (int)((float)mana / 1000000.0f * 1000.0f)`, including two float
  operations, Java narrowing and wrapped subtraction. GT matching and overwrite
  stages use this logical value while preserving the offered stack's tags.

These are matching contracts, not replacement item facts or sampled NBT values.
Unadapted dynamic getters remain errors in readers without this contract.
