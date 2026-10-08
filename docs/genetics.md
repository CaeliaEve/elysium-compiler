# Registered genetics

Compiler/contracts 0.38.0 (Source 38, Catalog 37) represent registered Forestry 4.10.17 bee, tree and butterfly roots and the optional Botany 2.5.24 flower root. These are species defaults and mutation registrations, not player genomes or evaluated world state.

Species identity uses the root UID and species allele UID. Members are fresh native individuals for every form: queen/princess/drone/larvae, sapling/pollen, butterfly/serum/caterpillar, and flower/seed/pollen. Item identities retain the complete native NBT. Fresh individuals avoid queen mating and flower seed/pollen age changes leaking between forms. Genes retain chromosome/allele identities, localized names, dominance and API scalar/vector values; behavioral providers have no invented value.

Bee product and specialty rates are native unmodified fractions per production cycle. The optional `jubilance` string reference is the exact `IAlleleBeeSpeciesCustom.getJubilanceProvider().getDescription()` result. Null provider/description stays null, and formatting/explicit empty descriptions are preserved. Export never calls `isJubilant` or simulates housing/world state. Tree lists have null probabilities and retain default fruit-family compatibility. Butterfly and flower records require empty production lists. Bee and butterfly natural nocturnality is distinct from tolerance genes.

Flower `flower` traits retain native acidity and soil-moisture symbols and the numeric `IFlowerType` ID. Soil moisture is distinct from Forestry air humidity. Flower colors remain native integer chromosome values.

Mutations retain the native parent pair, result species, actual result genome, base probability, localized condition descriptions, secret flag and zero-based occurrence for duplicate identical registrations. The root's `getMutations(false)` preserves native registrations without shuffling. Self-crosses retain both parent entries. Conditions and base rates do not claim current eligibility, research/housing modifiers, or aggregate final probabilities.

The web catalog supports species lists, member/item links, traits, default genes, result genes, origin/cross mutation pages, and localized bee specialty conditions for all four kinds. It does not reproduce BeeBetterAtBees graph layout/navigation or simulate butterfly loot, ecology, planting, pollination, growing cycles, or live breeding. Native NEI inventory and visibility reconciliation and live-game acceptance remain separate work.
