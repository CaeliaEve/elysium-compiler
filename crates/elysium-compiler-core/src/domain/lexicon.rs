//! Botania's NEI lexicon index. Entries associate native text/knowledge with displayed items.
use super::{references::{GeneticReferenceRow, ReferenceData}, Item};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LexiconText { pub key: String, pub value: String }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct LexiconKnowledge { pub name: LexiconText, pub color: String, pub auto_unlock: bool }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct LexiconItem { pub item: String, pub native_key: String, pub quick_info: LexiconText, pub has_quick_info: bool }

fn text(value: &LexiconText) -> Result<()> {
    ensure!(!value.key.is_empty() && value.key.len() <= 1048576 && !value.value.is_empty()
        && value.value.len() <= 1048576, "Missing or oversized native lexicon text");
    Ok(())
}
pub fn validate(id: &str, rows: &[&GeneticReferenceRow], items: &BTreeMap<&str, &Item>) -> Result<()> {
    let first=rows.first().context("Missing lexicon definition")?;
    let ReferenceData::LexiconDefinition { source, entries, pages, consuming }=&first.data
        else { anyhow::bail!("Lexicon definition must be first") };
    ensure!(!consuming && (1..=16384).contains(entries) && *pages <= 262144 && rows.len()==*entries as usize+1,
        "Incomplete or consuming lexicon domain");
    ensure!(source.handler=="vazkii.botania.client.integration.nei.recipe.RecipeHandlerLexicaBotania"
        && !source.owner.is_empty() && !source.key.is_empty(),"Invalid native lexicon origin");
    let records:Vec<_>=rows.iter().map(|row|&row.data).collect();
    ensure!(id==super::value_id("reference",&serde_json::json!({"kind":"botaniaLexiconIndex","records":records}))?,
        "Lexicon domain identity differs");
    let mut seen=BTreeSet::new();let mut actual_pages=0u64;
    for row in &rows[1..] {
        let ReferenceData::LexiconEntry { entry, name, knowledge, tagline, items: displayed }=&row.data
            else { anyhow::bail!("Mixed lexicon domain") };
        ensure!(seen.insert(entry.as_str()) && name.key==*entry,"Duplicate or contradictory lexicon entry key");
        text(name)?;text(&knowledge.name)?;
        ensure!(knowledge.name.key.starts_with("botania.knowledge.") && knowledge.color.chars().count()==2
            && knowledge.color.starts_with('§') && "0123456789abcdefklmnor".contains(knowledge.color.chars().nth(1).unwrap()),
            "Invalid native lexicon knowledge presentation");
        if let Some(tagline)=tagline { text(tagline)?; }
        actual_pages+=displayed.len() as u64;
        for item in displayed {
            ensure!(items.contains_key(item.item.as_str()) && !item.native_key.is_empty()
                && item.native_key.len()<=1048576,"Lexicon displayed item fact missing");
            text(&item.quick_info)?;
            ensure!(item.quick_info.key==if item.has_quick_info {format!("botania.nei.quickInfo:{}",item.native_key)}
                else {"botania.nei.lexicaNoInfo".into()},"Lexicon quick info source differs");
        }
    }
    ensure!(actual_pages==u64::from(*pages),"Lexicon lost native displayed pages");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_lexicon_keeps_native_text_and_knowledge_without_book_consumption() {
        let fixtures=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw:serde_json::Value=serde_json::from_slice(&std::fs::read(fixtures.join("botania-lexicon-contract.json")).unwrap()).unwrap();
        let rows:Vec<GeneticReferenceRow>=serde_json::from_value(raw["records"].clone()).unwrap();
        let source=crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain=super::super::Domain::load(&source).unwrap();
        let base=serde_json::to_value(&domain.items[0]).unwrap();
        for raw in raw["items"].as_array().unwrap() {
            let mut value=base.clone();
            for key in ["id","registry","meta","nbt","tags"] {value[key]=raw[key].clone();}
            let item:Item=serde_json::from_value(value).unwrap();
            domain.items.retain(|old|old.id!=item.id);domain.items.push(item);
        }
        let ReferenceData::LexiconDefinition{source:origin,..}=&rows[0].data else {panic!()};
        let mut category=domain.categories[0].clone();
        category.source=origin.clone();category.id=super::super::origin_id("category",origin).unwrap();
        category.reference=Some(rows[0].domain.clone());category.program=None;category.icon=None;category.machines.clear();
        domain.categories.push(category);domain.references=rows.clone();
        domain.validate(&source).unwrap();
        let items=domain.items.iter().map(|item|(item.id.as_str(),item)).collect();
        let reject=|mut altered:Vec<GeneticReferenceRow>| {
            let data:Vec<_>=altered.iter().map(|row|&row.data).collect();
            let id=super::super::value_id("reference",&serde_json::json!({"kind":"botaniaLexiconIndex","records":data})).unwrap();
            for row in &mut altered {row.domain=id.clone();row.id=row.content_id().unwrap();}
            assert!(super::super::references::validate(&altered,&items).is_err());
        };
        let mut truncated=rows.clone();truncated.pop();reject(truncated);
        let mut consuming=rows.clone();
        if let ReferenceData::LexiconDefinition{consuming,..}=&mut consuming[0].data {*consuming=true;}
        reject(consuming);
        let mut missing=rows.clone();
        if let ReferenceData::LexiconEntry{items,..}=&mut missing[1].data {items[0].quick_info.key="unrelated.key".into();}
        reject(missing);
    }
}
