# Inventory-dependent arcane recipes

This batch adds `arcaneSpecial` and `stickyJar`. Implementation and contract
generation have not yet been validated together. These notes are not an export
acceptance report.

`arcaneSpecial` binds a native class, operation, count branch, item roles and
the native aspect-tag registry. Recipe input/output records are valid specimens;
they do not enumerate every admissible inventory or guarantee a world execution.

- `primalCharm`: exactly six distinct shard metadata values 0–5 occupy outer
  cells 0, 1, 2, 6, 7, 8. Gold is in 3/5 and shard 6 in the center. Costs 25 of
  each primal aspect. This is not nine independent ingredient alternatives.
- `photoDeveloping`: one tagged plate at metadata 9, one to four paper cells
  and the same number of black-dye cells, with no extra items. Paper is tested
  before ore membership. Result metadata 10 copies the plate NBT, with one
  output per paper cell and five AIR/WATER/ORDER per paper cell.
- `nodeAttunement`: one matrix and one to eight tagged essence phials. Each
  phial must parse an aspect entry. Its duplicate entries add before dividing
  the amount by four, truncating toward zero. Signed int wrapping, zero and
  native null-aspect behavior are retained. Contributions then add the matrix's
  existing aspects; the result retains matrix count and other NBT, rewriting
  its `Aspects` list. Native matching stores a result cache and the result
  callback returns the last successful cached copy. Costs two of each primal.

`stickyJar` binds the real Gadomancy matching registry, including entries with
no GUI recipeStack. A nonsticky jar and a slime ball immediately below it form
a translating 1×2 relation in the workbench, with every other cell empty. The
native boolean test accepts an existing false/non-numeric `isStickyJar` tag.
The output copies the jar, writes byte 1 for that key and sets count one.
GUI-only `RecipeVisualStickyJar` cycling is excluded with an explicit reason;
its clock-driven update is not called to obtain executable recipe data.

The page and material tools retain execution conditions and variable results,
with separate Chinese/English explanations. Research, wand/creative payment,
earlier workbench registrations and external execution callbacks remain distinct
from an inventory specimen. Canonical NBT is not used to invent stringification
order for arbitrary non-string compound keys.
