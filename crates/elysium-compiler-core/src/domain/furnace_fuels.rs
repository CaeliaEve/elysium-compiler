//! Native fuel information. Smelting preview outputs are deliberately absent.
use super::{references::{GeneticReferenceRow, ReferenceData}, Item, Origin};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Scope { NativePanelFuelSamples }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ContainerPolicy { ExhaustedFuelStack }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Definition {
    pub source: Origin, pub ready: bool, pub consuming: bool, pub fuels: u32, pub scope: Scope,
    pub units_per_burn: u32, pub reference_smelt_ticks: u32, pub container_policy: ContainerPolicy,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Container { pub item: String, pub amount: String }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Candidate {
    pub item: String, pub template_amount: String, pub container: Option<Container>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_rule: Option<ContainerRule>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
#[schemars(rename="FuelFirestoneStage")]
pub enum FirestoneStage { Refined, Cracked }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
#[schemars(rename="FuelFirestoneComparison")]
pub enum FirestoneComparison {
    #[serde(rename="nextDoubleLessThanDamageFractionTimes1e-4")]
    NextDoubleLessThanDamageFractionTimes1e4,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(rename="FuelResourcePolicy")]
pub enum ResourcePolicy {
    #[serde(rename="ownerNameServerWorldZeroCreateNetworkDebitIfEnough")]
    OwnerNetwork,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag="kind", rename_all="camelCase", deny_unknown_fields)]
#[schemars(rename="FuelContainerRule")]
pub enum ContainerRule {
    RailcraftFirestone {
        stage: FirestoneStage,
        #[serde(rename="sourceMaxDamage")] source_max_damage: i32,
        #[serde(rename="degradationThreshold")] degradation_threshold: String,
        comparison: FirestoneComparison,
        #[serde(rename="damageStep")] damage_step: u32,
        #[serde(rename="degradedBase")] degraded_base: String,
        #[serde(rename="normalReturn")] normal_return: Option<Container>,
        #[serde(rename="degradedReturn")] degraded_return: Option<Container>,
    },
    BloodMagicLavaCrystal {
        #[serde(rename="energyUsed")] energy_used: i32,
        #[serde(rename="resourcePolicy")] resource_policy: ResourcePolicy,
        returned: Container,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Fuel { pub burn_ticks: i32, pub candidates: Vec<Candidate> }

pub fn validate(id: &str, rows: &[&GeneticReferenceRow], items: &BTreeMap<&str, &Item>) -> Result<()> {
    let first = rows.first().context("Missing native fuel definition")?;
    let ReferenceData::FurnaceFuelDefinition(definition) = &first.data else { anyhow::bail!("Native fuel definition must be first") };
    ensure!(definition.ready && !definition.consuming
        && definition.source.handler == "codechicken.nei.recipe.FuelRecipeHandler"
        && !definition.source.owner.is_empty() && !definition.source.key.is_empty()
        && definition.units_per_burn == 1 && definition.reference_smelt_ticks == 200, "Invalid native fuel source or furnace policy");
    ensure!(definition.fuels <= 262144 && rows.len() == definition.fuels as usize + 1, "Incomplete native fuel table");
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(id == super::value_id("reference", &serde_json::json!({"kind":"furnaceFuels","records":data}))?, "Native fuel table identity differs");
    let mut count = 0;
    for row in &rows[1..] {
        let ReferenceData::FurnaceFuel(fuel) = &row.data else { anyhow::bail!("Mixed native fuel domain") };
        ensure!(fuel.burn_ticks > 0 && (1..=4096).contains(&fuel.candidates.len()), "Invalid fuel duration or candidates");
        count += fuel.candidates.len();
        ensure!(count <= 262144, "Native fuel candidate traversal budget exceeded");
        for candidate in &fuel.candidates {
            ensure!(items.contains_key(candidate.item.as_str()), "Native fuel item fact absent");
            crate::identity::integer(&candidate.template_amount, 1, i32::MAX as i64)?;
            if let Some(container) = &candidate.container {
                ensure!(items.contains_key(container.item.as_str()), "Native fuel container fact absent");
                crate::identity::integer(&container.amount, 1, i32::MAX as i64)?;
            }
            if let Some(rule) = &candidate.container_rule {
                ensure!(candidate.container.is_none(), "Dynamic fuel also declares a fixed return");
                validate_rule(rule, items[candidate.item.as_str()], items)?;
            }
        }
    }
    Ok(())
}

fn validate_rule(rule:&ContainerRule, source:&Item, items:&BTreeMap<&str,&Item>)->Result<()> {
    match rule {
        ContainerRule::BloodMagicLavaCrystal{returned,..} =>
            check_return(Some(returned),source,source.meta,&source.nbt,items)?,
        ContainerRule::RailcraftFirestone{stage,source_max_damage,degradation_threshold,damage_step,
                degraded_base,normal_return,degraded_return,..} => {
            ensure!(*damage_step==1 && *source_max_damage==source.durability, "Native firestone durability rule differs");
            let expected=source.meta as f64 / *source_max_damage as f64 * 0.0001;
            let actual: f64=degradation_threshold.parse().context("Invalid native firestone threshold")?;
            ensure!(actual==expected || actual.is_nan() && expected.is_nan(), "Native firestone threshold differs");
            let tags=match &source.nbt {Some(crate::identity::Nbt::Compound{value})=>Some(value),None=>None,_=>anyhow::bail!("Invalid native fuel NBT root")};
            let unbreakable=super::change::number(tags.and_then(|value|value.get("Unbreakable")))? as i8 != 0;
            let normal_meta=if source.durability<=0 || unbreakable {Some(source.meta)} else {
                let damaged=source.meta.wrapping_add(1).max(0);
                (damaged<=source.durability).then_some(damaged)
            };
            if let Some(meta)=normal_meta {check_return(normal_return.as_ref(),source,meta,&source.nbt,items)?;}
            else {ensure!(normal_return.is_none(),"Broken firestone unexpectedly returns a container");}
            let base=items.get(degraded_base.as_str()).context("Native firestone degradation base absent")?;
            ensure!(base.meta==0 && base.nbt.is_none(),"Invalid native firestone degradation base");
            let (meta,nbt)=match stage {
                FirestoneStage::Cracked=>(0,None),
                FirestoneStage::Refined=>{
                    let custom=tags.and_then(|value|value.get("display")).and_then(|display|match display {
                        crate::identity::Nbt::Compound{value}=>value.get("Name"),_=>None,
                    }).and_then(|name|match name {crate::identity::Nbt::String{value}=>Some(value.clone()),_=>None});
                    let nbt=custom.map(|name|crate::identity::Nbt::Compound{value:BTreeMap::from([("display".into(),
                        crate::identity::Nbt::Compound{value:BTreeMap::from([("Name".into(),crate::identity::Nbt::String{value:name})])})])});
                    let empty=base.durability.wrapping_sub(1).max(0);
                    (if base.durability>0 {empty.wrapping_add(1).max(0)} else {empty},nbt)
                },
            };
            check_return(degraded_return.as_ref(),base,meta,&nbt,items)?;
        },
    }
    Ok(())
}
fn check_return(container:Option<&Container>, base:&Item, meta:i32, nbt:&Option<crate::identity::Nbt>, items:&BTreeMap<&str,&Item>)->Result<()> {
    let container=container.context("Native fuel return absent")?;
    let result=items.get(container.item.as_str()).context("Native fuel return fact absent")?;
    ensure!(container.amount=="1" && result.registry==base.registry && result.meta==meta && &result.nbt==nbt,
        "Native dynamic fuel return differs in count, identity, metadata or NBT");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_fuel_java_table_crosses_full_domain_and_rejects_missing_facts() {
        for name in ["fuel-reference-contract.json", "fuel-reference-semantic-contract.json", "fuel-dynamic-semantic-contract.json"] {
        let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(fixtures.join(name)).unwrap()).unwrap();
        let rows: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        for item in raw["items"].as_array().unwrap() {
            let mut value = base.clone();
            for key in ["id","registry","meta","nbt"] { value[key] = item[key].clone(); }
            if let Some(durability) = item.get("durability") { value["durability"] = durability.clone(); }
            value["icon"] = serde_json::Value::Null; value["aspects"] = serde_json::Value::Null;
            value["order"] = serde_json::Value::Null; value["tags"] = serde_json::json!([]);
            domain.items.push(serde_json::from_value(value).unwrap());
        }
        let ReferenceData::FurnaceFuelDefinition(definition) = &rows[0].data else { panic!() };
        let mut category = domain.categories[0].clone();
        category.source = definition.source.clone(); category.id = super::super::origin_id("category", &category.source).unwrap();
        category.reference = Some(rows[0].domain.clone()); category.program = None; category.icon = None; category.machines.clear();
        domain.categories.push(category); domain.references = rows.clone(); domain.validate(&source).unwrap();
        let items = domain.items.iter().map(|item| (item.id.as_str(), item)).collect();
        let check = |mut altered: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = altered.iter().map(|row| &row.data).collect();
            let id = super::super::value_id("reference", &serde_json::json!({"kind":"furnaceFuels","records":data})).unwrap();
            for row in &mut altered { row.domain = id.clone(); row.id = row.content_id().unwrap(); }
            super::super::references::validate(&altered, &items)
        };
        let mut missing = rows.clone(); missing.pop(); assert!(check(missing).is_err());
        let mut duration = rows.clone();
        if let ReferenceData::FurnaceFuel(fuel) = &mut duration[1].data { fuel.burn_ticks = -1; }
        assert!(check(duration).is_err());
        if name != "fuel-dynamic-semantic-contract.json" {
        let mut container = rows.clone();
        if let ReferenceData::FurnaceFuel(fuel) = &mut container[2].data { fuel.candidates[0].container.as_mut().unwrap().item = "item_missing".into(); }
        assert!(check(container).is_err());
        } else {
            let mut threshold=rows.clone();
            if let ReferenceData::FurnaceFuel(fuel)=&mut threshold[1].data {
                if let Some(ContainerRule::RailcraftFirestone{degradation_threshold,..})=&mut fuel.candidates[0].container_rule {
                    *degradation_threshold="0.5".into();
                }
            }
            assert!(check(threshold).is_err());
            let mut mixed=rows.clone();
            if let ReferenceData::FurnaceFuel(fuel)=&mut mixed[1].data {
                fuel.candidates[0].container=Some(Container{item:fuel.candidates[0].item.clone(),amount:"1".into()});
            }
            assert!(check(mixed).is_err());
            let mut lost_resource=rows.clone();
            if let ReferenceData::FurnaceFuel(fuel)=&mut lost_resource.last_mut().unwrap().data {
                if let Some(ContainerRule::BloodMagicLavaCrystal{returned,..})=&mut fuel.candidates[0].container_rule {returned.amount="2".into();}
            }
            assert!(check(lost_resource).is_err());
        }
        let mut empty = rows[..1].to_vec();
        if let ReferenceData::FurnaceFuelDefinition(definition) = &mut empty[0].data { definition.fuels = 0; }
        check(empty).unwrap();
        assert!(serde_json::from_value::<GeneticReferenceRow>(serde_json::json!({
            "id":rows[1].id,"domain":rows[1].domain,"order":1,
            "data":{"kind":"furnaceFuel","burnTicks":1600,"candidates":[],"fixedSmeltingOutput":"item_fake"}
        })).is_err());
        }
    }
}
