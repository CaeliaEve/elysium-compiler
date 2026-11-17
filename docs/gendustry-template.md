# Gendustry template crafting

The current Source 40 rule reader uses the registered, stateless `GeneRecipe` in
`CraftingManager`. A NEI `TemplateCraftingHandler` cache is not a source of
crafting rules or proof of a successful result. An initialized registry without
that recipe is a legal empty set; an unavailable registry remains a failure.

The process retains the native domain of one template and 1–8 compatible tagged
samples. Matching scans columns before rows, and the last sample for each
chromosome replaces earlier samples. The native result copies the offered
template's count, metadata, and unrelated NBT. The two blank sample items in the
local transport illustrate roles; they are not successful crafting observations.

Each occupied slot consumes one item. The sample item inherits Forge's native
crafting-container callback and returns one `gendustry:GeneSampleBlank` per
sample. The template has no container. The prior local fixture omitted these
returns. It is preserved as evidence and rejected by the corrected native domain
gate, rather than silently reinterpreted as a valid new source.

No schema shape changes are needed for this correction: the existing input
`returns`, typed process, source notes, and dynamic result fields carry the rule.
Java, Rust, and TypeScript must agree on the actual producer identity and on
the difference between role examples and observed results. The web page shows
the role examples and native conditions without inventing a successful result;
the material panel does not assign fixed two-sample costs or guaranteed returns
to an unevaluated dynamic craft.

The local two-language transports use an empty species-root fixture. Their
compiled/browser checks do not prove installed-pack genetics callbacks,
complete gene-domain ownership, native item pixels, animation, production
latency, or readiness for a unified game export. Those broader Gendustry and
release checks remain open in the implementation work list.
