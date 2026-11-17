//! TConstruct 1.13.57 native material information, not a consuming recipe.
use super::{references::{GeneticReferenceRow,ReferenceData},Item,OreTemplate};
use crate::identity::{integer,item_id};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ToolMaterial {
    pub material_id:i32,pub material_name:String,pub localization_string:String,
    pub localized_name:String,pub prefix_name:String,pub ability:String,
    pub harvest_level:i32,pub durability:i32,pub mining_speed:i32,pub attack:i32,
    /// Exact native IEEE754 float bits, including negative native modifiers.
    pub handle_modifier:String,pub reinforced:i32,pub stonebound:String,
    pub tip_style:String,pub primary_color:i32,pub parts:u32,
    pub bow:Option<BowMaterial>,pub arrow:Option<ArrowMaterial>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct BowMaterial {pub draw_speed:i32,pub flight_speed_max:String}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ArrowMaterial {pub mass:String,pub break_chance:String}

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum PartDisplayBinding {
    TgregInventory {version:String,material:String},
}

fn bits(value:&str)->Result<()> {
    ensure!(value.len()==8&&value.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)),"Invalid native material float bits");
    u32::from_str_radix(value,16)?;Ok(())
}
pub(super) fn validate(id:&str,rows:&[&GeneticReferenceRow],items:&BTreeMap<&str,&Item>)->Result<()> {
    let ReferenceData::ToolMaterialsDefinition{source,version,materials,parts,native_pages,consuming,..}=&rows[0].data else {anyhow::bail!("Tool materials definition must be first")};
    ensure!(source.owner=="TConstruct"&&source.handler=="tconstruct.plugins.nei.RecipeHandlerToolMaterials"&&source.key==source.handler
        &&version=="1.13.57-GTNH"&&!*consuming&&*materials>0&&*materials<=4096&&*parts<=262144&&*native_pages==*materials*2,
        "Native tool material domain origin/version/page evidence differs");
    let data:Vec<_>=rows.iter().map(|row|&row.data).collect();
    ensure!(id==super::value_id("reference",&serde_json::json!({"kind":"tconstructToolMaterials","records":data}))?,"Native material catalogue content identity differs");
    let mut definitions=BTreeMap::new();let mut positions=BTreeMap::<i32,BTreeSet<u32>>::new();let mut current=None;
    for row in &rows[1..] {match &row.data {
        ReferenceData::ToolMaterial{material}=>{
            ensure!(!material.material_name.is_empty()&&!material.localization_string.is_empty()&&material.parts>0&&material.parts<=262144,
                "Native tool material name/parts missing");
            bits(&material.handle_modifier)?;bits(&material.stonebound)?;
            if let Some(bow)=&material.bow {bits(&bow.flight_speed_max)?;}
            if let Some(arrow)=&material.arrow {bits(&arrow.mass)?;bits(&arrow.break_chance)?;}
            ensure!(definitions.insert(material.material_id,material).is_none(),"Repeated native material ID");current=Some(material.material_id);
        },
        ReferenceData::ToolMaterialPart{material_id,index,template,display,display_binding}=>{
            ensure!(current==Some(*material_id),"Native parts separated from owning material");
            let material=definitions.get(material_id).context("Missing native part material")?;
            ensure!(*index<material.parts&&positions.entry(*material_id).or_default().insert(*index),"Duplicate or out-of-range native part");
            validate_part(template,display,display_binding,items)?;
        },
        _=>anyhow::bail!("Tool material domain mixes foreign reference kinds"),
    }}
    ensure!(definitions.len()==*materials as usize&&positions.values().map(BTreeSet::len).sum::<usize>()==*parts as usize,
        "Native material catalogue incomplete");
    for (id,material) in definitions {ensure!(positions.get(&id).is_some_and(|p|p.len()==material.parts as usize),"Native material parts incomplete");}Ok(())
}
fn validate_part(template:&OreTemplate,display:&Option<String>,binding:&Option<PartDisplayBinding>,items:&BTreeMap<&str,&Item>)->Result<()> {
    item_id(&template.registry,template.meta,template.nbt.as_ref())?;
    integer(&template.amount,i32::MIN as i64,i32::MAX as i64)?;
    if let Some(display)=display {
        let fact=items.get(display.as_str()).context("Native material part display absent")?;
        let expected = if let Some(PartDisplayBinding::TgregInventory{version,material}) = binding {
            ensure!(version=="1.7.10-GTNH-1.0.28"&&template.registry.starts_with("TGregworks:tGregToolPart")
                &&template.meta>=0&&template.meta!=32767&&!material.is_empty()&&material.len()<=128,"Invalid native TGreg display binding");
            let mut tags = match &template.nbt {
                None=>BTreeMap::new(),
                Some(crate::identity::Nbt::Compound{value})=>value.clone(),
                _=>anyhow::bail!("Native part template NBT is not a compound"),
            };
            ensure!(!tags.contains_key("material"),"Native display binding overwrote a template material");
            tags.insert("material".into(),crate::identity::Nbt::String{value:material.clone()});
            Some(crate::identity::Nbt::Compound{value:tags})
        } else {template.nbt.clone()};
        ensure!(fact.registry==template.registry&&fact.nbt==expected&&fact.meta!=32767
            &&(template.meta==32767||fact.meta==template.meta),"Native material part concrete display differs");
    }else{ensure!(template.meta==32767&&binding.is_none(),"Concrete native material part display omitted");}Ok(())
}
