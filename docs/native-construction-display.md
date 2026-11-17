# Native construction and display snapshots

Contracts `0.40.0-semantic.79` retain two distinct construction-only definitions:
`construction` is a registered StructureLib compatibility provider; `gtconstruction`
is the pinned GT cleanroom constructor without an enumerable structure definition.
Neither invents pieces or establishes that the machine's gameplay conditions pass.
Both retain actual placed blocks, tile NBT, requested parameters and orientation.

The Botania `1.12.28-GTNH` enchanter requires an explicit completion boundary.
Its registered `BotaniaMultiblockInfoContainer` places native `StructureLibFlower`
display blocks. The installed enchanter's `canEnchanterExist` does not recognize
these as ordinary Botania flowers. Ticking that controller destroys the native
display controller and replaces it with lapis; replacing the flowers would instead
invent a different construction.

NESQL therefore captures this exact provider/controller's display geometry before
its controller gameplay tick. Other owned tiles still tick, and all ordinary
providers retain normal controller integrity checks. The build carries optional
`completion: "nativeDisplayBeforeControllerTick"`. Compiler validation requires
the exact Botania provider, enchanter item at metadata zero with no NBT, native
tile source key, construction-only definition and north-facing orientation.
The field is rejected for unrelated providers and required for this one. An
omitted field on other providers preserves their existing content identities.

NeoNEI explains this boundary separately in Chinese and English: this is a native
display snapshot, not a claim that the enchanter has formed. Item/fluid visuals
and model capture remain subject to their normal independent validation gates.

The scoped Java test uses the installed Botania block constructors, tile class
and registered provider in an owned preview. Its 44 placed cells and tile NBT
serialize into `botania-enchanter-structure-contract.json`; Rust validates the
actual geometry/content identities and consumer tests decode it strictly.
The fixture's tile-name registrations are isolated test setup, not game evidence.
It has no models (`rendered: false`). This local data-profile result does not
replace live full-profile rendering or export acceptance.

Contracts `0.40.0-semantic.84` separately admit
`completion: "nativeDisplayBeforeUnavailableTileTick"` for the pinned Railcraft
9.16.33 / StructureCompat 0.7.4 rock-crusher provider. A disabled native
`ROCK_CRUSHER` tile deletes itself on its first update. The owned display uses
the native creative constructor and captures before that tile update, after
checking native availability and the exact machine-alpha metadata-15 controller.
It does not change the game configuration or claim an operational machine.
Enabled instances keep ordinary completion; other providers cannot claim this
boundary. Chinese and English UI explanations identify this limitation.

The native Blood Magic 1.7.52 tier-one altar is also a valid single-controller
construction. Its survival constructor returns -1 after placing that one block;
this exact provider/tier result is accepted without relaxing other providers'
empty-construction checks.
