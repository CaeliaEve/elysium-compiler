//! InfernalMobs' loaded client pools. Rarity is a conditional roll, not a global drop chance.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Item, Origin,
};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Tier {
    Elite,
    Ultra,
    Inferno,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    NativeClientConfiguration,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Selection {
    FinalModifierCountRepeatedUniformPool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum QuantitySource {
    LoadedPoolStack,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Enchanting {
    NativeInfernalPerSelection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum EnchantmentKind {
    NativeRandomEnchantedBook,
    RemainingModifierCount,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Definition {
    pub source: Origin,
    pub enabled: bool,
    pub ready: bool,
    pub scope: Scope,
    pub consuming: bool,
    pub tiers: u8,
    pub entries: u32,
    pub dimension_blacklist: Vec<i32>,
    pub selection: Selection,
    pub quantity_source: QuantitySource,
    pub enchanting: Enchanting,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TierDefinition {
    pub tier: Tier,
    pub rarity: u32,
    pub requested_min_modifiers: u32,
    pub requested_max_modifiers: u32,
    pub entries: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PoolItem {
    pub tier: Tier,
    pub entry: u32,
    pub item: String,
    pub amount: String,
    pub enchantability: u32,
    pub enchantment_kind: EnchantmentKind,
}

pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::InfernalDropsDefinition(definition) =
        &rows.first().context("Missing infernal pools")?.data
    else {
        anyhow::bail!("Infernal pool definition must be first")
    };
    ensure!(
        definition.ready
            && !definition.consuming
            && definition.entries <= 262144
            && definition.dimension_blacklist.len() <= 16384,
        "Unready, consuming or oversized infernal pools"
    );
    ensure!(
        definition.source.handler == "com.kuba6000.mobsinfo.nei.MobHandlerInfernal"
            && !definition.source.owner.is_empty()
            && !definition.source.key.is_empty(),
        "Wrong infernal pool origin"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"mobsInfoInfernalPools","records":data})
        )?,
        "Infernal pool identity differs"
    );
    ensure!(
        rows.iter()
            .try_fold(0usize, |size, row| Ok::<_, anyhow::Error>(
                size + serde_json::to_vec(row)?.len()
            ))?
            <= 64 * 1024 * 1024,
        "Infernal pools exceed byte budget"
    );
    if !definition.enabled {
        ensure!(
            rows.len() == 1
                && definition.tiers == 0
                && definition.entries == 0
                && definition.dimension_blacklist.is_empty(),
            "Disabled infernal handler retains pool rules"
        );
        return Ok(());
    }
    ensure!(
        definition.tiers == 3 && rows.len() == 4 + definition.entries as usize,
        "Infernal pool catalogue is incomplete"
    );
    let mut cursor = 1usize;
    let mut entries = 0u64;
    for tier in [Tier::Elite, Tier::Ultra, Tier::Inferno] {
        let ReferenceData::InfernalTier(header) =
            &rows.get(cursor).context("Missing infernal tier")?.data
        else {
            anyhow::bail!("Infernal tier order differs")
        };
        cursor += 1;
        ensure!(
            header.tier == tier
                && (1..=i32::MAX as u32).contains(&header.rarity)
                && (1..=65536).contains(&header.requested_min_modifiers)
                && (1..=65536).contains(&header.requested_max_modifiers)
                && header.entries <= 262144,
            "Invalid native infernal tier or requested modifier range"
        );
        // The native configuration can have reversed endpoints; selection sorts them later.
        for ordinal in 0..header.entries {
            let ReferenceData::InfernalPoolItem(entry) = &rows
                .get(cursor)
                .context("Missing infernal pool entry")?
                .data
            else {
                anyhow::bail!("Mixed infernal pool domain")
            };
            cursor += 1;
            entries += 1;
            let item = items
                .get(entry.item.as_str())
                .context("Infernal pool item fact is absent")?;
            let amount = entry
                .amount
                .parse::<u32>()
                .context("Invalid loaded infernal quantity")?;
            ensure!(
                entry.tier == tier
                    && entry.entry == ordinal
                    && amount > 0
                    && amount <= i32::MAX as u32
                    && amount.to_string() == entry.amount
                    && entry.enchantability <= i32::MAX as u32,
                "Infernal entry order, quantity or enchantability differs"
            );
            if item.registry == "minecraft:enchanted_book" {
                ensure!(
                    entry.enchantment_kind == EnchantmentKind::NativeRandomEnchantedBook,
                    "Vanilla enchanted book lost its native generation policy"
                );
            }
        }
    }
    ensure!(
        cursor == rows.len() && entries == u64::from(definition.entries),
        "Infernal pools lost entries"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_infernal_pools_validate_with_duplicates_empty_tier_and_disabled_state() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixtures.join("infernal-drops-contract.json")).unwrap(),
        )
        .unwrap();
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        for field in ["records", "disabled"] {
            let mut domain = super::super::Domain::load(&source).unwrap();
            let base = serde_json::to_value(&domain.items[0]).unwrap();
            for raw in raw["items"].as_array().unwrap() {
                let mut value = base.clone();
                for key in ["id", "registry", "meta", "nbt", "tags"] {
                    value[key] = raw[key].clone();
                }
                let item: Item = serde_json::from_value(value).unwrap();
                domain.items.retain(|old| old.id != item.id);
                domain.items.push(item);
            }
            let rows: Vec<GeneticReferenceRow> =
                serde_json::from_value(raw[field].clone()).unwrap();
            let ReferenceData::InfernalDropsDefinition(definition) = &rows[0].data else {
                panic!()
            };
            let mut category = domain.categories[0].clone();
            category.source = definition.source.clone();
            category.id = super::super::origin_id("category", &category.source).unwrap();
            category.reference = Some(rows[0].domain.clone());
            category.program = None;
            category.icon = None;
            category.machines.clear();
            domain.categories.push(category);
            domain.references = rows.clone();
            domain.validate(&source).unwrap();
            let items = domain
                .items
                .iter()
                .map(|item| (item.id.as_str(), item))
                .collect();
            let reject = |mut changed: Vec<GeneticReferenceRow>| {
                let data: Vec<_> = changed.iter().map(|row| &row.data).collect();
                let id = super::super::value_id(
                    "reference",
                    &serde_json::json!({"kind":"mobsInfoInfernalPools","records":data}),
                )
                .unwrap();
                for row in &mut changed {
                    row.domain = id.clone();
                    row.id = row.content_id().unwrap();
                }
                assert!(super::super::references::validate(&changed, &items).is_err());
            };
            let mut changed = rows.clone();
            if let ReferenceData::InfernalDropsDefinition(definition) = &mut changed[0].data {
                definition.ready = false;
            }
            reject(changed);
            if field == "records" {
                let mut changed = rows.clone();
                changed.pop();
                reject(changed);
                let mut changed = rows.clone();
                if let ReferenceData::InfernalTier(header) = &mut changed[1].data {
                    header.rarity = 0;
                }
                reject(changed);
                let mut changed = rows.clone();
                if let ReferenceData::InfernalPoolItem(entry) = &mut changed[2].data {
                    entry.amount = "0".into();
                }
                reject(changed);
                let mut changed = rows.clone();
                if let ReferenceData::InfernalPoolItem(entry) = &mut changed[5].data {
                    entry.enchantment_kind = EnchantmentKind::RemainingModifierCount;
                }
                reject(changed);
            }
        }
    }
}
