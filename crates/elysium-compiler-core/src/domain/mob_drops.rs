//! Mobs Info client reference rules. These are not guaranteed outputs or a world-state simulator.
use super::{references::{GeneticReferenceRow, ReferenceData}, Item};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct MobSpawn { pub native_class: String, pub description: String }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct MobDefinition {
    pub entity: String, pub display_entity: String, pub entity_class: String,
    pub name: String, #[serde(rename="mod")] pub mod_name: String, pub max_health: String,
    pub infernal_type: i8, pub peaceful: bool, pub vial: bool, pub visible: bool,
    pub drops: u32, pub damage_weight_total: u32, pub representations: Vec<String>,
    pub spawn_known: bool, pub spawns: Vec<MobSpawn>, pub additional_info: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum MobDropType { Normal, Rare, Additional, Infernal }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MobDamage { pub meta: u32, pub weight: u32 }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct MobChanceModifier {
    pub native_class: String, pub priority: i32, pub description: Vec<String>, pub payload: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct MobDrop {
    pub entity: String, pub drop: u32, pub item: String, pub amount: String,
    pub drop_type: MobDropType, pub base_chance: u32, pub chance_scale: u32,
    pub variable_chance: bool, pub lootable: bool, pub player_only: bool,
    pub enchantment_power: Option<u32>, pub damages: Option<Vec<MobDamage>>,
    pub modifiers: Vec<MobChanceModifier>, pub additional_info: Vec<String>,
}

fn text(value: &str) -> Result<()> {
    ensure!(!value.is_empty() && value.len()<=262144, "Missing or oversized native mob text"); Ok(())
}
fn lines(values: &[String]) -> Result<()> {
    ensure!(values.len()<=4096, "Native mob description exceeds line budget");
    ensure!(values.iter().all(|value|value.len()<=262144),"Native mob description exceeds text budget"); Ok(())
}

pub fn validate(id: &str, rows: &[&GeneticReferenceRow], items: &BTreeMap<&str, &Item>) -> Result<()> {
    let first=rows.first().context("Missing native mob catalogue")?;
    let ReferenceData::MobDropsDefinition {source,mobs,drops,hidden_mode,ready,consuming}=&first.data
        else {anyhow::bail!("Native mob definition must be first")};
    ensure!(*ready && !consuming && (1..=16384).contains(mobs) && *drops<=262144
        && rows.len()==1+*mobs as usize+*drops as usize, "Unready, consuming or incomplete mob catalogue");
    ensure!(source.handler=="com.kuba6000.mobsinfo.nei.MobHandler" && !source.owner.is_empty()
        && !source.key.is_empty(), "Wrong native mob catalogue origin");
    let data: Vec<_>=rows.iter().map(|row|&row.data).collect();
    ensure!(id==super::value_id("reference",&serde_json::json!({"kind":"mobsInfoDrops","records":data}))?,
        "Native mob catalogue identity differs");
    let mut cursor=1usize; let mut seen=BTreeSet::new(); let mut actual=0u64;
    for _ in 0..*mobs {
        let row=rows.get(cursor).context("Missing native mob definition")?;
        let ReferenceData::MobDefinition(mob)=&row.data else {anyhow::bail!("Native mob page is missing")};
        cursor+=1;
        for value in [&mob.entity,&mob.display_entity,&mob.entity_class,&mob.name,&mob.mod_name] {text(value)?;}
        ensure!(seen.insert(mob.entity.as_str()) && (-1..=2).contains(&mob.infernal_type)
            && (*hidden_mode || mob.visible), "Duplicate mob or contradictory discovery/eligibility");
        let health=mob.max_health.parse::<f32>().context("Invalid native mob health")?;
        ensure!(health.is_finite() && health>0.0, "Invalid native mob health");
        ensure!(mob.representations.len()<=4096 && mob.spawns.len()<=16384
            && (mob.spawn_known || mob.spawns.is_empty()), "Invalid native mob information bounds");
        for item in &mob.representations {ensure!(items.contains_key(item.as_str()),"Native mob representation missing");}
        for spawn in &mob.spawns {text(&spawn.native_class)?;text(&spawn.description)?;}
        lines(&mob.additional_info)?; let mut damage_total=0u64;
        for index in 0..mob.drops {
            let row=rows.get(cursor).context("Native mob lost a drop record")?;
            let ReferenceData::MobDrop(drop)=&row.data else {anyhow::bail!("Mixed native mob domain")};
            cursor+=1; actual+=1;
            ensure!(drop.entity==mob.entity && drop.drop==index && items.contains_key(drop.item.as_str())
                && drop.chance_scale==10000 && drop.base_chance<=i32::MAX as u32,
                "Native drop identity, order, chance scale or item differs");
            let amount=drop.amount.parse::<u32>().context("Invalid native drop amount")?;
            ensure!(amount>0 && amount<=i32::MAX as u32 && amount.to_string()==drop.amount,
                "Invalid native drop amount");
            ensure!(drop.enchantment_power.is_none_or(|value|value<=i32::MAX as u32)
                && drop.modifiers.len()<=256 && (drop.variable_chance || drop.modifiers.is_empty()),
                "Invalid native enchantment/chance policy");
            if let Some(damages)=&drop.damages {
                ensure!(!damages.is_empty() && damages.len()<=65536,"Missing native damage distribution");
                let mut metadata=BTreeSet::new();
                for damage in damages {
                    ensure!(metadata.insert(damage.meta) && damage.meta<=i32::MAX as u32 && damage.weight>0
                        && damage.weight<=i32::MAX as u32,"Invalid native damage weight");
                    damage_total+=u64::from(damage.weight);
                }
            }
            for modifier in &drop.modifiers {
                text(&modifier.native_class)?;lines(&modifier.description)?;
                ensure!(modifier.payload.len()<=131072 && modifier.payload.len()%2==0
                    && modifier.payload.bytes().all(|byte|byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
                    "Invalid native modifier payload");
            }
            lines(&drop.additional_info)?;
        }
        ensure!(damage_total==u64::from(mob.damage_weight_total),"Native shared damage weight total differs");
    }
    ensure!(cursor==rows.len() && actual==u64::from(*drops),"Native mob catalogue lost records"); Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_native_mob_rules_survive_full_domain_validation() {
        let fixtures=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw:serde_json::Value=serde_json::from_slice(&std::fs::read(fixtures.join("mob-drops-contract.json")).unwrap()).unwrap();
        let rows:Vec<GeneticReferenceRow>=serde_json::from_value(raw["records"].clone()).unwrap();
        let source=crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain=super::super::Domain::load(&source).unwrap();
        let base=serde_json::to_value(&domain.items[0]).unwrap();
        for raw in raw["items"].as_array().unwrap() {
            let mut value=base.clone();
            for key in ["id","registry","meta","nbt","tags"] {value[key]=raw[key].clone();}
            let item:Item=serde_json::from_value(value).unwrap();domain.items.retain(|old|old.id!=item.id);domain.items.push(item);
        }
        let ReferenceData::MobDropsDefinition {source:origin,..}=&rows[0].data else {panic!()};
        let mut category=domain.categories[0].clone();category.source=origin.clone();
        category.id=super::super::origin_id("category",origin).unwrap();category.reference=Some(rows[0].domain.clone());
        category.program=None;category.icon=None;category.machines.clear();
        domain.categories.push(category);domain.references=rows.clone();domain.validate(&source).unwrap();
        let items=domain.items.iter().map(|item|(item.id.as_str(),item)).collect();
        let reject=|mut changed:Vec<GeneticReferenceRow>| {
            let data:Vec<_>=changed.iter().map(|row|&row.data).collect();
            let id=super::super::value_id("reference",&serde_json::json!({"kind":"mobsInfoDrops","records":data})).unwrap();
            for row in &mut changed {row.domain=id.clone();row.id=row.content_id().unwrap();}
            assert!(super::super::references::validate(&changed,&items).is_err());
        };
        let mut changed=rows.clone();changed.pop();reject(changed);
        let mut changed=rows.clone();if let ReferenceData::MobDrop(drop)=&mut changed[2].data {drop.damages.as_mut().unwrap()[0].weight+=1;}reject(changed);
        let mut changed=rows.clone();if let ReferenceData::MobDropsDefinition{ready,..}=&mut changed[0].data {*ready=false;}reject(changed);
        let mut changed=rows.clone();if let ReferenceData::MobDrop(drop)=&mut changed[2].data {drop.entity="unrelated".into();}reject(changed);
    }
}
