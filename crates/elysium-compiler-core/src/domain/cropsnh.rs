//! CropsNH registration facts. Membership and costs are not executable machine recipes.
use super::{references::{GeneticReferenceRow, ReferenceData}, Item, Origin};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CropsNhSection { Crops, AlternateSeeds, Mutations, MutationPools }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CropsNhDefinition {
    pub source: Origin,
    pub native_version: String,
    pub semantics: CropsNhSemantics,
    pub section: CropsNhSection,
    pub entries: u32,
    pub consuming: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CropsNhSemantics { CropsnhRegistriesV1 }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CropsNhCropKey { pub crop: String, pub numeric_id: u32, pub seed: String }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CropsNhStack {
    pub registry: String,
    pub meta: i32,
    pub nbt: Option<Nbt>,
    pub nbt_wildcard: bool,
    pub amount: String,
    pub item: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CropsNhBlock { pub registry: String, pub meta: Option<i32> }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum CropsNhSoil {
    Blocks { name: String, blocks: Vec<CropsNhBlock> },
    AnyOf { name: String, soils: Vec<CropsNhSoil> },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum CropsNhRequirement {
    MinLight { level: i32 },
    MaxLight { level: i32 },
    MachineOnly,
    WorldOnly,
    Migrated,
    SubSoil { name: String, materials: Vec<String>, ores: Vec<String>, blocks: Vec<CropsNhBlock> },
    MachineCatalyst { ores: Vec<CropsNhOreCatalyst>, items: Vec<CropsNhStack> },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CropsNhOreCatalyst { pub ore: String, pub amount: String }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CropsNhDrop { pub stack: CropsNhStack, pub weight: i32 }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CropsNhCrop {
    pub crop: CropsNhCropKey,
    pub tier: i32,
    pub growth_duration: i32,
    pub min_seed_bed_tier: i32,
    pub machine_breeding_tier: i32,
    pub hidden: bool,
    pub drop_chance_bits: u32,
    pub crossing_threshold_bits: u32,
    pub breeding_threshold_bits: u32,
    pub liked_biomes: Vec<String>,
    pub soil: Option<CropsNhSoil>,
    pub requirements: Vec<CropsNhRequirement>,
    pub drops: Vec<CropsNhDrop>,
    pub alternate_seeds: Vec<CropsNhStack>,
    pub duplication_catalysts: Vec<CropsNhStack>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CropsNhAlternateSeed {
    pub registry: String,
    pub meta: Option<i32>,
    pub crop: CropsNhCropKey,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CropsNhMutation {
    pub output: CropsNhCropKey,
    pub parents: Vec<CropsNhCropKey>,
    pub requirements: Vec<CropsNhRequirement>,
    pub machine_eut: i32,
    pub machine_duration: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CropsNhMutationPool { pub pool: String, pub members: Vec<CropsNhCropKey> }

fn name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}

fn amount(value: &str) -> Result<()> {
    let count: i32 = value.parse().context("Invalid CropsNH stack quantity")?;
    ensure!(count > 0 && count.to_string() == value, "Noncanonical CropsNH stack quantity");
    Ok(())
}

fn crop(key: &CropsNhCropKey, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(name(&key.crop) && key.crop.contains(':') && key.numeric_id > 0, "Invalid CropsNH crop key");
    let seed = items.get(key.seed.as_str()).context("CropsNH seed fact missing")?;
    let Some(Nbt::Compound { value }) = &seed.nbt else { anyhow::bail!("CropsNH seed NBT missing") };
    ensure!(seed.registry == "cropsnh:genericSeed" && seed.meta == 0
        && matches!(value.get("crop"), Some(Nbt::String { value }) if value == &key.crop),
        "CropsNH seed does not identify its registered crop");
    Ok(())
}

fn stack(value: &CropsNhStack, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(name(&value.registry) && value.registry.contains(':'), "Invalid CropsNH stack registry");
    amount(&value.amount)?;
    if let Some(nbt) = &value.nbt { nbt.validate()?; }
    let wildcard = matches!(value.meta, -1 | 32767) || value.nbt_wildcard;
    ensure!(!value.nbt_wildcard || value.nbt.is_none(), "CropsNH wildcard NBT became a literal tag");
    if let Some(id) = &value.item {
        ensure!(!wildcard, "CropsNH wildcard template became a concrete item");
        let fact = items.get(id.as_str()).context("CropsNH stack fact missing")?;
        ensure!(fact.registry == value.registry && fact.meta == value.meta
            && serde_json::to_value(&fact.nbt)? == serde_json::to_value(&value.nbt)?,
            "CropsNH stack differs from its item fact");
    } else {
        ensure!(wildcard, "Concrete CropsNH stack has no item fact");
    }
    Ok(())
}

fn blocks(values: &[CropsNhBlock]) -> Result<()> {
    ensure!(values.len() <= 65536, "CropsNH block budget exceeded");
    let mut seen = BTreeSet::new();
    for block in values {
        ensure!(name(&block.registry) && block.registry.contains(':')
            && !matches!(block.meta, Some(-1 | 32767))
            && seen.insert((block.registry.as_str(), block.meta)), "Invalid or duplicate CropsNH block match");
    }
    Ok(())
}

fn soil(value: &CropsNhSoil, depth: usize, remaining: &mut usize) -> Result<()> {
    ensure!(depth < 16 && *remaining > 0, "CropsNH soil tree budget exceeded");
    *remaining -= 1;
    match value {
        CropsNhSoil::Blocks { name: id, blocks: values } => {
            ensure!(name(id), "Invalid CropsNH soil name"); blocks(values)?;
        }
        CropsNhSoil::AnyOf { name: id, soils } => {
            ensure!(name(id) && soils.len() <= 256, "Invalid CropsNH compound soil");
            for value in soils { soil(value, depth + 1, remaining)?; }
        }
    }
    Ok(())
}

fn requirements(values: &[CropsNhRequirement], growth: bool, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(values.len() <= 256, "CropsNH requirement budget exceeded");
    for value in values {
        match value {
            CropsNhRequirement::MinLight { .. } | CropsNhRequirement::MaxLight { .. }
                | CropsNhRequirement::Migrated => ensure!(growth, "Growth-only CropsNH breeding requirement"),
            CropsNhRequirement::WorldOnly => ensure!(!growth, "Breeding-only CropsNH growth requirement"),
            CropsNhRequirement::MachineOnly => {}
            CropsNhRequirement::SubSoil { name: id, materials, ores, blocks: values } => {
                ensure!(name(id) && materials.len() <= 65536 && ores.len() <= 65536
                    && materials.iter().chain(ores).all(|s| name(s)), "Invalid CropsNH subsoil definition");
                blocks(values)?;
            }
            CropsNhRequirement::MachineCatalyst { ores, items: values } => {
                ensure!(!growth && ores.len() <= 65536 && values.len() <= 65536
                    && (!ores.is_empty() || !values.is_empty()), "Invalid CropsNH machine catalyst");
                for ore in ores { ensure!(name(&ore.ore), "Invalid CropsNH catalyst ore"); amount(&ore.amount)?; }
                for value in values { stack(value, items)?; }
            }
        }
    }
    Ok(())
}

pub fn validate(id: &str, rows: &[&GeneticReferenceRow], items: &BTreeMap<&str, &Item>) -> Result<()> {
    let ReferenceData::CropsNhDefinition(def) = &rows[0].data else { anyhow::bail!("CropsNH definition missing") };
    let handler = match def.section {
        CropsNhSection::Crops => "NEICropsNHCropHandler",
        CropsNhSection::AlternateSeeds => "NEICropsNHAlternateSeedHandler",
        CropsNhSection::Mutations => "NEICropsNHCropstickBreedingHandler",
        CropsNhSection::MutationPools => "NEICropsNHMutationPoolHandler",
    };
    ensure!(!def.consuming && def.source.owner == "cropsnh" && !def.source.key.is_empty()
        && def.source.handler == format!("com.gtnewhorizon.cropsnh.compatibility.NEI.{handler}")
        && name(&def.native_version) && def.entries <= 65536 && rows.len() == def.entries as usize + 1,
        "Invalid or incomplete CropsNH reference domain");
    let records: Vec<_> = rows.iter().map(|r| &r.data).collect();
    ensure!(id == super::value_id("reference", &serde_json::json!({"kind":"cropsnh","records":records}))?,
        "CropsNH domain content identity differs");
    let mut seen = BTreeSet::new();
    let mut numbers = BTreeSet::new();
    for row in &rows[1..] {
        match (&row.data, def.section) {
            (ReferenceData::CropsNhCrop(value), CropsNhSection::Crops) => {
                crop(&value.crop, items)?;
                ensure!(seen.insert(value.crop.crop.clone()) && numbers.insert(value.crop.numeric_id)
                    && value.crop.numeric_id == row.order && value.growth_duration > 0,
                    "Invalid CropsNH registration order or growth duration");
                for bits in [value.drop_chance_bits, value.crossing_threshold_bits, value.breeding_threshold_bits] {
                    ensure!(f32::from_bits(bits).is_finite(), "Nonfinite CropsNH native float");
                }
                ensure!(value.liked_biomes.len() <= 256 && value.liked_biomes.iter().all(|s| name(s))
                    && value.drops.len() <= 65536 && value.alternate_seeds.len() <= 65536
                    && value.duplication_catalysts.len() <= 65536, "CropsNH crop fact budget exceeded");
                if let Some(value) = &value.soil { soil(value, 0, &mut 4096)?; }
                requirements(&value.requirements, true, items)?;
                for drop in &value.drops { ensure!(drop.weight >= 0, "Negative CropsNH drop weight"); stack(&drop.stack, items)?; }
                for value in value.alternate_seeds.iter().chain(&value.duplication_catalysts) { stack(value, items)?; }
            }
            (ReferenceData::CropsNhAlternateSeed(value), CropsNhSection::AlternateSeeds) => {
                crop(&value.crop, items)?;
                ensure!(name(&value.registry) && value.registry.contains(':') && !matches!(value.meta, Some(-1 | 32767))
                    && seen.insert(format!("{}:{:?}", value.registry, value.meta)), "Duplicate or invalid effective alternate seed");
            }
            (ReferenceData::CropsNhMutation(value), CropsNhSection::Mutations) => {
                crop(&value.output, items)?;
                ensure!((2..=4).contains(&value.parents.len()), "CropsNH mutation must have two to four parents");
                let mut parents = BTreeSet::new();
                for parent in &value.parents {
                    crop(parent, items)?;
                    ensure!(parents.insert(parent.crop.as_str()), "Repeated CropsNH mutation parent");
                }
                requirements(&value.requirements, false, items)?;
            }
            (ReferenceData::CropsNhMutationPool(value), CropsNhSection::MutationPools) => {
                ensure!(name(&value.pool) && seen.insert(value.pool.clone()) && !value.members.is_empty()
                    && value.members.len() <= 65536, "Invalid CropsNH pool membership");
                let mut members = BTreeSet::new();
                for member in &value.members {
                    crop(member, items)?;
                    ensure!(members.insert(member.crop.as_str()), "Repeated CropsNH pool member");
                }
            }
            _ => anyhow::bail!("Mixed CropsNH reference section"),
        }
    }
    Ok(())
}
