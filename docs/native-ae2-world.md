# AE2 world-action matching

Current Source 40 / Catalog 39 development contracts use ae2EnderDust for the
singularity action, introduced in @elysium/contracts 0.40.0-semantic.39. The native
EntitySingularity call is itemMatches(offered, registered, false), so a same-item
input is accepted if its metadata equals the registered metadata or is 32767.
NBT and registered count are ignored. This is intentionally a dedicated native
predicate, not generic ore membership or an expression interpreter.

The rule is restricted to native AE2 world actions and dustEnder/dustEnderPearl.
Ordinary templates must bind their concrete display metadata. A registered
wildcard can use a concrete display representative, but that representative gets
no positive use edge unless the predicate accepts it. Output acquisition remains
conditional on world actions and source stock.

Current ae-world-locale-0..8-en_US/zh_CN and ae-world-ender-notes transport fixtures
are actual Java records with canonical producer category/recipe IDs. The older
ae-world-0..8, ae-world-semantic-0..8 and ae-world-ender-wildcard files remain
historical snapshots; they are not the current locale contract.
ae-world-ender-matching.json contains 24 observations of the installed
native itemMatches implementation with the actual argument direction, metadata,
NBT and count combinations. Native matcher parity, complete domain validation,
strict frontend decoding and the shared Catalog/API path are checked locally.

Contracts semantic.44 removes localized process.description from the unreleased
development chain. Native prose is required in a stable `ae2:world/<material>`
note with the exact GuiText source key(s), and its localized string record is
validated normally. Neither note text nor its translated reference participates
in recipe_id; actual action/input/output rules remain in that hash. Missing or
misattributed notes and reinjected localized process fields fail explicitly.
Native English/Chinese resources produce nine matching recipe IDs. The shared
online/offline API hydrates all explanations and the existing frontend notes
component consumes them. This is a local native fixture boundary; full language
bundles, live whole-pack content and final protocol/release alignment remain open.
