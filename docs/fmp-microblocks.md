# ForgeMultipart native microblock rules

The Source 40 / Catalog 39 development contract (`0.40.0-semantic.52`)
represents the installed ForgeMultipart 1.7.2 `MicroRecipe` as shared rules and
bounded witnesses. It does not serialize NEI geometry or stone-only shaped recipes.

A category's `program` and its `fmpMicroblock` process refer to the same program.
The process contains `program`, `branch`, `materialId` and nine physical `layout`
cells. The shared program has five chunked sections: `fmpMaterials`, `fmpNames`,
`fmpAliases`, `fmpSaws` and one `fmpSettings` row. Existing chunk offset, size,
whole-program identity and reference gates apply. Native registry order and the
first full-block match are significant; the saw maximum is the stored value from
the last native calculation, not a recomputation over the current registry.

`fmpMaterial` predicates contain the program, target material ID and, when `mat`
is present, its observed `nativeText`. Native lookup makes one alias hop, then
looks in the name table; missing compound, missing `mat` and unknown names resolve
to ID zero. A finite alias list cannot express this predicate. String projections
must equal their tag. Non-string tags retain Java's native NBT text observation:
sorted typed compound NBT cannot reconstruct native HashMap iteration order.
Missing `mat` must have no projection. This observation is not guessed from JSON.

The full domain gate evaluates hollow, gluing, thinning, splitting and hollow-fill
in native priority and checks every output witness. Counts, metadata and discarded
or preserved NBT remain native facts. The built-in saw returns a fresh count-one
stack with damage plus one only when its item is damageable; otherwise it returns
the offered stack unchanged. The installed ProjectRed gem saw preserves count and
NBT and adds one damage. Workbench breakage handling remains a separate operation.

The shared online/offline query caches each native program and its lookup index.
Page responses hydrate the selected material block and used saw policies, with a
bounded preview and full count of known names. They do not repeat all material and
alias rows in every recipe. Physical layouts are witnesses, not fixed admission
patterns for every branch. These local fixtures do not establish whole-pack pixel,
animation, English-content or production latency acceptance.
