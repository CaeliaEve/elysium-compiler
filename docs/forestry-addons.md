# NEIAddons Forestry information

The development paired Source 40 / Catalog 39 contract includes dedicated
`forestryAddonsDefinition`, `forestryAddonsSpecies`, `forestryAddonsOutput` and
`forestryAddonsMutation` records. NEIAddons 1.17.0's two produce handlers and
four breeding handlers use these reference domains; they are not consuming
machine recipes. No cached NEI recipe, background, slot border or progress
image is created by the producer adapter.

Definitions bind the exact handler, species root, mode, initialized registry,
secret visibility policy, actual member-role type map and complete counts.
Species preserve native names and templates, analyzed native member specimens,
their actual quantities and types, ordinary/specialty output coverage and the
native jubilance provider/description. Mutations retain both parents, offspring,
the complete genotype, conditional descriptions, hidden policy and native order.
Duplicate registrations are retained. A null description collection remains
explicitly null; it does not mean no eligibility conditions exist.

Production records retain original item metadata/NBT, quantities and Float
rates from the native species APIs. They do not inherit the NEI GUI's display
merge: the native utility increments stack count for matching item/rate entries
instead of summing source quantities. Stable role/item/quantity/Float ordering
is used for unordered output maps; native species and mutation enumeration is
retained. Raw Float bits and the native decimal spelling stay together.
Values are nominal species parameters, not predictions of a running machine or
world. Invalid stacks, missing templates, negative/nonfinite values and callback
exceptions fail explicitly. There is no guessed unit, basis-point rounding,
default guaranteed chance or silent failed-read-to-empty fallback.

Compiler validates the typed owner/root/mode, counts/order/closure, template
species binding, all native member roles/items, quantities, Float roundtrip,
specialty coverage and eligibility policy. Pages share one `/forestry-info`
query through the HTTP backend and offline worker. Species overview only
hydrates displayed member facts; products are fetched on selection and paged,
and mutation pages only hydrate visible parent/offspring forms.

The Java conformance test executes installed NEIAddons analyzed-specimen
callbacks using controlled Forestry providers. The actual Java record file is
read by the Rust Domain validator and TS reader, then used to build an actual
Compiler Catalog for browser/deep-link/offline tests. Test providers, fixture
names and absent icons do not prove native mod species coverage, English
content or native pixel acceptance. Whole-pack game operations remain part of
the separately delegated export acceptance.
