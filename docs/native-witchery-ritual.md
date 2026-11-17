# Witchery ritual action dispatch

Implementation status: unverified batch; no new contract release or runtime acceptance yet.

`witcheryNativeRitual` reads Witchery 0.24.1's original ritual recipe arrays. The constructor's `expandedRecipes` list is a display projection and is not the runtime matcher. The prior `witcheryRitual` reader has a historical-only boundary to be finalized with the unified protocol release.

Physical inputs dispatch through the actual `Hashtable<BrewItemKey, BrewAction>` buckets, stored entry hashes, chain order, identity, and either-side metadata-32767 wildcard. The extractor reads those JDK entries directly; it does not create the map's lazy entry-set wrapper. Query hashes use the observed no-argument item unlocalized name, Java UTF-16 hashing and signed-int wrapping. They are not derived from registry IDs or translated labels. A mutated key's current hash is not substituted for its stored entry hash. A trigger binds the actual registered action instance, even if another action has an equal logical key.

Original recipe requirements compare each dispatched action's item and metadata literally, ignoring NBT and stack size. One offered action removes one requirement, including duplicate requirements. The first satisfied original definition wins; extras are allowed and an empty original pattern immediately matches. Source keeps the original pattern order and exposes only owned real content specimens. A reachable action without a safe specimen is an error, not a fabricated exclusion. Permanent first-match shadowing is excluded only with multiset coverage proof.

The process scope is explicitly `postAdmissionActions`. It models `getRecipeResult` on that action list; it does not claim that every displayed input sequence passes `canAdd`, nullification, world callbacks, cauldron capacity, heat or altar checks. The trigger power is distinct from total brew power. The world-item output is a conditional potential specimen, and material planning must not multiply it as a guaranteed yield or fixed input debit.

Source bindings, item examples and query names are checked before sealing. No brew-key constructor, world mutation, action callback or native registry mutation is invoked during extraction. Unknown name callbacks require an explicit adapter. Original raw JARs and archived evidence remain unchanged. Static source/bytecode inspection is not runtime proof.
