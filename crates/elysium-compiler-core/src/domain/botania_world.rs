//! Native Botania world-state processes. Inventory/world admission remains an explicit prerequisite.
use super::{Consumption,Item,Kind,Match,OutputRole,Recipe};
use crate::identity::Nbt;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BotaniaWorldMachine")]
#[serde(rename_all="camelCase")]
pub enum Machine{Daisy,Petal,Trade,Brewery}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BotaniaWorldSelector")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Selector{pub registry:Option<String>,pub metadata:i32,pub wildcard:bool,pub species:Option<String>,pub block:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BotaniaWorldBlock")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct BlockResult{pub registry:String,pub metadata:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BotaniaWorldProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct WorldProcess {
    pub version:String,pub machine:Machine,pub selector:u32,pub predicates:Vec<Vec<Selector>>,
    pub input_slots:Vec<Option<u32>>,pub priority:Vec<Vec<Vec<Selector>>>,pub species_dominance:BTreeMap<String,bool>,
    pub seed:Option<String>,pub container:Option<u32>,pub mana:Option<i32>,pub brew:Option<String>,pub world_result:Option<BlockResult>,
}
fn string(value:&str)->bool{!value.is_empty()&&value.len()<=512}
fn groups(source:&[Vec<Selector>],machine:Machine,budget:&mut usize)->Result<()>{
    ensure!(source.len()<=4096,"Botania world predicate group budget");
    for group in source{*budget=budget.checked_sub(group.len()).context("Botania world selector budget")?;
        for s in group {ensure!(s.registry.as_deref().is_some_and(string)||machine==Machine::Daisy&&s.block.as_deref().is_some_and(string),"Botania world predicate has no item/block identity");
            ensure!(machine==Machine::Daisy||s.block.is_none(),"World block role attached to an inventory matcher");
            ensure!(s.species.is_none()||machine==Machine::Trade&&s.wildcard&&s.species.as_deref().is_some_and(string),"Botania bee species predicate differs");
        }
    }Ok(())
}
fn compound(value:Option<&Nbt>)->Option<&BTreeMap<String,Nbt>>{match value{Some(Nbt::Compound{value})=>Some(value),_=>None}}
fn text(value:Option<&Nbt>)->Option<&str>{match value{Some(Nbt::String{value})=>Some(value),_=>None}}
fn species(item:&Item,dominance:&BTreeMap<String,bool>)->Option<String>{
    let genome=compound(compound(item.nbt.as_ref())?.get("Genome"))?;
    let Nbt::List{element,value}=genome.get("Chromosomes")? else{return None;};if element!="compound"{return None;}
    let mut selected=None;
    for row in value{let Some(row)=compound(Some(row))else{continue;};let ordinal=match row.get("Slot"){Some(Nbt::Byte{value})=>value.parse::<i8>().ok().map(i32::from),Some(Nbt::Short{value})=>value.parse::<i16>().ok().map(|n|i32::from(n as i8)),Some(Nbt::Int{value})=>value.parse::<i32>().ok().map(|n|i32::from(n as i8)),_=>Some(0)};
        if ordinal==Some(0){let a=text(row.get("UID0"))?;let b=text(row.get("UID1"))?;let left=*dominance.get(a)?;let right=*dominance.get(b)?;selected=Some(if left||!right{a}else{b}.to_owned());}
    }selected
}
fn accepts(selector:&Selector,item:&Item,dominance:&BTreeMap<String,bool>)->bool{
    selector.registry.as_deref()==Some(item.registry.as_str())&&(selector.wildcard||selector.metadata==item.meta)
        &&selector.species.as_ref().is_none_or(|uid|species(item,dominance).as_ref()==Some(uid))
}
pub(super) fn validate(p:&WorldProcess,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    let handler=match p.machine{Machine::Daisy=>"PureDaisy",Machine::Petal=>"PetalApothecary",Machine::Trade=>"ElvenTrade",Machine::Brewery=>"Brewery"};
    ensure!(p.version=="1.12.28-GTNH"&&r.source.owner=="Botania"&&r.source.handler==format!("vazkii.botania.client.integration.nei.recipe.RecipeHandler{handler}")
        &&r.grid.is_none()&&r.magic.is_none()&&r.duration.is_none()&&r.energy.is_none()&&r.properties.is_empty(),"Botania world family acquired GUI, fixed timing or generic crafting semantics");
    ensure!(p.priority.len()<=4096&&p.species_dominance.len()<=262144&&p.species_dominance.keys().all(|s|string(s)),"Botania world shared source budget");
    let mut budget=262144;groups(&p.predicates,p.machine,&mut budget)?;for prior in &p.priority{groups(prior,p.machine,&mut budget)?;}
    ensure!(p.input_slots.len()==p.predicates.len(),"Botania input-to-predicate roles differ");
    let mut slot=0u32;
    for (group,actual) in p.predicates.iter().zip(&p.input_slots){
        if actual.is_none(){ensure!(p.machine==Machine::Daisy&&!group.is_empty()&&group.iter().all(|s|s.registry.is_none()&&s.block.is_some()),"Only itemless world blocks may lack an inventory example");continue;}
        ensure!(*actual==Some(slot)&&!group.is_empty(),"Botania world input role differs");let input=r.inputs.get(slot as usize).context("Botania ingredient omitted")?;
        ensure!(input.kind==Kind::Item&&input.slot==slot&&!input.choices.is_empty(),"Botania ingredient kind/slot differs");
        for c in &input.choices{let item=items.get(c.id.as_str()).context("Botania world item fact missing")?;
            ensure!(c.amount=="1"&&matches!(c.consume,Consumption::Consume)&&c.returns.is_empty()&&group.iter().any(|s|accepts(s,item,&p.species_dominance)
                &&matches!(c.rule,Match::Wildcard{meta,nbt:true}if meta==s.wildcard)),"Botania world candidate changed its native predicate or consumption");}
        slot+=1;
    }
    if p.machine==Machine::Petal{let seed=p.seed.as_ref().context("Native seed trigger omitted")?;let input=r.inputs.get(slot as usize).context("Native seed input omitted")?;
        ensure!(input.kind==Kind::Item&&input.slot==16&&input.choices.len()==1&&input.choices[0].id==*seed&&input.choices[0].amount=="1"&&matches!(input.choices[0].consume,Consumption::Consume)
            &&matches!(input.choices[0].rule,Match::Wildcard{meta:true,nbt:true})&&input.choices[0].returns.is_empty()&&items.get(seed.as_str()).is_some_and(|i|i.registry=="minecraft:wheat_seeds")&&p.predicates.len()<=16,"Botania seed example/consumption differs");slot+=1;
    }else{ensure!(p.seed.is_none(),"Seed trigger attached to another machine");}
    if p.machine==Machine::Brewery{ensure!(p.container==Some(6)&&p.mana.is_some()&&p.brew.as_deref().is_some_and(string)&&p.predicates.len()<=6,"Botania native container/mana/brew missing");
        let input=r.inputs.get(slot as usize).context("Brew container omitted")?;ensure!(input.kind==Kind::Item&&input.slot==6&&input.choices.len()==1&&input.choices[0].amount=="1"&&matches!(input.choices[0].consume,Consumption::Consume)&&input.choices[0].returns.is_empty()&&matches!(input.choices[0].rule,Match::Wildcard{meta:false,nbt:true}),"Native brew container is consumed, not retained");slot+=1;
    }else{ensure!(p.container.is_none()&&p.mana.is_none()&&p.brew.is_none(),"Brew container state attached to another machine");}
    ensure!(r.inputs.len()==slot as usize,"Unexpected Botania world input");
    if p.machine==Machine::Daisy{ensure!(p.predicates.len()==1&&p.world_result.as_ref().is_some_and(|s|string(&s.registry))&&r.outputs.len()<=1,"Pure Daisy world result missing");}
    else{ensure!(p.world_result.is_none()&&r.outputs.len()==1,"Botania inventory result arity differs");}
    if let Some(out)=r.outputs.first(){ensure!(out.kind==Kind::Item&&out.slot==0&&matches!(out.role,OutputRole::Result)&&out.amount.is_some()&&out.quantity.is_none()&&out.change.is_none()&&out.chance.numerator=="1"&&out.chance.denominator=="1","Botania world result must remain a conditional native prototype");}
    Ok(())
}
