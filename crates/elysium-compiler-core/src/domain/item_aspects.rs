//! Native scanned item -> aspect observations, including explicit cache/parse gaps.
use super::{references::{GeneticReferenceRow, ReferenceData}, Item};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ScanStatus { Available, NoAspects, CacheMiss, InvalidKey }

pub fn validate(id: &str, rows: &[&GeneticReferenceRow], items: &BTreeMap<&str, &Item>) -> Result<()> {
    let first = rows.first().context("Missing item aspect definition")?;
    let ReferenceData::ItemAspectsDefinition { source, scans, ready, consuming, .. } = &first.data
        else { anyhow::bail!("Item aspect definition must be first") };
    ensure!(*ready && !consuming && *scans <= 262144 && rows.len() == *scans as usize + 1,
        "Incomplete or unready scanned-item catalogue");
    ensure!(source.handler == "com.gtnewhorizons.aspectrecipeindex.nei.ItemsContainingAspectHandler"
        && source.owner == "aspectrecipeindex" && !source.key.is_empty(), "Invalid item aspect native origin");
    let records: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(id == super::value_id("reference", &serde_json::json!({"kind":"thaumcraftScannedItemAspects","records":records}))?,
        "Item aspect domain identity differs");
    for row in &rows[1..] {
        let ReferenceData::ItemAspectObservation { scan, status, item, aspects } = &row.data
            else { anyhow::bail!("Mixed item aspect domain") };
        ensure!(!scan.is_empty() && scan.len() <= 1024, "Invalid scanned-object key");
        let parsed = scan.chars().skip(1).collect::<String>().parse::<i32>().is_ok();
        ensure!(parsed != (*status == ScanStatus::InvalidKey), "Scan parse status contradicts key");
        match status {
            ScanStatus::CacheMiss | ScanStatus::InvalidKey => {
                ensure!(item.is_none() && aspects.is_none(), "Unresolved scan fabricated item or aspects");
            }
            ScanStatus::Available | ScanStatus::NoAspects => {
                let item = items.get(item.as_deref().context("Scanned item identity missing")?).context("Scanned item fact absent")?;
                let values = aspects.as_ref().context("Scanned aspect quantities missing")?;
                ensure!(values.len() <= 4096 && values.is_empty() == (*status == ScanStatus::NoAspects),
                    "Scan status contradicts aspect amounts");
                ensure!(serde_json::to_value(&item.aspects)? == serde_json::to_value(aspects)?,
                    "Scanned quantities contradict native item bonus aspects");
                ensure!(values.windows(2).all(|pair| pair[0].aspect < pair[1].aspect), "Duplicate or unordered scanned aspects");
                for amount in values {
                    crate::identity::integer(&amount.amount, 0, i32::MAX as i64)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_scanned_items_preserve_native_quantities_and_gap_provenance() {
        let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(fixtures.join("item-aspects-contract.json")).unwrap()).unwrap();
        let rows: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        domain.recipes.clear(); domain.research.clear();
        for item in &mut domain.items { item.aspects = None; }
        let text = domain.strings[0].id.clone();
        domain.aspects = raw["aspects"].as_array().unwrap().iter().map(|raw| {
            let mut value = raw.clone();
            value["name"] = text.clone().into(); value["description"] = text.clone().into();
            value["color"] = 4294967295u64.into(); value["icon"] = serde_json::Value::Null;
            serde_json::from_value(value).unwrap()
        }).collect();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        for raw in raw["items"].as_array().unwrap() {
            let mut value = base.clone();
            for key in ["id", "registry", "meta", "nbt", "tags", "aspects"] { value[key] = raw[key].clone(); }
            domain.items.push(serde_json::from_value(value).unwrap());
        }
        let ReferenceData::ItemAspectsDefinition { source: origin, .. } = &rows[0].data else { panic!() };
        let mut category = domain.categories[0].clone();
        category.source = origin.clone(); category.id = super::super::origin_id("category", origin).unwrap();
        category.reference = Some(rows[0].domain.clone()); category.program = None; category.icon = None; category.machines.clear(); domain.categories.push(category);
        domain.references = rows.clone(); domain.validate(&source).unwrap();
        let items = domain.items.iter().map(|item| (item.id.as_str(), item)).collect();
        let check = |mut altered: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = altered.iter().map(|row| &row.data).collect();
            let id = super::super::value_id("reference", &serde_json::json!({"kind":"thaumcraftScannedItemAspects","records":data})).unwrap();
            for row in &mut altered { row.domain = id.clone(); row.id = row.content_id().unwrap(); }
            super::super::references::validate(&altered, &items)
        };
        check(serde_json::from_value(raw["configuredRecords"].clone()).unwrap()).unwrap();
        let mut incomplete = rows.clone(); incomplete.pop(); assert!(check(incomplete).is_err());
        let mut changed = rows.clone();
        if let ReferenceData::ItemAspectObservation { aspects: Some(aspects), .. } = &mut changed[1].data { aspects[0].amount = "999".into(); }
        assert!(check(changed).is_err());
        let mut unready = rows.clone();
        if let ReferenceData::ItemAspectsDefinition { ready, .. } = &mut unready[0].data { *ready = false; }
        assert!(check(unready).is_err());
        let mut empty = rows[..1].to_vec();
        if let ReferenceData::ItemAspectsDefinition { scans, .. } = &mut empty[0].data { *scans = 0; }
        check(empty).unwrap();
    }
}
