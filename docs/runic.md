# Runic augmentation

Source 19 / Catalog 18: `process: {kind: "runic", charge: int32}` identifies the
Thaumcraft 4.2.3.5a `InfusionRunicAugmentRecipe` process. It is not a fixed-cost
infusion recipe. `charge`, `magic.aspects`, `magic.instability`, and the amount of
the `pedestals` choice are observations for the supplied central item example.
Consumers must label them as samples and must not use them as prices for another
metadata/NBT variant. The central choice covers the registered IRunicArmor item,
regardless of metadata or NBT. Registered item types are enumerated, not hypothetical
combinations of their NBT. The supplied examples do not establish constant base charge.

At operation start the required state parameter is the actual offered stack's
`EventHandlerRunic.getFinalCharge(stack)`. This is the native
`IRunicArmor.getRunicCharge(stack)` plus `getByte("RS.HARDEN")`, added as a Java
signed int. Implementations may inspect nested inventories (BloodMagic BoundArmour
does); clients cannot derive this parameter by subtracting the example's hardening
or by assuming the item class returns a constant. The web viewer reports the rule
and sample; it does not simulate mod code or accept example costs as actual costs.

For actual final charge `c`:

- Input slot 0 is the central equipment. Slot 1 is one diamond on one pedestal.
  Slot 2 is the repeating Thaumcraft ItemResource metadata 14 group, with
  `1 + max(0,c)` distinct pedestals, one item on each. Different acceptable ore
  alternatives may occupy different pedestals. The serialized positive decimal
  `amount` is the **sample number of pedestals**, not a stack quantity.
  Returns are native container observations per used pedestal, not one aggregate
  container for the group. Actual containers come from each offered stack's native
  getContainerItem call; changing its NBT can change that observation too.
- `v = (int)(32.0 * Math.pow(2.0,c))`, including Java saturation at INT_MAX and
  truncation to zero for tiny positive values. When v > 0, tutamen and praecantatio
  cost v/2 each (integer division), potentia costs v. When v <= 0 the cost list is
  empty. Registered zero-valued aspects are retained when v=1.
- Base instability is `5 + c/2`, using truncation toward zero, including negative
  results. Actual altar instability, hazards, extra essentia use, interruptions and
  world reachability are not a fixed recipe price or a guaranteed completion.
- Research is RUNICAUGMENTATION. No fixed duration or EU/t is asserted.

`change.action: {kind: "runic"}` copies the entire central stack and its NBT,
replacing RS.HARDEN with `byte(getByte(RS.HARDEN)+1)`. Missing/non-numeric tags
read as zero; numeric primitive coercions use Minecraft 1.7.10 conversions, including
float/double floor and narrowing. Byte overflow is intentional: 127 becomes -128.
The ordinary pedestal inventory limit is one. Native infusion replacement preserves
the input count; the output record and transformation samples show one item, not a
promise that an externally overfilled pedestal is normalized. No live stack is mutated.

`rule: {kind: "infusion", template: itemId, ores: sortedNames}` expresses the
pinned matcher for an **untagged** component template. Extra offered NBT is allowed.
It accepts either identical item/metadata, subject to the native stack limit, or an
offered stack whose **first native ore ID** belongs to one of the template's ore
groups. Being in a later group alone is insufficient. `ores` equals the template's
registered group list; it does not specify the order of an offered item's groups.
Exported alternatives are independently checked by the actual native matcher on
copies. The compiler verifies references and group declarations; it cannot prove
first-ID order from a sorted Item.tags list and does not claim to do so. This
predicate is forbidden inside priority exclusions that would require such replay.

The view uses the native infusion renderer with two component-group icons. The
repeated group has an explicit sample pedestal-count label and explanation; it does
not allocate one widget per possible runtime charge. This keeps capture bounded even
at INT_MAX, where the symbolic count is 2147483648. It is a compact symbolic view,
not a screenshot asserting that two pedestals suffice.

Local evidence uses the installed Thaumcraft JAR (SHA256
587b5a084643e617e9d87319ad34b341ebb920e09d060e5a5fbedecb96553f89), remapped only
for the isolated Java test. Tests exercise actual recipe methods, with a synthetic
IRunicArmor provider to cover varying state. No world, player, native crafting loop
or live registry completeness is asserted. Unified live acceptance remains required.
