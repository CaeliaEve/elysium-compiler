//! AspectRecipeIndex composition knowledge, independent of its configured visibility.
use super::{references::{GeneticReferenceRow, ReferenceData}, Aspect};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Visibility { NativePlayerKnowledge, AllRegisteredAspects }

pub fn validate(id: &str, rows: &[&GeneticReferenceRow]) -> Result<()> {
    let first = rows.first().context("Missing aspect reference definition")?;
    let ReferenceData::AspectCombinationsDefinition { source, aspects, consuming, .. } = &first.data
        else { anyhow::bail!("Aspect reference definition must be first") };
    ensure!(!consuming && source.handler == "com.gtnewhorizons.aspectrecipeindex.nei.AspectCombinationHandler"
        && source.owner == "aspectrecipeindex" && !source.key.is_empty(), "Invalid aspect reference origin or consumption");
    ensure!((1..=4096).contains(aspects) && rows.len() == *aspects as usize + 1,
        "Incomplete aspect reference catalogue");
    let records: Vec<_> = rows.iter().map(|r| &r.data).collect();
    ensure!(id == super::value_id("reference", &serde_json::json!({"kind":"thaumcraftAspectCombinations","records":records}))?,
        "Aspect reference content identity differs");
    let mut seen = BTreeSet::new();
    let mut previous = "";
    for row in &rows[1..] {
        let ReferenceData::AspectCombination { aspect, components, .. } = &row.data
            else { anyhow::bail!("Mixed aspect reference domain") };
        ensure!(aspect.as_str() > previous && seen.insert(aspect.as_str())
            && (components.is_empty() || components.len() == 2), "Invalid aspect composition order or arity");
        previous = aspect;
    }
    for row in &rows[1..] {
        if let ReferenceData::AspectCombination { components, .. } = &row.data {
            ensure!(components.iter().all(|id| seen.contains(id.as_str())), "Missing aspect component");
        }
    }
    Ok(())
}

/// Compare the handler observations with the actual content definitions, including knowledge.
pub fn validate_facts(rows: &[GeneticReferenceRow], aspects: &[Aspect]) -> Result<()> {
    let facts: BTreeMap<_, _> = aspects.iter().map(|a| (a.id.as_str(), a)).collect();
    let policies: BTreeMap<_, _> = rows.iter().filter_map(|row| match &row.data {
        ReferenceData::AspectCombinationsDefinition { visibility, .. } => Some((row.domain.as_str(), *visibility)),
        _ => None,
    }).collect();
    for row in rows {
        match &row.data {
            ReferenceData::AspectCombinationsDefinition { aspects, .. } => {
                ensure!(*aspects as usize == facts.len(), "Aspect reference registry coverage differs");
            }
            ReferenceData::AspectCombination { aspect, components, visible } => {
                let fact = facts.get(aspect.as_str()).context("Aspect reference fact absent")?;
                let expected = match policies.get(row.domain.as_str()).context("Aspect visibility policy absent")? {
                    Visibility::NativePlayerKnowledge => fact.discovered.context("Native discovered-aspect knowledge is unavailable")?,
                    Visibility::AllRegisteredAspects => true,
                };
                ensure!(&fact.components == components && *visible == expected,
                    "Aspect reference contradicts native composition or configured visibility");
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ari_policy_shows_undiscovered_aspects_without_changing_knowledge() {
        let mut raw: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/aspect-combinations-contract.json"
        )).unwrap();
        raw["records"][0]["data"]["source"]["owner"] = "aspectrecipeindex".into();
        raw["records"][0]["data"]["source"]["handler"] =
            "com.gtnewhorizons.aspectrecipeindex.nei.AspectCombinationHandler".into();
        raw["records"][0]["data"]["visibility"] = "allRegisteredAspects".into();
        for row in raw["records"].as_array_mut().unwrap().iter_mut().skip(1) {
            row["data"]["visible"] = true.into();
        }
        let mut rows: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone())
            .expect("ARI configuration must be distinct from discovered knowledge");
        let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
        let id = super::super::value_id("reference", &serde_json::json!({
            "kind": "thaumcraftAspectCombinations", "records": data
        })).unwrap();
        for row in &mut rows { row.domain = id.clone(); row.id = row.content_id().unwrap(); }
        validate(&id, &rows.iter().collect::<Vec<_>>()).unwrap();
        let facts: Vec<Aspect> = raw["aspects"].as_array().unwrap().iter().map(|aspect| {
            let mut value = aspect.clone();
            value["name"] = "text_fixture".into(); value["description"] = "text_fixture".into();
            value["color"] = 4294967295u64.into(); value["icon"] = serde_json::Value::Null;
            serde_json::from_value(value).unwrap()
        }).collect();
        assert!(facts.iter().any(|aspect| aspect.discovered == Some(false)));
        validate_facts(&rows, &facts).unwrap();
        let mut hidden = rows.clone();
        if let ReferenceData::AspectCombination { visible, .. } = &mut hidden[1].data { *visible = false; }
        assert!(validate_facts(&hidden, &facts).is_err());
        let mut unknown = facts.clone();
        unknown[0].discovered = None;
        validate_facts(&rows, &unknown).unwrap();
        rows[0].data = serde_json::from_value({
            let mut definition = raw["records"][0]["data"].clone();
            definition["visibility"] = "nativePlayerKnowledge".into(); definition
        }).unwrap();
        assert!(validate_facts(&rows, &facts).is_err());
        assert!(validate_facts(&rows, &unknown).is_err());
    }

    #[test]
    fn java_aspect_graph_survives_full_domain_validation() {
        let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(fixtures.join("aspect-combinations-contract.json")).unwrap()).unwrap();
        let rows: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        // Isolate this native registry from the unrelated magic fixture's aspect IDs.
        domain.recipes.clear();
        domain.research.clear();
        for item in &mut domain.items { item.aspects = None; }
        let text = domain.strings[0].id.clone();
        domain.aspects = raw["aspects"].as_array().unwrap().iter().map(|raw| {
            let mut value = raw.clone();
            value["name"] = text.clone().into(); value["description"] = text.clone().into();
            value["color"] = 4294967295u64.into(); value["icon"] = serde_json::Value::Null;
            serde_json::from_value(value).unwrap()
        }).collect();
        let ReferenceData::AspectCombinationsDefinition { source: origin, .. } = &rows[0].data else { panic!() };
        let mut category = domain.categories[0].clone();
        category.source = origin.clone(); category.id = super::super::origin_id("category", origin).unwrap();
        category.reference = Some(rows[0].domain.clone()); category.program = None; category.icon = None; category.machines.clear();
        domain.categories.push(category); domain.references = rows.clone();
        domain.validate(&source).unwrap();
        let reject = |mut altered: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = altered.iter().map(|r| &r.data).collect();
            let id = super::super::value_id("reference", &serde_json::json!({"kind":"thaumcraftAspectCombinations","records":data})).unwrap();
            for row in &mut altered { row.domain = id.clone(); row.id = row.content_id().unwrap(); }
            assert!(super::super::references::validate(&altered, &BTreeMap::new()).is_err()
                || validate_facts(&altered, &domain.aspects).is_err());
        };
        let mut truncated = rows.clone(); truncated.pop(); reject(truncated);
        let mut consuming = rows.clone();
        if let ReferenceData::AspectCombinationsDefinition { consuming, .. } = &mut consuming[0].data { *consuming = true; }
        reject(consuming);
        let mut graph = rows.clone();
        if let ReferenceData::AspectCombination { components, .. } = &mut graph[1].data { components.clear(); }
        reject(graph);
        domain.aspects[0].discovered = None;
        assert!(validate_facts(&rows, &domain.aspects).is_err());
    }
}
