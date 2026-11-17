//! Forge registration catalogues. Raw templates and display specimens have distinct identities.
use super::{references::{GeneticReferenceRow,ReferenceData},Fluid,Item,Origin};
use crate::identity::{fluid_id,item_id,Nbt};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase")]
#[schemars(rename="ForgeRegistryKind")]
pub enum Registry {OreDictionary,FluidRegistry}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="ForgeRegistryDefinition")]
pub struct Definition {pub source:Origin,pub registry:Registry,pub enabled:bool,pub entries:u32,pub consuming:bool}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="ForgeRegistryStack")]
pub struct Stack {pub registry:String,pub meta:i32,pub amount:i32,pub nbt:Option<Nbt>,pub displays:Vec<String>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="ForgeOreReference")]
pub struct OreGroup {pub native_id:u32,pub name:String,pub members:Vec<Stack>,pub items:Vec<String>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="ForgeRegisteredBlock")]
pub struct Block {pub registry:String,pub display:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="ForgeFluidReference")]
pub struct FluidInfo {pub id:String,pub registry:String,pub native_id:u32,pub temperature:i32,pub luminosity:i32,
    pub density:i32,pub viscosity:i32,pub gaseous:bool,pub placeable:bool,pub block:Option<Block>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="ForgeContainerFluid")]
pub struct ContainerFluid {pub id:String,pub amount:i32,pub nbt:Option<Nbt>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="ForgeContainerReference")]
pub struct Container {pub empty:Stack,pub filled:Stack,pub fluid:ContainerFluid}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="ForgeFluidEntry")]
pub struct FluidEntry {pub fluid:FluidInfo,pub items:Vec<String>,pub containers:Vec<Container>}

fn stack(value:&Stack,items:&BTreeMap<&str,&Item>,ids:&mut BTreeSet<String>)->Result<()> {
    item_id(&value.registry,value.meta,value.nbt.as_ref())?;
    ensure!(value.displays.len()<=65536&&!value.displays.is_empty(),"Forge template has no bounded concrete display");
    for id in &value.displays {
        let item=items.get(id.as_str()).context("Forge display item fact absent")?;
        ensure!(item.registry==value.registry&&item.nbt==value.nbt&&item.meta!=32767
            &&(value.meta==32767||item.meta==value.meta),"Forge template and concrete display differ");
        ids.insert(id.clone());
    }Ok(())
}
pub(super) fn validate(id:&str,rows:&[&GeneticReferenceRow],items:&BTreeMap<&str,&Item>)->Result<()> {
    let ReferenceData::ForgeRegistryDefinition(def)=&rows[0].data else{anyhow::bail!("Forge definition must be first")};
    let suffix=if def.registry==Registry::OreDictionary{"OreDictionary"}else{"FluidRegistry"};
    ensure!(def.source.owner=="neiintegration"&&def.source.handler==format!("tonius.neiintegration.mods.mcforge.RecipeHandler{suffix}")
        &&def.source.key==def.source.handler&&!def.consuming&&def.entries<=262144&&rows.len()==1+def.entries as usize
        &&(def.enabled||def.entries==0),"Forge registry origin, enabled state or count differs");
    let data:Vec<_>=rows.iter().map(|row|&row.data).collect();
    ensure!(id==super::value_id("reference",&serde_json::json!({"kind":"forgeRegistry","records":data}))?,"Forge registry content identity differs");
    let mut names=BTreeSet::new();let mut native_ids=BTreeSet::new();let mut prior_name=None;let mut prior_id=None;
    for row in &rows[1..] {
        let mut ids=BTreeSet::new();
        match &row.data {
            ReferenceData::ForgeOreGroup(group)=>{
                ensure!(def.registry==Registry::OreDictionary&&!group.name.is_empty()&&group.name.len()<=65536
                    &&names.insert(group.name.as_str())&&native_ids.insert(group.native_id)
                    &&prior_name.is_none_or(|name:&str|name.encode_utf16().cmp(group.name.encode_utf16()).is_lt()),"Forge ore order, name or native ID differs");
                prior_name=Some(group.name.as_str());
                ensure!(group.members.len()<=262144,"Forge ore member budget exceeded");
                for member in &group.members{stack(member,items,&mut ids)?;}
                ensure!(group.items.len()==ids.len()&&group.items.iter().cloned().collect::<BTreeSet<_>>()==ids,"Forge ore display census differs");
            },
            ReferenceData::ForgeFluid(entry)=>{
                let fluid=&entry.fluid;
                ensure!(def.registry==Registry::FluidRegistry&&fluid.id==fluid_id(&fluid.registry,None)?
                    &&names.insert(fluid.registry.as_str())&&native_ids.insert(fluid.native_id)&&fluid.native_id>0
                    &&prior_id.is_none_or(|id|id<fluid.native_id)&&fluid.placeable==fluid.block.is_some(),"Forge fluid identity, native order or placeable block differs");
                prior_id=Some(fluid.native_id);
                if let Some(block)=&fluid.block {
                    ensure!(!block.registry.is_empty()&&block.registry.len()<=65536,"Forge fluid block registry absent");
                    if let Some(id)=&block.display {let item=items.get(id.as_str()).context("Forge fluid block display absent")?;
                        ensure!(item.meta==0&&item.nbt.is_none(),"Forge fluid block display state differs");ids.insert(id.clone());}
                }
                ensure!(entry.containers.len()<=262144,"Forge container budget exceeded");
                for container in &entry.containers {
                    stack(&container.empty,items,&mut ids)?;stack(&container.filled,items,&mut ids)?;
                    ensure!(container.fluid.id==fluid_id(&fluid.registry,container.fluid.nbt.as_ref())?,"Forge container fluid NBT identity differs");
                }
                ensure!(entry.items.len()==ids.len()&&entry.items.iter().cloned().collect::<BTreeSet<_>>()==ids,"Forge fluid display census differs");
            },
            _=>anyhow::bail!("Forge registry mixes foreign reference rows"),
        }
    }Ok(())
}
pub(super) fn validate_facts(rows:&[GeneticReferenceRow],fluids:&BTreeMap<&str,&Fluid>)->Result<()> {
    for row in rows {if let ReferenceData::ForgeFluid(entry)=&row.data {
        let info=&entry.fluid;let fact=fluids.get(info.id.as_str()).context("Forge fluid fact absent")?;
        ensure!(fact.registry==info.registry&&fact.nbt.is_none(),"Forge fluid fact identity differs");
        for container in &entry.containers {
            let fact=fluids.get(container.fluid.id.as_str()).context("Forge container fluid fact absent")?;
            ensure!(fact.registry==info.registry&&fact.nbt==container.fluid.nbt,"Forge container fluid fact differs");
        }
    }}Ok(())
}
