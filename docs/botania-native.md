# Native Botania recipe laws

The pinned source is Botania `1.12.28-GTNH`. Mana pool and runic altar handlers
require `botaniaPool` and `botaniaRunic` processes respectively. They cannot
be published as a generic fixed craft. These records have no NEI view, fixed
duration, energy, crafting grid or fixed English rule properties.

`botaniaPool` retains the native mana cost and the distinct block-below-pool
states `none`, `alchemy`, and `conjuration`. Its input predicate removes earlier
eligible native matches. The first match wins even when the pool lacks mana;
selection does not continue to a cheaper later recipe. A live dropped item,
initialized catalyst state, age outside 101–129 and enough mana are necessary.
On the server one item is consumed without a crafting container; the copied
result receives age 105. Mana settlement follows the native signed integer
update and capacity clamp. The mana parameter is a signed Java int, including
negative values, zero and MIN_VALUE. Comparison, negation and addition retain
native signed 32-bit overflow; negative values are not promised to refill a
fixed amount. No clamp is applied to the recipe parameter itself.

`botaniaRunic` has a packed stable altar-inventory scope, without a cached
recipe override. Its ordered `predicates` use literal registry/metadata
matching; metadata 32767 in an ore predicate means a wildcard. The native
matcher walks physical slots and removes the first remaining matching
requirement. It does not search all permutations. `earlier` contains all
preceding registry predicates, including hidden skull-output entries and
nonpositive-mana entries which still preempt selection. Empty earlier
requirement groups never match; an empty whole requirement list can match the
empty packed inventory. The browser reproduces this selection before showing
the conditional result and allows physical inputs to be reordered.

Altar inputs use slots 0–15, one item per occupied slot; the separately dropped
livingrock requirement uses semantic slot 16. All other altar slots must be
empty. Completion requires a positive initialized mana target, enough mana,
a wand trigger, and live livingrock in the altar bounds. Server settlement
clears every altar slot. Runes are dropped with their actual NBT for a
noncreative player or no player; creative players receive no rune refunds.
One dropped livingrock is consumed. Other inputs and containers are not
returned. The 60 ticks are post-craft cooldown, not total processing time.

Java transports retain output count and NBT. Rust validates handler/process
binding, process literals, candidate predicates, rune consumption and the
separate completion item. The generated schema and TypeScript consumers use
the same structures. Rule explanations are authored separately in Chinese and
English; no English property is relabeled as Chinese. Material calculations
list these outputs as conditional and do not multiply them into guaranteed
production totals.

`floatingFlowers` remains documented in [floating-flowers.md](floating-flowers.md).
It now has the same process-only rule presentation and native-handler gate.
Its non-String final `type` remains an explicit capability gap; the adapter
must not invent compound NBT storage order. Pool/altar stack, nested NBT, ore
member/view and requirement-list identities are bound in the same session.
Unknown stack/NBT storage owners are rejected before copying, and aggregate
identity/NBT/definition budgets are enforced. Broader callbacks, transient
state coverage and the floating reader's nested identities remain under audit.
Tests now use scoped real registries and the sole production constructors;
the collection-injection constructors are retired. Local fixtures and parity
tests are not an installed-pack census or game-export acceptance.
