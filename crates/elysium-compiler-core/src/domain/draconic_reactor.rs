//! The reactor NEI page is a configured process reference, not a fixed consuming recipe.
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
pub enum FuelRole {
    Block,
    Ingot,
    Nugget,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ProductRole {
    Tiny,
    Small,
    Large,
    Full,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    NativeConfiguredReference,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FuelMatching {
    OrderedOreDictionary,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AutomationExtraction {
    FullShardStrictlyGreater,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ManualExtraction {
    FuelRefundBeforeFragments,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ManualAccess {
    OfflineOnly,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    StateDependentEnergyAndConversion,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Fuel {
    pub role: FuelRole,
    pub item: String,
    pub ore: String,
    pub units: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Product {
    pub role: ProductRole,
    pub item: String,
    pub units: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub maximum_fuel_storage: u32,
    pub configured_fuel_storage: u32,
    pub configured_nugget_units: u32,
    pub configured_tiny_chaos_units: u32,
    pub output_multiplier: String,
    pub fuel_usage_multiplier: String,
    pub linear_fuel_usage: bool,
    pub large_explosion: bool,
    pub automation: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Definition {
    pub source: Origin,
    pub ready: bool,
    pub consuming: bool,
    pub scope: Scope,
    pub core: String,
    pub fuels: Vec<Fuel>,
    pub products: Vec<Product>,
    pub config: Config,
    pub fuel_matching: FuelMatching,
    pub automation_extraction: AutomationExtraction,
    pub manual_extraction: ManualExtraction,
    pub manual_stack_upgrade_threshold: u32,
    pub manual_access: ManualAccess,
    pub operation: Operation,
}

fn units(text: &str) -> Result<u64> {
    let value = text
        .parse::<u32>()
        .context("Invalid native reactor units")?;
    ensure!(
        value <= i32::MAX as u32 && value.to_string() == text,
        "Invalid native reactor units"
    );
    Ok(u64::from(value))
}
pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    ensure!(
        rows.len() == 1,
        "Native reactor reference must have one complete definition"
    );
    let ReferenceData::DraconicReactorDefinition(definition) = &rows[0].data else {
        anyhow::bail!("Missing native reactor definition")
    };
    ensure!(
        definition.ready
            && !definition.consuming
            && definition.fuels.len() == 3
            && definition.products.len() == 4
            && definition.manual_stack_upgrade_threshold == 64,
        "Native reactor reference became incomplete or consuming"
    );
    ensure!(
        definition.source.handler
            == "com.brandon3055.draconicevolution.integration.nei.ReactorNEIHandler"
            && !definition.source.owner.is_empty()
            && !definition.source.key.is_empty(),
        "Wrong native reactor source"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"draconicReactorInformation","records":data})
        )?,
        "Native reactor reference identity differs"
    );
    ensure!(
        items.contains_key(definition.core.as_str()),
        "Native reactor core representation is missing"
    );
    let roles = [FuelRole::Block, FuelRole::Ingot, FuelRole::Nugget];
    let ores = [
        "blockDraconiumAwakened",
        "ingotDraconiumAwakened",
        "nuggetDraconiumAwakened",
    ];
    let base = units(&definition.fuels[2].units)?;
    for (index, fuel) in definition.fuels.iter().enumerate() {
        ensure!(
            fuel.role == roles[index]
                && fuel.ore == ores[index]
                && items.contains_key(fuel.item.as_str())
                && units(&fuel.units)? == base * [81, 9, 1][index],
            "Native fuel matching order or units differ"
        );
    }
    let roles = [
        ProductRole::Tiny,
        ProductRole::Small,
        ProductRole::Large,
        ProductRole::Full,
    ];
    let base = units(&definition.products[0].units)?;
    let fragments = items
        .get(definition.products[0].item.as_str())
        .context("Native chaos fragment is missing")?;
    for (index, product) in definition.products.iter().enumerate() {
        let item = items
            .get(product.item.as_str())
            .context("Native reactor product is missing")?;
        ensure!(
            product.role == roles[index] && units(&product.units)? == base * [1, 9, 81, 729][index],
            "Native reactor output roles or units differ"
        );
        if index < 3 {
            ensure!(
                item.registry == fragments.registry
                    && item.meta == index as i32
                    && item.nbt.is_none(),
                "Native chaos fragment variants differ"
            );
        }
    }
    let config = &definition.config;
    ensure!(
        config.maximum_fuel_storage <= 10_000_000
            && config.configured_fuel_storage <= 10_000_000
            && config.configured_nugget_units <= 1_000_000
            && config.configured_tiny_chaos_units <= 1_000_000,
        "Native reactor configuration exceeds installed bounds"
    );
    for text in [&config.output_multiplier, &config.fuel_usage_multiplier] {
        let value = text
            .parse::<f64>()
            .context("Invalid native reactor multiplier")?;
        ensure!(
            text.len() <= 64 && value.is_finite() && (0.0..=1_000_000.0).contains(&value),
            "Invalid native reactor multiplier"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_reactor_units_validate_as_reference_and_reject_fake_conversion() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixtures.join("draconic-reactor-contract.json")).unwrap(),
        )
        .unwrap();
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
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
            serde_json::from_value(raw["records"].clone()).unwrap();
        let ReferenceData::DraconicReactorDefinition(definition) = &rows[0].data else {
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
        let reject = |change: fn(&mut Definition)| {
            let mut changed = rows.clone();
            if let ReferenceData::DraconicReactorDefinition(definition) = &mut changed[0].data {
                change(definition);
            }
            let data: Vec<_> = changed.iter().map(|row| &row.data).collect();
            let id = super::super::value_id(
                "reference",
                &serde_json::json!({"kind":"draconicReactorInformation","records":data}),
            )
            .unwrap();
            changed[0].domain = id;
            changed[0].id = changed[0].content_id().unwrap();
            assert!(super::super::references::validate(&changed, &items).is_err());
        };
        reject(|definition| definition.consuming = true);
        reject(|definition| definition.products[3].units = "144".into());
        reject(|definition| definition.config.output_multiplier = "NaN".into());
        reject(|definition| {
            definition.fuels.pop();
        });
    }
}
