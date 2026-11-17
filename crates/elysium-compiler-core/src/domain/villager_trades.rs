//! Mobs Info trade-list observations, separate from consuming recipe production.
use super::{references::{GeneticReferenceRow, ReferenceData}, Item, Origin};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TradeItem {
    pub item: String,
    /// Signed Java-int observations from Mobs Info, never executable consumption amounts.
    pub sizes: Vec<i32>,
    pub enchantment_power: Option<u32>,
}

fn origin(source: &Origin) -> Result<()> {
    ensure!(source.handler == "com.kuba6000.mobsinfo.nei.VillagerTradesHandler"
        && !source.owner.is_empty() && !source.key.is_empty(), "Invalid villager reference origin");
    Ok(())
}
fn item(value: &TradeItem, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(items.contains_key(value.item.as_str()), "Villager trade item fact absent");
    ensure!(!value.sizes.is_empty() && value.sizes.len() <= 65536
        && value.sizes.windows(2).all(|v| v[0] < v[1]), "Invalid villager quantity alternatives");
    ensure!(value.enchantment_power.is_none_or(|v| v <= i32::MAX as u32), "Invalid native enchantment power");
    Ok(())
}
fn frequency(value: &str) -> Result<()> {
    // Preserve the decimal form of the original finite double, including values above one.
    // This is accumulated native offer frequency, not probability of receiving the output.
    ensure!(!value.is_empty() && value.len() <= 640, "Invalid native trade frequency");
    let parts: Vec<_> = value.split('.').collect();
    ensure!(parts.len() <= 2 && parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        && (parts[0] == "0" || !parts[0].starts_with('0'))
        && (parts.len() == 1 || !parts[1].ends_with('0')), "Non-canonical native trade frequency");
    ensure!(value.parse::<f64>()?.is_finite(), "Non-finite native trade frequency");
    Ok(())
}

pub fn validate(id: &str, rows: &[&GeneticReferenceRow], items: &BTreeMap<&str, &Item>) -> Result<()> {
    let records: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(id == super::value_id("reference", &serde_json::json!({"kind":"mobsInfoVillagerTrades","records":records}))?,
        "Villager catalogue content identity differs");
    let mut professions = BTreeSet::new();
    let mut current: Option<(i32, u32, u32, &Origin)> = None;
    for row in rows {
        match &row.data {
            ReferenceData::VillagerProfession {source, profession, name, mod_name, offers} => {
                origin(source)?;
                if let Some((_, total, seen, previous)) = current {
                    ensure!(total == seen, "Villager profession lost offers");
                    ensure!(serde_json::to_value(previous)? == serde_json::to_value(source)?, "Mixed villager source identities");
                }
                ensure!(professions.insert(*profession) && !name.is_empty() && name.len() <= 262144
                    && !mod_name.is_empty() && mod_name.len() <= 262144 && *offers <= 262144, "Invalid villager profession");
                current = Some((*profession, *offers, 0, source));
            }
            ReferenceData::VillagerOffer {source, profession, offer, native_frequency, inputs, output} => {
                origin(source)?;
                let (parent, total, seen, parent_source) = current.as_mut().context("Villager offer precedes profession")?;
                ensure!(*profession == *parent && *offer == *seen && *seen < *total
                    && serde_json::to_value(source)? == serde_json::to_value(*parent_source)?, "Villager offer order or source differs");
                frequency(native_frequency)?;
                ensure!((1..=2).contains(&inputs.len()), "Villager trade needs one or two inputs");
                for input in inputs { item(input, items)?; }
                item(output, items)?;
                *seen += 1;
            }
            _ => anyhow::bail!("Mixed villager reference domain"),
        }
    }
    let (_, total, seen, _) = current.context("Missing villager profession")?;
    ensure!(total == seen, "Villager profession lost offers");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_signed_observations_preserve_java_int_bounds() {
        let json = serde_json::json!({"item":"item_fixture","sizes":[-2147483648,-3,0,2,2147483647],"enchantmentPower":null});
        let value: TradeItem = serde_json::from_value(json.clone()).expect("native signed alternatives");
        assert_eq!(serde_json::to_value(value).unwrap(), json);
        for invalid in [-2147483649i64, 2147483648i64] {
            let mut bad = json.clone(); bad["sizes"] = serde_json::json!([invalid]);
            assert!(serde_json::from_value::<TradeItem>(bad).is_err());
        }
    }

    #[test]
    fn java_villager_trades_are_complete_typed_references() {
        let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let fixture: serde_json::Value = serde_json::from_slice(&std::fs::read(fixtures.join("villager-trades-contract.json")).unwrap()).unwrap();
        let rows: Vec<GeneticReferenceRow> = serde_json::from_value(fixture["records"].clone()).unwrap();
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        let items: Vec<Item> = fixture["items"].as_array().unwrap().iter().map(|raw| {
            let mut item = base.clone();
            for key in ["id", "registry", "meta", "nbt", "tags"] { item[key] = raw[key].clone(); }
            serde_json::from_value(item).unwrap()
        }).collect();
        let facts = items.iter().map(|item| (item.id.as_str(), item)).collect();
        super::super::references::validate(&rows, &facts).unwrap();
        let ReferenceData::VillagerProfession { source: origin, .. } = &rows[0].data else { panic!() };
        let mut category = domain.categories[0].clone();
        category.source = origin.clone(); category.id = super::super::origin_id("category", origin).unwrap();
        category.reference = Some(rows[0].domain.clone()); category.program = None; category.icon = None; category.machines.clear();
        domain.categories.push(category); domain.references = rows.clone();
        for item in items.clone() { domain.items.retain(|old| old.id != item.id); domain.items.push(item); }
        domain.validate(&source).unwrap();
        let refs: Vec<_> = rows.iter().collect();
        validate(&rows[0].domain, &refs, &facts).unwrap();
        // Rebind content identities so these tests exercise semantics, not only stale hashes.
        let reject = |mut altered: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = altered.iter().map(|row| &row.data).collect();
            let id = super::super::value_id("reference", &serde_json::json!({"kind":"mobsInfoVillagerTrades","records":data})).unwrap();
            for row in &mut altered { row.domain = id.clone(); row.id = row.content_id().unwrap(); }
            assert!(super::super::references::validate(&altered, &facts).is_err());
        };
        let mut truncated = rows.clone(); truncated.pop(); reject(truncated);
        let mut invalid = rows.clone();
        if let ReferenceData::VillagerOffer { inputs, .. } = &mut invalid[1].data { inputs[0].sizes = vec![5, 2]; }
        reject(invalid);
        let mut invalid = rows.clone();
        if let ReferenceData::VillagerOffer { native_frequency, .. } = &mut invalid[1].data { *native_frequency = "NaN".into(); }
        reject(invalid);
        let mut invalid = rows.clone();
        if let ReferenceData::VillagerOffer { profession, .. } = &mut invalid[1].data { *profession = 99; }
        reject(invalid);
    }
}
