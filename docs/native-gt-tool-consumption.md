# Native GT crafting tool consumption

Development Source 40 / Catalog 39 and contracts `0.40.0-semantic.35` add
`consume.kind = gtTool` for the native shaped/shapeless crafting reader.

`cost` is the installed `getToolDamagePerContainerCraft` constant in GT native
units; it is not vanilla item durability. `owner` identifies one of the 24
reviewed constant callback owners in GT 5.09.51.482. The compiler checks their
exact constants. Both installed broken-result owners return null.

The native container copies one tool and keeps it in the crafting grid.
Physical tools add the cost to their signed-long damage. At the native maximum,
the tool disappears. Electric tools first attempt native energy use; only after
successful use do they draw `nextInt(0,25)`. A zero draw adds wear, with the same
break condition. Failed energy use retains the tool. Charge changes can also
change tool metadata and normalization. No export-time RNG is advanced.

`sample = {electric, damage, limit}` describes the captured candidate only. The
compiler derives these fields from its typed `GT.ToolStats` compound using the
native numeric accessors. Missing or nonnumeric values follow native defaults;
signed values and numeric narrowing are preserved. This sample does not predict
an arbitrary player's tool state. The actual branch depends on offered NBT.

The choice has amount one and no fixed ingredient returns. NeoNEI lists one
reusable tool, explains energy and conditional wear, and does not multiply tool
count by batches or promise a fixed number of replacement tools. The new note
helper has Chinese and English text; this does not establish complete bilingual
UI or native language coverage.

The Java proof uses installed method bodies with controlled registration and
statistics. Two actual-adapter rows cross the full Rust gates and Catalog/API/
strict frontend decoder. Native item pixels, names and whole-pack tool coverage
still require the subsequent authorized game acceptance.

The separate GT result-stage helpers bind callbacks and compare concrete owned
inventories against the original native result callback. The full GT multi-input
result process and candidate linkage are not yet connected to production capture.
