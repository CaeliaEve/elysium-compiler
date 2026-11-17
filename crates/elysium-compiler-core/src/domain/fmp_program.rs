//! Shared FMP native lookup/source facts. Recipes contain bounded branch witnesses.
use super::{fmp::Saw, Item};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "FmpNativeMaterial")]
pub struct Material {
    pub id: u32,
    pub name: String,
    pub block: String,
    pub cutter_strength: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "FmpNativeName")]
pub struct Name {
    pub name: String,
    pub id: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "FmpNativeAlias")]
pub struct Alias {
    pub name: String,
    pub target: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "FmpNativeSettings")]
pub struct Settings {
    pub native_recipe: String,
    pub version: String,
    pub micro_registry: String,
    pub max_saw_strength: i32,
    pub alias_policy: AliasPolicy,
    pub missing_policy: MissingPolicy,
    pub missing_compound_policy: CompoundPolicy,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AliasPolicy {
    SingleHop,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum MissingPolicy {
    IdZero,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CompoundPolicy {
    ReadOnlyIdZero,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "FmpNativeProgram")]
pub struct Program {
    pub materials: Vec<Material>,
    pub names: Vec<Name>,
    pub aliases: Vec<Alias>,
    pub saws: Vec<Saw>,
    pub settings: Vec<Settings>,
}

impl Program {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.settings.len() == 1
                && self.materials.len() <= 65536
                && self.names.len() <= 262144
                && self.aliases.len() <= 262144
                && self.saws.len() <= 4096,
            "Invalid FMP shared source budget"
        );
        let settings = &self.settings[0];
        ensure!(
            settings.native_recipe == "codechicken.microblock.MicroRecipe"
                && settings.version == "1.7.2"
                && !settings.micro_registry.is_empty(),
            "Unadapted FMP native settings"
        );
        let mut seen = BTreeSet::new();
        for (id, material) in self.materials.iter().enumerate() {
            ensure!(
                material.id == id as u32
                    && material.name.len() <= 65535
                    && seen.insert(&material.name)
                    && material.block.starts_with("item_"),
                "Invalid FMP ordered material source"
            );
        }
        ensure!(
            self.names
                .windows(2)
                .all(|w| java_cmp(&w[0].name, &w[1].name).is_lt())
                && self
                    .aliases
                    .windows(2)
                    .all(|w| java_cmp(&w[0].name, &w[1].name).is_lt()),
            "FMP lookup keys duplicate or unordered"
        );
        for name in &self.names {
            ensure!(
                name.id >= 0
                    && (name.id as usize) < self.materials.len()
                    && name.name.len() <= 65535,
                "FMP native name points outside the material table"
            );
        }
        for alias in &self.aliases {
            ensure!(
                alias.name.len() <= 65535 && alias.target.len() <= 65535,
                "FMP alias exceeds native key budget"
            );
        }
        ensure!(
            self.saws
                .windows(2)
                .all(|w| java_cmp(&w[0].registry, &w[1].registry).is_lt()),
            "Duplicate or unordered native saw source"
        );
        for saw in &self.saws {
            ensure!(
                saw.strength == saw.max_strength
                    && !saw.registry.is_empty()
                    && match saw.owner.as_str() {
                        "codechicken.microblock.ItemSaw" =>
                            matches!(saw.nbt, super::fmp::SawNbt::FreshWhenDamageable),
                        "mrtjp.projectred.exploration.ItemGemSaw" =>
                            matches!(saw.nbt, super::fmp::SawNbt::Preserve),
                        _ => false,
                    },
                "Unadapted FMP saw law"
            );
        }
        Ok(())
    }
    pub fn settings(&self) -> &Settings {
        &self.settings[0]
    }
    pub fn material(&self, id: u32) -> Result<&Material> {
        self.materials
            .get(id as usize)
            .context("FMP material ID absent")
    }
    pub fn resolve(&self, name: &str) -> i32 {
        let name = self
            .aliases
            .iter()
            .find(|alias| alias.name == name)
            .map_or(name, |alias| alias.target.as_str());
        self.names
            .iter()
            .find(|entry| entry.name == name)
            .map_or(0, |entry| entry.id)
    }
    /// NBT text is observed by Java on the offered specimen; sorted typed NBT
    /// cannot recover HashMap order for non-string tags. Missing mat is ID zero.
    pub fn micro_id(&self, item: &Item, projection: Option<&str>) -> Result<i32> {
        let tag = match &item.nbt {
            Some(Nbt::Compound { value }) => value.get("mat"),
            None => None,
            _ => anyhow::bail!("FMP stack NBT must be a compound"),
        };
        let Some(tag) = tag else {
            ensure!(
                projection.is_none(),
                "FMP invented a native material text for missing mat"
            );
            return Ok(0);
        };
        let text = match tag {
            Nbt::String { value } => {
                ensure!(
                    projection == Some(value.as_str()),
                    "FMP native material text differs from its string tag"
                );
                value.as_str()
            }
            _ => projection.context("FMP non-string mat requires its observed native NBT text")?,
        };
        ensure!(
            text.len() <= 524288,
            "FMP native material text exceeds its budget"
        );
        Ok(self.resolve(text))
    }
    pub fn find_material(&self, item: &Item, items: &BTreeMap<&str, &Item>) -> Result<i32> {
        for material in &self.materials {
            let block = items
                .get(material.block.as_str())
                .context("FMP native full block fact absent")?;
            if item.registry == block.registry && item.meta == block.meta && item.nbt == block.nbt {
                return Ok(self.resolve(&material.name));
            }
        }
        Ok(-1)
    }
}
fn java_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}
