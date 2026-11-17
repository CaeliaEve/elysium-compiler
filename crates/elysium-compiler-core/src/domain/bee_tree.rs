//! BBAB information: all native mutations and analyzed specimens; no GUI or consuming recipe.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Chance, Item, Origin,
};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AncestorSelection {
    FirstRegisteredMutation,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AncestorSecrets {
    NativeUnfiltered,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Specimens {
    NativeAnalyzedTemplates,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub species: u32,
    pub mutations: u32,
    pub eligible: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Definition {
    pub source: Origin,
    pub root: String,
    pub ready: bool,
    pub consuming: bool,
    pub show_secret: bool,
    pub show_duplicate_trees: bool,
    pub ancestor_selection: AncestorSelection,
    pub ancestor_secrets: AncestorSecrets,
    pub specimens: Specimens,
    pub native_tree_depth: u32,
    pub native_visible_parent_depth: u32,
    pub counts: Counts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Species {
    pub species: String,
    pub name: String,
    pub princess: String,
    pub drone: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mutation {
    pub ordinal: u32,
    pub handler: String,
    pub parents: [String; 2],
    pub result: String,
    pub template: Vec<Option<String>>,
    pub chance: Chance,
    pub conditions: Vec<String>,
    pub secret: bool,
    pub eligible: bool,
}
fn uid(text: &str) -> bool {
    !text.is_empty() && text.len() <= 256
}
pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::BeeTreeDefinition(definition) = &rows[0].data else {
        anyhow::bail!("Missing native bee tree definition")
    };
    ensure!(
        definition.ready
            && !definition.consuming
            && definition.root == "rootBees"
            && definition.source.handler
                == "hellfirepvp.beebetteratbees.client.gui.BBABGuiRecipeTreeHandler"
            && !definition.source.owner.is_empty()
            && !definition.source.key.is_empty()
            && definition.native_tree_depth == 4
            && definition.native_visible_parent_depth == 3,
        "Native bee tree source, readiness or display policy differs"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"bbabBeeTree","records":data})
        )?,
        "Native bee tree content identity differs"
    );
    ensure!(
        definition.counts.species <= 65536
            && definition.counts.mutations <= 262144
            && rows.len()
                == 1 + definition.counts.species as usize + definition.counts.mutations as usize,
        "Native bee tree counts are incomplete"
    );
    let mut species = BTreeMap::new();
    let mut previous = "";
    for row in &rows[1..1 + definition.counts.species as usize] {
        let ReferenceData::BeeTreeSpecies(specimen) = &row.data else {
            anyhow::bail!("Expected native bee tree species")
        };
        ensure!(
            uid(&specimen.species)
                && specimen.species.as_str() > previous
                && !specimen.name.is_empty()
                && specimen.name.len() <= 65536
                && specimen.princess != specimen.drone,
            "Invalid or unordered native bee tree species"
        );
        for id in [&specimen.princess, &specimen.drone] {
            items
                .get(id.as_str())
                .context("Analyzed bee tree member fact is absent")?;
        }
        previous = &specimen.species;
        species.insert(specimen.species.as_str(), specimen);
    }
    let mut used = BTreeSet::new();
    let mut eligible = 0;
    for (index, row) in rows[1 + definition.counts.species as usize..]
        .iter()
        .enumerate()
    {
        let ReferenceData::BeeTreeMutation(mutation) = &row.data else {
            anyhow::bail!("Expected native bee tree mutation")
        };
        ensure!(
            mutation.ordinal as usize == index
                && !mutation.handler.is_empty()
                && mutation.handler.len() <= 512
                && !mutation.template.is_empty()
                && mutation.template.len() <= 256
                && mutation.template[0].as_ref() == Some(&mutation.result)
                && mutation.template.iter().flatten().all(|value| uid(value))
                && mutation.conditions.len() <= 256
                && mutation
                    .conditions
                    .iter()
                    .all(|text| !text.is_empty() && text.len() <= 65536)
                && mutation.eligible == (!mutation.secret || definition.show_secret),
            "Invalid native bee tree order, genome, conditions or secret visibility"
        );
        super::check::chance(&mutation.chance)?;
        for species_id in mutation
            .parents
            .iter()
            .chain(std::iter::once(&mutation.result))
        {
            ensure!(
                species.contains_key(species_id.as_str()),
                "Native bee tree parent/result is absent"
            );
            used.insert(species_id.as_str());
        }
        if mutation.eligible {
            eligible += 1;
        }
    }
    ensure!(
        eligible == definition.counts.eligible && used.len() == species.len(),
        "Bee tree lost visible route counts or has orphan species"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_bee_tree_full_domain_rejects_omission_and_secret_reclassification() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixtures.join("bee-tree-contract.json")).unwrap(),
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
        let ReferenceData::BeeTreeDefinition(definition) = &rows[0].data else {
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
        let facts = domain
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        let reject = |mut changed: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = changed.iter().map(|row| &row.data).collect();
            let id = super::super::value_id(
                "reference",
                &serde_json::json!({"kind":"bbabBeeTree","records":data}),
            )
            .unwrap();
            for row in &mut changed {
                row.domain = id.clone();
                row.id = row.content_id().unwrap();
            }
            assert!(super::super::references::validate(&changed, &facts).is_err());
        };
        let mut missing = rows.clone();
        missing.pop();
        reject(missing);
        let mut hidden = rows.clone();
        if let ReferenceData::BeeTreeMutation(route) = &mut hidden[6].data {
            route.eligible = true;
        }
        reject(hidden);
        let mut genome = rows.clone();
        if let ReferenceData::BeeTreeMutation(route) = &mut genome[5].data {
            route.template[0] = None;
        }
        reject(genome);
        let mut unready = rows.clone();
        if let ReferenceData::BeeTreeDefinition(definition) = &mut unready[0].data {
            definition.ready = false;
        }
        reject(unready);
        let mut empty = rows[..1].to_vec();
        if let ReferenceData::BeeTreeDefinition(definition) = &mut empty[0].data {
            definition.counts = Counts {
                species: 0,
                mutations: 0,
                eligible: 0,
            };
        }
        let data: Vec<_> = empty.iter().map(|row| &row.data).collect();
        empty[0].domain = super::super::value_id(
            "reference",
            &serde_json::json!({"kind":"bbabBeeTree","records":data}),
        )
        .unwrap();
        empty[0].id = empty[0].content_id().unwrap();
        super::super::references::validate(&empty, &BTreeMap::new()).unwrap();
    }
}
