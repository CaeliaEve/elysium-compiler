# Galacticraft refinery tank conversion

`galacticraftRefinery` is the Source40/Catalog39 development process for
GalacticraftCore3.3.13-GTNH. The native handler requires this law; a generic
two-tick, guaranteed-output recipe is rejected.

The scope is `fluidTankConversion`. Each operation drains at most1mB of oil and
then attempts to fill the configured `fuel` or `fuelgc`. Native canProcess checks
input volume, output volume and disabled state, but does not ensure the output
tank has compatible fluid/NBT. A rejected fill does not roll back oil drainage.
The fuel quantity is therefore a conditional potential rather than a fixed
result. The one-mB oil debit has no item-container return.

Piped input names must start with case-sensitive `oil`. Canonical oil retains
offered NBT; alternative names normalize to the canonical oil without tags. The
process carries both source and normalized fluid IDs. Native item canisters,
bucket handling and fuel-container transfers are separate machine I/O; this law
does not model those as consumed recipe ingredients.

Each tank holds24000mB. Normal updates need an enabled, powered machine with
positive oil and free output volume. Fresh processing initializes a two-tick
countdown on one startup tick; uninterrupted queued operations use two countdown
ticks. A blocked tick resets progress. The60/90gJ values are maximum extraction
settings under normal/hard mode, not a fixed measured energy debit or wall-clock
duration. No generic duration or energy field is emitted.

The producer reads the real Forge fluids/IDs/names maps and oil/fuel/config roles.
It binds roots and actual member instances, rather than identityHashCode values.
Cheap per-row guards check roots/counts/config; batch and sealing verification
check the complete bounded lookup/fact signature and identity sequence. Unknown
map implementations and fluid hash/equals overrides fail before callback use.
The no-argument Fluid getters used in observation are final in the installed
Forge; they are not polymorphic callbacks. Native FluidStack may publish an
equivalent lazy delegate. Only verified native name/target delegates are adopted;
existing observed delegate replacement or target drift is rejected. Closing a
cursor clears owned references and leaves native maps intact.

Actual installed native smeltItem/fill tests cover alternate tag stripping,
canonical tag preservation, legacy/current fuel, capacity and drain-before-NBT
fill rejection. Complete native updating, container I/O, actual game visuals,
the whole registry, full English coverage and production performance remain
separate acceptance gates. The controlled transport fixtures preserve raw recipe
IDs across Java/Rust/TS and Chinese/English note transport.
