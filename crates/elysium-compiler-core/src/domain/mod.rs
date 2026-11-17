//! Shared game facts. Source and catalog use these meanings without compatibility aliases.
mod change;
mod check;
pub(crate) mod images;
mod native_crafting;
mod gt_tool;
mod native_container;
mod native_item_metadata;
pub mod gt_crafting;
pub mod gt_probability;
mod crafting_nbt;
#[cfg(test)]
mod native_contracts;
mod floating;
pub mod botania;
pub mod botania_cache;
pub mod botania_cache_machine;
pub mod botania_bee;
pub mod botania_selection;
pub mod botania_session;
pub mod botania_program;
pub mod botania_pool;
pub mod botania_world;
pub mod forestry;
mod genetics;
mod grinding;
mod harvest;
pub mod blood;
pub mod blood_calcinator;
pub mod inventory_process;
pub mod blood_orb;
pub mod imbuing;
pub mod native_brewing;
pub mod arcane_special;
pub mod infusion;
pub mod infusion_merge;
pub mod infusion_ingredient;
pub mod runic;
pub mod salis_wand;
pub mod matter_manipulator;
pub mod infusion_disguise;
pub mod binnie;
pub mod binnie_genetics;
pub mod chisel;
pub mod distillery;
pub mod creativecore;
pub mod ae_crafting;
pub mod wireless_terminal;
pub mod infusion_cost;
pub mod projectred;
pub mod enchanter;
pub mod gendustry;
pub mod gendustry_fluid;
pub mod bottler;
pub mod fabricator;
pub mod forestry_processing;
mod industry;
pub mod integration;
mod magic;
mod matching;
pub mod ae_world;
pub mod extrautils_soul;
pub mod decay;
pub mod decay_state;
pub mod solar;
pub mod circuit;
pub mod refinery;
pub mod banner;
pub mod lootbag;
pub mod space_acquisition;
pub mod crop_prediction;
pub mod cal_imprint;
pub mod fmp;
pub mod witchery;
pub mod native_oven;
pub mod native_ritual;
pub mod witchery_spinning;
pub mod witching_spinning;
mod model;
mod ores;
mod process;
pub mod programs;
pub mod visual;
pub mod fmp_program;
pub mod xu_microblock;
pub mod references;
pub mod cropsnh;
pub mod forge_registry;
pub mod projectblue;
pub mod ic2_advanced;
pub mod structure_references;
pub mod aspect_combinations;
pub mod item_aspects;
pub mod furnace_fuels;
pub mod fireworks;
pub mod lathe;
pub mod lexicon;
pub mod mob_drops;
pub mod infernal_drops;
pub mod draconic_reactor;
pub mod bee_tree;
pub mod forestry_addons;
pub mod nei_information;
pub mod usage_queries;
pub mod nei_diagrams;
pub mod ender_storage;
pub mod underground_fluids;
pub mod gt_minerals;
pub mod bartworks_ores;
pub mod betterquesting;
pub mod villager_trades;
pub mod tool_materials;
mod quantity;
mod rolling;
pub mod extreme;
pub mod rail_blast;
mod soul;
mod structure;
pub use change::{Change, Edit, Stack};
pub use genetics::*;
pub use grinding::{GrindingBall, GrindingRequirement};
pub use industry::*;
pub use magic::*;
pub use model::*;
pub use ores::*;
pub use process::{HarmonyMode, HarmonyOutcome, Process, VatConsumption};
pub use quantity::{quantity_bounds, Quantity};
pub use rolling::CraftingSelector;
pub use soul::{SoulFilter, SoulSelector};
pub use structure::*;

use crate::identity::Nbt;
use crate::source::{Source, CORE_COLLECTIONS};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub owner: String,
    pub handler: String,
    pub key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub registry: String,
    pub meta: i32,
    pub nbt: Option<Nbt>,
    /// Reference to a localized, Minecraft-formatted string.
    pub name: String,
    pub tooltip: Vec<String>,
    pub stack_limit: u32,
    /// Raw ItemStack.getMaxDamage() result; negative values retain mod-specific meaning.
    pub durability: i32,
    pub tools: BTreeMap<String, i32>,
    /// Actual ItemArmor classification; durability alone does not identify armor or tools.
    pub armor: bool,
    /// Exact OreDictionary membership reported by the game.
    pub tags: Vec<String>,
    pub icon: Option<String>,
    /// NEI order; null denotes an item discovered only through a recipe.
    pub order: Option<u32>,
    /// Thaumcraft's returned object tags; null means it provided no value, not zero aspects.
    pub aspects: Option<Vec<AspectAmount>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Fluid {
    pub id: String,
    /// The actual global Forge fluid key, which need not contain a namespace.
    pub registry: String,
    pub nbt: Option<Nbt>,
    pub name: String,
    /// Kelvin.
    pub temperature: i32,
    pub density: i32,
    pub viscosity: i32,
    pub luminosity: u8,
    pub gaseous: bool,
    pub icon: Option<String>,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Item,
    Fluid,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub kind: Kind,
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    /// Signed-64-bit count/millibuckets. Zero requires a native fluid policy; Forestry has native signed values.
    pub amount: String,
    pub consume: Consumption,
    pub returns: Vec<Remainder>,
    pub rule: Match,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Remainder {
    pub kind: Kind,
    pub id: String,
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Match {
    /// Display anchor only; the ordered shared pool session owns admission.
    #[serde(rename="nativePoolInput")]
    NativePoolInput,
    #[serde(rename="matterManipulatorInput")]
    MatterManipulatorInput,
    #[serde(rename="matterManipulatorUpgrade")]
    MatterManipulatorUpgrade,
    #[serde(rename = "nativeZeroFluid")]
    NativeZeroFluid { policy: NativeZeroFluidPolicy },
    #[serde(rename="fmpMaterial",rename_all="camelCase")]
    FmpMaterial {program:String,material_id:i32,#[serde(default,skip_serializing_if="Option::is_none")]native_text:Option<String>},
    #[serde(rename="calInput",rename_all="camelCase")]
    CalInput {role:cal_imprint::Role,#[schemars(length(min=1,max=1000))] imprint_ids:Vec<i32>},
    #[serde(rename="bannerInput",rename_all="camelCase")]
    BannerInput { max_patterns:u8 },
    Exact,
    #[serde(rename = "facadeTexture")]
    FacadeTexture,
    #[serde(rename = "ae2CrystalSeed")]
    AeCrystalSeed { start: i32, end: i32 },
    /// EntitySingularity passes the offered stack first to itemMatches(false).
    #[serde(rename = "ae2EnderDust")]
    AeEnderDust { ore: String, #[schemars(range(min = 0))] metadata: i32 },
    /// Native oven admission partitioned by the offered stack's Genome tags.
    /// The display anchor may belong to another partition; it is not a witness.
    #[serde(rename = "witcheryOvenNbt")]
    WitcheryOvenNbt { meta: bool, product: witchery::OvenByproduct },
    #[serde(rename = "chiselSource")]
    ChiselSource { meta: bool, nbt: bool },
    #[serde(rename = "chiselTool")]
    ChiselTool { source: u32 },
    #[serde(rename = "distilleryJar")]
    DistilleryJar,
    #[serde(rename = "creativeCore")]
    CreativeCore { input:u32 },
    #[serde(rename="enchanterBook")]
    EnchanterBook,
    #[serde(rename="lumbermillTank")]
    LumbermillTank,
    #[serde(rename="nativeMetadataKey")]
    NativeMetadataKey { code:u16 },
    /// Native literal metadata and tag presence; compound values at required
    /// keys trigger the pinned matcher's recursive failure.
    #[serde(rename = "bloodmagic-altar")]
    BloodAltar { tagged: bool, keys: Vec<String> },
    /// Example of a multi-slot integration rule, not an independent item predicate.
    Integration,
    /// Native Forestry shared-stock requirement; the linked machine process owns matching.
    Forestry,
    /// Require absent or empty root NBT. `meta: true` ignores metadata.
    Untagged {
        meta: bool,
    },
    /// BuildCraft StackHelper: either-side wildcard skips metadata and NBT;
    /// otherwise native subtype and typed-tag comparison. See docs/buildcraft.md.
    Buildcraft {
        wildcard: bool,
        subtypes: bool,
    },
    /// Native ItemSoulVessel extraction, including null and non-string root ids.
    /// Canonical display samples do not restrict the admitted input domain.
    Soul {
        filter: SoulFilter,
    },
    /// AE2 precise item matching for an ordinary NBT anchor: literal metadata,
    /// null/empty root equivalence, numeric floats, recursive typed tags. See docs/inscriber.md.
    Ae,
    /// Pinned TC untagged infusion component. Match template item/meta, or an
    /// offered stack whose FIRST native ore group is in ores. Unlike Ore, mere
    /// membership in a later group is insufficient. Extra offered NBT is allowed.
    Infusion {
        template: String,
        ores: Vec<String>,
    },
    /// A native Forestry member of this registry and root, with the given analysis state.
    /// Examples use canonical native serialization; other valid genomes are not excluded.
    Member {
        root: String,
        analyzed: bool,
    },
    Ore {
        name: String,
        exclusive: bool,
    },
    /// Ignore selected parts of an ingredient's identity. For fluids, the only
    /// wildcard is `meta: false, nbt: true`; the fluid registry is always fixed.
    Wildcard {
        meta: bool,
        nbt: bool,
    },
    /// Same registry, any metadata, with the named root tag absent or a string.
    /// Other tags are unrestricted. This explicitly excludes native non-string
    /// coercion paths that cannot be reconstructed from canonical NBT.
    StringTag {
        key: String,
    },
    /// Literal raw metadata independent of the concrete display item. Registry
    /// and, unless nbt is true, NBT still come from that item. Absent root keys
    /// are rejected even when their stored value is false or zero.
    Metadata {
        /// Nonnegative raw metadata, or Railcraft's native -1 sentinel.
        #[schemars(range(min = -1))]
        value: i32,
        nbt: bool,
        absent: Vec<String>,
    },
    /// Finite physical-metadata partition, or its complement. Shared values keep
    /// large native smelting domains compact without changing item identity.
    #[serde(rename = "metadataSet")]
    MetadataSet {
        #[schemars(length(min = 1, max = 65536))]
        values: Vec<i32>,
        exclude: bool,
        nbt: bool,
    },
    /// Pinned logical getter; Item.meta remains the physical stack metadata.
    #[serde(rename = "nativeItemMetadata")]
    NativeItemMetadata {
        callback: NativeItemMetadataCallback,
        value: i32,
        nbt: bool,
    },
    /// Compare only the named root tags with the choice; other tags are allowed.
    /// Presence checks ignore the tag's type and value, including false/zero.
    Tags {
        meta: bool,
        keys: Vec<String>,
        present: Vec<String>,
        absent: Vec<String>,
    },
    /// Match exact registry/metadata and remaining NBT after removing only these
    /// root keys from the offered stack. Normalize an empty offered compound to
    /// null, then compare it to the unmodified choice's NBT. All other tags matter.
    WithoutTags {
        keys: Vec<String>,
    },
    /// Match the base predicate, except any listed earlier predicate. Each
    /// exclusion has its own item anchor; it is not an accepted alternative.
    /// Exactly one level is permitted; members and nested exclusions are rejected.
    Except {
        base: Box<Match>,
        exclude: Vec<MatchCase>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NativeZeroFluidPolicy {
    /// GTRecipe's cost calculation ignores this entry. Machine lookup/validation is separate.
    GregtechCost,
    /// Industrial Rock Breaker checks fluid identity even when its amount is zero.
    GregtechPresence,
    /// Vat containsFluid requires the exact fluid and NBT; a zero-volume object is sufficient.
    VatMatch,
    /// Incubator still requires the exact fluid/NBT object, with a native zero drain cost.
    BinnieIncubator,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatchCase {
    pub id: String,
    pub rule: Match,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Consumption {
    Consume,
    Keep,
    /// One native draw determines whether a unit is consumed.
    Chance { chance: Chance },
    /// IC2 blast: amount is the stock gate; its process consumes a unit at each stage.
    Staged,
    /// Vat: presence of one item is sufficient; remove min(amount, available) at start.
    Upto,
    /// An ordered native requirement; the process allocates actual machine slots.
    Allocated,
    /// Optional SAG ball inventory; consumed on loading active ball state, not per task.
    Reserve,
    /// Splice tools: conditional damageItem(1, fakePlayer) at completion, not fixed durability.
    Wear,
    /// Consume the entire offered stack. `amount` is the minimum/example count.
    Stack,
    /// Internal fluid stock gate; the paired native process defines when and how it drains.
    /// Some machines empty it at start; others repeatedly allow partial drains during work.
    Buffer,
    /// One item on each distinct pedestal. With Runic, amount is the sample
    /// multiplicity; the process defines the actual input-dependent count.
    Pedestals,
    Damage {
        points: u32,
    },
    /// Installed GT container tool: native units, conditional energy use and wear.
    /// Sample fields describe only the captured candidate, not arbitrary player NBT.
    #[serde(rename = "gtTool")]
    GtTool { cost: u32, owner: String, sample: GtToolSample },
    /// Workbench calls the pinned callback on the actual offered stack after consuming one.
    /// The result depends on wear/RNG or cosmetic NBT, never a fixed sample return.
    #[serde(rename = "nativeContainer")]
    NativeContainer { callback: NativeContainerCallback },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NativeContainerCallback {
    /// IC2 2.2.828: copy, attemptDamageItem(1, IC2.random), null on break.
    Ic2Cutter,
    /// Botania 1.12.28: deserialize cosmeticItem from the offered bauble NBT, or null.
    BotaniaCosmetic,
    /// Botania 1.12.28: copy the offered talisman, debit up to 64 blocks and reset active metadata.
    BotaniaBlackHole,
    /// Thaumic Bases 1.8.13: damage + 1, fresh tagless stack; absent when exhausted.
    ThaumicMortar,
    /// Railcraft 9.16.33: native random wear and enchantment-dependent breakage.
    RailFirestone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NativeItemMetadataCallback {
    BotaniaManaMirror,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GtToolSample {
    pub electric: bool,
    pub damage: String,
    pub limit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub slot: u32,
    pub kind: Kind,
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Chance {
    pub numerator: String,
    pub denominator: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputRole {
    Result,
    Return,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub slot: u32,
    pub kind: Kind,
    pub id: String,
    pub amount: Option<String>,
    pub quantity: Option<Quantity>,
    pub chance: Chance,
    pub role: OutputRole,
    /// With a change, id/amount are the first input choice's example, not a fixed result.
    pub change: Option<Change>,
}

impl Output {
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.id.as_str()).chain(
            self.change
                .iter()
                .flat_map(|change| change.samples.iter().map(|sample| sample.id.as_str())),
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Count,
    Tick,
    Eu,
    EuPerTick,
    Mb,
    Kelvin,
    Pascal,
    Rpm,
    Mana,
    Lp,
    Vis,
    Percent,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum PropertyValue {
    Integer {
        value: String,
    },
    Decimal {
        value: String,
    },
    Quantity {
        amount: String,
        unit: Unit,
    },
    Text {
        text: String,
    },
    Flag {
        value: bool,
    },
    Reference {
        target: Reference,
    },
    List {
        values: Vec<PropertyValue>,
    },
    Map {
        values: BTreeMap<String, PropertyValue>,
    },
    Symbol {
        namespace: String,
        value: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
}

/// Native explanatory text, bound to a stable field key and its reader source.
/// Display text is not part of a recipe's semantic identity.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub key: String,
    pub source: String,
    pub text: String,
}

/// Native crafting arrangement, independent of presentation.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    /// Row-major item input slot references, with explicit empty cells.
    pub cells: Vec<Option<u32>>,
    pub mirror: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub id: String,
    pub source: Origin,
    pub category: String,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    /// Raw native tick value, when defined. Negative values are upstream anomalies,
    /// not elapsed time; preserve them and do not infer an effective execution duration.
    pub duration: Option<String>,
    /// Signed EU/t; negative values denote generation.
    pub energy: Option<String>,
    /// Namespaced properties with an explicit value type and unit.
    pub properties: BTreeMap<String, Property>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
    pub grid: Option<Grid>,
    pub magic: Option<MagicRecipe>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process: Option<Process>,
    /// GT native arrays and unresolved execution policy, separate from display projections.
    #[serde(default, rename = "gtNative", skip_serializing_if = "Option::is_none")]
    pub gt_native: Option<gt_probability::GtNative>,
    /// Native category-local position; branches can tie. Sort tied records by id.
    pub order: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub source: Origin,
    pub icon: Option<Reference>,
    pub machines: Vec<Reference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    /// Independently transported native reference domain, not consuming recipes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub order: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub source: Origin,
    pub name: Option<String>,
    pub members: Vec<String>,
    pub representative: String,
    pub collapsed: bool,
    pub order: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub ticks: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Resource,
    Capture,
    Block,
    Fluid,
    Entity,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetOrigin {
    pub kind: AssetKind,
    pub location: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub id: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    /// Empty means a still image; otherwise every frame specifies its crop and tick duration.
    pub frames: Vec<Frame>,
    pub interpolate: bool,
    pub source: AssetOrigin,
    /// Still preview plus a reviewed inventory program; never a recipe GUI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub id: String,
    pub locale: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    pub visuals: Vec<visual::Visual>,
    pub references: Vec<references::GeneticReferenceRow>,
    pub programs: Vec<programs::ProgramChunk>,
    pub items: Vec<Item>,
    pub fluids: Vec<Fluid>,
    pub recipes: Vec<Recipe>,
    pub categories: Vec<Category>,
    pub groups: Vec<Group>,
    pub assets: Vec<Asset>,
    pub strings: Vec<Text>,
    pub materials: Vec<Material>,
    pub circuits: Vec<Circuit>,
    pub species: Vec<Species>,
    pub mutations: Vec<Mutation>,
    #[serde(rename = "oreGroups")]
    pub ore_groups: Vec<OreGroup>,
    #[serde(rename = "oreMembers")]
    pub ore_members: Vec<OreMember>,
    pub structures: Vec<Structure>,
    pub blocks: Vec<Block>,
    pub models: Vec<Model>,
    pub builds: Vec<Build>,
    pub shapes: Vec<Shape>,
    pub aspects: Vec<Aspect>,
    pub research: Vec<Research>,
}

impl Domain {
    pub fn load(source: &Source) -> Result<Self> {
        let result = Self::load_for_compile(source)?;
        images::verify(source, &result.assets)?;
        Ok(result)
    }

    /// Compilation verifies pixels as they enter the atlas, before publication.
    pub(crate) fn load_for_compile(source: &Source) -> Result<Self> {
        source.environment()?;
        ensure!(source.manifest.scope.collections.iter().map(String::as_str).eq(CORE_COLLECTIONS.iter().copied()),
            "a catalog requires all domain collections; unsupported extensions or partial collections cannot be published");
        let result = Self {
            visuals: read(source, "visuals")?,
            references: read(source,"references")?,
            programs: read(source, "programs")?,
            items: read(source, "items")?,
            fluids: read(source, "fluids")?,
            recipes: read(source, "recipes")?,
            categories: read(source, "categories")?,
            groups: read(source, "groups")?,
            assets: read(source, "assets")?,
            strings: read(source, "strings")?,
            materials: read(source, "materials")?,
            circuits: read(source, "circuits")?,
            species: read(source, "species")?,
            mutations: read(source, "mutations")?,
            ore_groups: read(source, "ore-groups")?,
            ore_members: read(source, "ore-members")?,
            structures: read(source, "structures")?,
            blocks: read(source, "blocks")?,
            models: read(source, "models")?,
            builds: read(source, "builds")?,
            shapes: read(source, "shapes")?,
            aspects: read(source, "aspects")?,
            research: read(source, "research")?,
        };
        result.validate_records(source)?;
        Ok(result)
    }
}

fn read<T: DeserializeOwned>(source: &Source, kind: &str) -> Result<Vec<T>> {
    let mut rows = Vec::new();
    source.visit(kind, |value| {
        let id = value["id"].as_str().unwrap_or_default().to_owned();
        rows.push(serde_json::from_value(value).with_context(|| format!("decode {kind}/{id}"))?);
        Ok(())
    })?;
    Ok(rows)
}

pub fn content_id(prefix: &str, record: &impl Serialize) -> Result<String> {
    let mut value = crate::canonical::value(record)?;
    value
        .as_object_mut()
        .context("record must be an object")?
        .remove("id");
    value_id(prefix, &value)
}

pub fn origin_id(prefix: &str, source: &Origin) -> Result<String> {
    value_id(prefix, &serde_json::to_value(source)?)
}

pub fn recipe_id(recipe: &Recipe) -> Result<String> {
    let values: BTreeMap<_, _> = recipe
        .properties
        .iter()
        .map(|(key, property)| (key, &property.value))
        .collect();
    let mut magic = serde_json::to_value(&recipe.magic)?;
    if let Some(studies) = magic.get_mut("research").and_then(Value::as_array_mut) {
        for study in studies {
            study
                .as_object_mut()
                .expect("research link object")
                .remove("completed");
        }
    }
    let mut outputs = serde_json::to_value(&recipe.outputs)?;
    for output in outputs.as_array_mut().expect("output array") {
        if output.get("change").is_some_and(Value::is_object) {
            output.as_object_mut().expect("output object").remove("id");
            output
                .as_object_mut()
                .expect("output object")
                .remove("amount");
            output["change"]
                .as_object_mut()
                .expect("change object")
                .remove("samples");
            output["change"]
                .as_object_mut()
                .expect("change object")
                .remove("bindings");
        }
    }
    let mut key = json!({
        "source": recipe.source, "category": recipe.category,
        "inputs": recipe.inputs, "outputs": outputs,
        "duration": recipe.duration, "energy": recipe.energy, "properties": values,
        "grid": recipe.grid, "magic": magic,
    });
    if let Some(process) = &recipe.process {
        key["process"] = crate::canonical::value(process)?;
    }
    if let Some(native) = &recipe.gt_native {
        key["gtNative"] = crate::canonical::value(native)?;
    }
    value_id("recipe", &key)
}

fn value_id(prefix: &str, value: &Value) -> Result<String> {
    Ok(format!(
        "{prefix}_{:x}",
        Sha256::digest(crate::canonical::bytes(value)?)
    ))
}

pub fn validate_change(
    recipe: &Recipe,
    output: &Output,
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    change::validate(recipe, output, items)
}
