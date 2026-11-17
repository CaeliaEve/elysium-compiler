//! Pinned GT world-generation configuration, native selection law and independently verified NEI projection.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Fluid, Origin,
};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Selection {
    NativeThousandTicketsInclusive,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AmountGeneration {
    NativeFifthPowerVeinData,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub dimensions: u32,
    pub entries: u32,
    pub displays: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Definition {
    pub source: Origin,
    pub ready: bool,
    pub consuming: bool,
    pub selection: Selection,
    pub amount_generation: AmountGeneration,
    pub counts: Counts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Dimension {
    pub dimension: String,
    pub display_dimension: Option<String>,
    pub entries: u32,
    pub total_weight: u32,
    pub null_tickets: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    pub dimension: String,
    pub ordinal: u32,
    pub registry: String,
    pub fluid: Option<String>,
    pub weight: u32,
    pub min_amount: u32,
    pub max_amount: u32,
    pub decrease_per_operation: u32,
    pub selected_tickets: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Display {
    pub registry: String,
    pub ordinal: u32,
    pub dimension: String,
    pub native_chance: u32,
    pub min_amount: u32,
    pub max_amount: u32,
}
fn identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256
}
type DisplayKey = (String, String, u32, u32, u32);
pub fn validate(id: &str, rows: &[&GeneticReferenceRow]) -> Result<()> {
    let ReferenceData::UndergroundFluidsDefinition(definition) = &rows[0].data else {
        anyhow::bail!("Missing underground-fluid definition")
    };
    ensure!(
        definition.ready
            && !definition.consuming
            && definition.source.handler
                == "gtneioreplugin.plugin.gregtech5.PluginGT5UndergroundFluid"
            && !definition.source.owner.is_empty()
            && definition.source.key == "GTOrePluginUndergroundFluid",
        "Underground-fluid source, readiness or semantics differ"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"gtUndergroundFluids","records":data})
        )?,
        "Underground-fluid content identity differs"
    );
    let counts = &definition.counts;
    ensure!(
        counts.dimensions <= 4096
            && counts.entries <= 262144
            && counts.displays <= 262144
            && rows.len()
                == 1 + counts.dimensions as usize
                    + counts.entries as usize
                    + counts.displays as usize,
        "Underground-fluid record counts are incomplete"
    );
    let mut index = 1;
    let mut entries = 0;
    let mut previous = "";
    let mut expected = BTreeMap::<DisplayKey, u32>::new();
    for _ in 0..counts.dimensions {
        let ReferenceData::UndergroundFluidDimension(dimension) = &rows
            .get(index)
            .context("Missing underground-fluid dimension")?
            .data
        else {
            anyhow::bail!("Missing underground-fluid dimension")
        };
        index += 1;
        ensure!(
            identifier(&dimension.dimension)
                && dimension.dimension.as_str() > previous
                && dimension.entries <= 65536
                && dimension.total_weight <= i32::MAX as u32
                && dimension.null_tickets <= 1000
                && dimension
                    .display_dimension
                    .as_ref()
                    .is_none_or(|text| identifier(text)),
            "Invalid underground-fluid dimension or order"
        );
        previous = &dimension.dimension;
        let mut sum = 0u64;
        let mut seen = BTreeSet::new();
        let mut leaves = Vec::new();
        for ordinal in 0..dimension.entries {
            let row = rows.get(index).context("Missing underground-fluid entry")?;
            index += 1;
            let ReferenceData::UndergroundFluidEntry(entry) = &row.data else {
                anyhow::bail!("Unexpected underground-fluid dimension record")
            };
            ensure!(
                entry.dimension == dimension.dimension
                    && entry.ordinal == ordinal
                    && identifier(&entry.registry)
                    && seen.insert(&entry.registry)
                    && entry.weight <= i32::MAX as u32 / 10000
                    && entry.min_amount <= entry.max_amount
                    && entry.max_amount > 0
                    && entry.max_amount <= i32::MAX as u32
                    && entry.decrease_per_operation <= i32::MAX as u32
                    && entry.selected_tickets <= 1000,
                "Invalid underground-fluid entry, weight, amounts or native selection order"
            );
            if let Some(fluid) = &entry.fluid {
                ensure!(
                    *fluid == crate::identity::fluid_id(&entry.registry, None)?,
                    "Underground-fluid registry/identity differs"
                );
                if let Some(label) = &dimension.display_dimension {
                    ensure!(
                        dimension.total_weight > 0,
                        "Underground-fluid display has zero weight sum"
                    );
                    *expected
                        .entry((
                            label.clone(),
                            entry.registry.clone(),
                            entry.weight * 10000 / dimension.total_weight,
                            entry.min_amount,
                            entry.max_amount,
                        ))
                        .or_default() += 1;
                }
            }
            sum += u64::from(entry.weight);
            leaves.push(entry);
            entries += 1;
        }
        ensure!(
            sum == u64::from(dimension.total_weight) && (leaves.is_empty() || sum > 0),
            "Underground-fluid native accumulated weight differs"
        );
        let mut selected = vec![0u32; leaves.len()];
        let mut null = 0;
        for ticket in 0..1000 {
            let mut remaining = ticket;
            let mut found = false;
            for (i, entry) in leaves.iter().enumerate() {
                let threshold = entry.weight * 1000 / dimension.total_weight;
                if remaining <= threshold {
                    selected[i] += 1;
                    found = true;
                    break;
                }
                remaining -= threshold;
            }
            if !found {
                null += 1;
            }
        }
        ensure!(null==dimension.null_tickets&&leaves.iter().zip(selected).all(|(entry,tickets)|entry.selected_tickets==tickets),
            "Underground-fluid probability differs from native integer and inclusive-boundary selection");
    }
    ensure!(
        entries == counts.entries,
        "Underground-fluid entry count differs"
    );
    let mut actual = BTreeMap::<DisplayKey, u32>::new();
    let mut previous_registry = "";
    let mut ordinal = 0;
    for row in &rows[index..] {
        let ReferenceData::UndergroundFluidDisplay(display) = &row.data else {
            anyhow::bail!("Unexpected underground-fluid display record")
        };
        if display.registry != previous_registry {
            ensure!(
                display.registry.as_str() > previous_registry,
                "Underground-fluid display groups out of order"
            );
            ordinal = 0;
            previous_registry = &display.registry;
        }
        ensure!(
            identifier(&display.registry)
                && identifier(&display.dimension)
                && display.ordinal == ordinal,
            "Invalid underground-fluid display order"
        );
        ordinal += 1;
        *actual
            .entry((
                display.dimension.clone(),
                display.registry.clone(),
                display.native_chance,
                display.min_amount,
                display.max_amount,
            ))
            .or_default() += 1;
    }
    ensure!(
        actual == expected,
        "Underground-fluid NEI projection is stale or incomplete"
    );
    Ok(())
}
pub fn validate_facts(rows: &[GeneticReferenceRow], fluids: &BTreeMap<&str, &Fluid>) -> Result<()> {
    let registries: BTreeSet<_> = fluids
        .values()
        .map(|fluid| fluid.registry.as_str())
        .collect();
    for row in rows {
        if let ReferenceData::UndergroundFluidEntry(entry) = &row.data {
            if let Some(id) = &entry.fluid {
                let fluid = fluids
                    .get(id.as_str())
                    .context("Native underground fluid fact is absent")?;
                ensure!(
                    fluid.registry == entry.registry && fluid.nbt.is_none(),
                    "Underground-fluid fact does not match configuration"
                );
            } else {
                ensure!(
                    !registries.contains(entry.registry.as_str()),
                    "Registered underground fluid presented as unresolved"
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/underground-fluids-contract.json"
        ))
        .unwrap()
    }
    fn reseal(rows: &mut [GeneticReferenceRow]) {
        let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
        let id = super::super::value_id(
            "reference",
            &serde_json::json!({"kind":"gtUndergroundFluids","records":data}),
        )
        .unwrap();
        for row in rows {
            row.domain = id.clone();
            row.id = row.content_id().unwrap();
        }
    }
    #[test]
    fn java_underground_native_law_rejects_reclassified_probabilities_and_missing_projection() {
        let mut rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(fixture()["records"].clone()).unwrap();
        super::super::references::validate(&rows, &BTreeMap::new()).unwrap();
        let original = rows.clone();
        if let ReferenceData::UndergroundFluidEntry(entry) = &mut rows[2].data {
            entry.selected_tickets -= 1;
        }
        reseal(&mut rows);
        assert!(super::super::references::validate(&rows, &BTreeMap::new()).is_err());
        let mut rows = original.clone();
        rows.pop();
        if let ReferenceData::UndergroundFluidsDefinition(definition) = &mut rows[0].data {
            definition.counts.displays -= 1;
        }
        reseal(&mut rows);
        assert!(super::super::references::validate(&rows, &BTreeMap::new()).is_err());
        let mut rows = original;
        if let ReferenceData::UndergroundFluidsDefinition(definition) = &mut rows[0].data {
            definition.ready = false;
        }
        reseal(&mut rows);
        assert!(super::super::references::validate(&rows, &BTreeMap::new()).is_err());
    }
    #[test]
    fn java_underground_records_pass_complete_domain_and_reject_missing_fluid() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        let fixture = fixture();
        let rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(fixture["records"].clone()).unwrap();
        let ReferenceData::UndergroundFluidsDefinition(definition) = &rows[0].data else {
            panic!()
        };
        let base = serde_json::to_value(&domain.fluids[0]).unwrap();
        for raw in fixture["fluids"].as_array().unwrap() {
            let mut fact = base.clone();
            for key in [
                "id",
                "registry",
                "nbt",
                "temperature",
                "density",
                "viscosity",
                "gaseous",
                "luminosity",
            ] {
                fact[key] = raw[key].clone();
            }
            fact["icon"] = serde_json::Value::Null;
            domain.fluids.push(serde_json::from_value(fact).unwrap());
        }
        let mut category = domain.categories[0].clone();
        category.source = definition.source.clone();
        category.id = super::super::origin_id("category", &category.source).unwrap();
        category.reference = Some(rows[0].domain.clone());
        category.program = None;
        category.icon = None;
        category.machines.clear();
        domain.categories.push(category);
        domain.references = rows;
        domain.validate(&source).unwrap();
        domain.fluids.pop();
        assert!(domain.validate(&source).is_err());
    }
}
