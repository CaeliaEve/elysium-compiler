//! The WitcheryExtras tile Mixin differs from OvenRecipe.isInputMatch and its NEI helper.
use super::{Consumption,Item,Kind,Match,OutputRole,Quantity,Recipe};
use super::witchery::{JarMode,OvenByproduct};
use crate::identity::Nbt;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="NativeOvenSelector")]
#[serde(deny_unknown_fields)]
pub struct Selector {pub registry:String,pub metadata:i32,pub nbt:Option<Nbt>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="NativeOvenPrototype")]
#[serde(deny_unknown_fields)]
pub struct Prototype {pub registry:String,pub metadata:i32,pub nbt:Option<Nbt>,pub count:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="NativeOvenDefinition")]
#[serde(deny_unknown_fields)]
pub struct Definition {pub input:Option<Selector>,pub jars:i32,pub gate:Option<Prototype>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="NativeOvenSmelting")]
#[serde(deny_unknown_fields)]
pub struct Smelting {pub metadata:i32,pub output:Prototype}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="NativeOvenProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NativeOven {
    pub version:String,pub selected:u32,pub definitions:Vec<Definition>,pub furnace:Vec<Smelting>,
    pub jar_mode:JarMode,pub jar_stock:u32,pub filtered_double:bool,pub byproduct:OvenByproduct,
    pub byproduct_item:Option<String>,pub witchery_sapling:String,pub input_name:Option<String>,pub roles:BTreeMap<String,String>,
}
fn tags(a:Option<&Nbt>,b:Option<&Nbt>)->bool {match (a,b){(None,None)=>true,(Some(a),Some(b))=>super::matching::native_tag(a,b,true),_=>false}}
pub fn matches(s:&Selector,o:&Item)->bool {
    s.registry==o.registry&&(s.metadata==32767||o.meta==32767||s.metadata==o.meta)&&tags(s.nbt.as_ref(),o.nbt.as_ref())
}
pub fn select(p:&NativeOven,o:&Item,jars:i32)->Option<usize>{p.definitions.iter().position(|d|d.jars<=jars&&d.input.as_ref().is_some_and(|s|matches(s,o)))}
pub fn smelt<'a>(p:&'a NativeOven,o:&Item)->Option<&'a Prototype>{p.furnace.iter().find(|d|d.metadata==32767||d.metadata==o.meta).map(|d|&d.output)}
#[derive(Clone,Debug)]
pub struct MainTransition {pub selected:Option<usize>,pub admitted:bool,pub output:Option<Prototype>,pub input_debit:i32}
/// Limits are runtime inventory/item callbacks, not invented constants or a guarantee of placement.
/// The Mixin preflights its registered gate, then may fail to merge the DIFFERENT furnace result.
pub fn main_transition(p:&NativeOven,input:&Item,jars:i32,current:Option<&Prototype>,inventory_limit:i32,gate_stack_limit:i32)->Result<MainTransition>{
    let selected=select(p,input,jars);
    let blocked=||MainTransition{selected,admitted:false,output:current.cloned(),input_debit:0};
    let Some(index)=selected else{return Ok(blocked());};
    if let Some(current)=current{
        let gate=p.definitions[index].gate.as_ref().context("native canSmelt would dereference a null registered gate")?;
        if !same_item(current,gate){return Ok(blocked());}
        let count=current.count.wrapping_add(gate.count);
        if count>inventory_limit||count>gate_stack_limit{return Ok(blocked());}
    }
    let native=smelt(p,input).context("native smeltItem would dereference a null furnace result")?;
    let output=match current{
        None=>Some(native.clone()),
        Some(existing)if same_item(existing,native)=>{let mut result=existing.clone();result.count=result.count.wrapping_add(native.count);Some(result)},
        Some(existing)=>Some(existing.clone()), // Completion still takes the input and attempts byproduct placement.
    };
    Ok(MainTransition{selected,admitted:true,output,input_debit:1})
}
fn same_item(a:&Prototype,b:&Prototype)->bool{a.registry==b.registry&&a.metadata==b.metadata}
/// Contributions must be in the three native direction-query positions, preserving f64 addition order.
pub fn byproduct_transition(p:&NativeOven,roll:f64,contributions:&[f64],jar_present:bool,product:Option<&Prototype>,current:Option<&Prototype>,current_stack_limit:i32)->Result<(Option<Prototype>,i32)>{
    let filtered=if p.filtered_double{0.8}else{0.3};
    ensure!(roll.is_finite()&&(0.0..1.0).contains(&roll)&&contributions.len()==3
        &&contributions.iter().all(|v|*v==0.0||*v==0.25||*v==filtered),"invalid native oven RNG/funnel observation");
    let mut extra=0.0;for value in contributions{extra+=value;}
    if !jar_present||roll>(0.3+extra).min(1.0){return Ok((current.cloned(),0));}
    let Some(product)=product else{return Ok((current.cloned(),0));};
    match current{
        None=>Ok((Some(product.clone()),1)),
        Some(existing)if same_item(existing,product)&&existing.count.wrapping_add(product.count)<current_stack_limit=>{
            let mut result=existing.clone();result.count=result.count.wrapping_add(product.count);Ok((Some(result),1))
        },
        Some(existing)=>Ok((Some(existing.clone()),0)),
    }
}
pub fn cook_ticks(attached:u32)->Result<u32>{ensure!(attached<=3,"native oven has only three queried funnel positions");Ok(180-20*attached)}
fn product(p:&NativeOven,item:&Item)->OvenByproduct{
    use OvenByproduct::*;
    if item.registry=="minecraft:sapling"&&item.meta!=3{return match item.meta{0=>Exhale,1=>Hint,2=>Breath,_=>None};}
    if item.registry==p.witchery_sapling{return match item.meta{0=>Whiff,1=>Reek,2=>Odour,_=>None};}
    if p.input_name.as_deref()==Some("tile.bop.saplings")&&item.meta==6{return Hint;}
    super::witchery::oven_nbt_product(item.nbt.as_ref())
}
fn prototype(p:&Prototype)->Result<()>{crate::identity::registry_name(&p.registry)?;ensure!(p.metadata>=0,"negative native oven metadata");if let Some(n)=&p.nbt{ensure!(matches!(n,Nbt::Compound{..}),"native oven NBT must be a compound");n.validate()?;}Ok(())}
pub(super) fn validate(p:&NativeOven,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    ensure!(p.version=="1.3.8"&&r.source.owner=="WitcheryExtras"&&r.source.handler=="alkalus.main.nei.NEI_Handler_Oven"
        &&r.duration.is_none()&&r.energy.is_none()&&r.grid.is_none()&&r.magic.is_none()&&r.properties.is_empty(),"invalid actual native oven origin or fixed cost");
    ensure!(p.definitions.len()<=4096&&p.furnace.len()<=65536&&p.jar_stock<=i32::MAX as u32
        &&p.input_name.as_ref().is_none_or(|n|n.len()<=65536),"native oven context exceeds budget");
    for d in &p.definitions{if let Some(s)=&d.input{crate::identity::registry_name(&s.registry)?;ensure!(s.metadata>=0,"negative oven source metadata");if let Some(n)=&s.nbt{ensure!(matches!(n,Nbt::Compound{..}),"oven source tags must be a compound");n.validate()?;}}if let Some(g)=&d.gate{prototype(g)?;}}
    for d in &p.furnace{ensure!(d.metadata>=0,"negative furnace source metadata");prototype(&d.output)?;}
    crate::identity::registry_name(&p.witchery_sapling)?;
    let d=p.definitions.get(p.selected as usize).context("oven selected registration missing")?;
    ensure!(r.inputs.len()==if p.jar_mode==JarMode::Absent{1}else{2},"invalid oven specimen roles");
    let input=&r.inputs[0];ensure!(input.kind==Kind::Item&&input.slot==0&&input.choices.len()==1,"invalid oven input specimen");
    let c=&input.choices[0];ensure!(c.amount=="1"&&matches!(c.rule,Match::Exact)&&matches!(c.consume,Consumption::Consume)&&c.returns.is_empty(),"oven concrete specimen changed");
    let offered=items.get(c.id.as_str()).context("oven offered item missing")?;
    ensure!(select(p,offered,p.jar_stock as i32)==Some(p.selected as usize),"oven specimen is not selected by the actual Mixin");
    let keys=["jar","exhale","hint","breath","whiff","reek","odour","foul"];
    ensure!(p.roles.len()==keys.len(),"oven role identities differ");
    for key in keys{let id=p.roles.get(key).context("oven role omitted")?;ensure!(id.starts_with("item_")&&items.contains_key(id.as_str()),"oven role fact omitted");}
    if p.jar_mode==JarMode::Absent{ensure!(p.jar_stock==0&&d.jars<=0&&p.byproduct==OvenByproduct::None&&p.byproduct_item.is_none(),"absent oven jars acquired a byproduct");}
    else{
        ensure!(p.jar_stock==(d.jars.max(1) as u32),"oven canonical jar-stock specimen differs");
        let jar=&r.inputs[1];ensure!(jar.kind==Kind::Item&&jar.slot==1&&jar.choices.len()==1,"oven jar slot differs");let c=&jar.choices[0];
        ensure!(Some(&c.id)==p.roles.get("jar")&&c.amount==p.jar_stock.to_string()&&matches!(c.rule,Match::Wildcard{meta:false,nbt:true})&&matches!(c.consume,Consumption::Allocated)&&c.returns.is_empty(),"oven jar specimen/debit differs");
        ensure!(p.byproduct==product(p,offered),"oven native byproduct differs from input tags/role");
    }
    let main=smelt(p,offered).context("reachable oven specimen would fail in native smeltItem")?;
    let o=r.outputs.first().context("oven main specimen missing")?;let fact=items.get(o.id.as_str()).context("oven main item missing")?;
    ensure!(main.count>0&&fact.registry==main.registry&&fact.meta==main.metadata&&tags(fact.nbt.as_ref(),main.nbt.as_ref()),"oven main result must come from native furnace");
    potential(o,0,"witchery:ovenMain","nativeOutputInventory",main.count)?;
    let key=match p.byproduct{OvenByproduct::None=>None,OvenByproduct::Foul=>Some("foul"),OvenByproduct::Exhale=>Some("exhale"),OvenByproduct::Hint=>Some("hint"),OvenByproduct::Breath=>Some("breath"),OvenByproduct::Whiff=>Some("whiff"),OvenByproduct::Reek=>Some("reek"),OvenByproduct::Odour=>Some("odour")};
    if let Some(key)=key{ensure!(p.jar_mode==JarMode::Conditional&&p.byproduct_item.as_ref()==p.roles.get(key)&&r.outputs.len()==2,"oven byproduct role differs");let o=&r.outputs[1];ensure!(Some(&o.id)==p.byproduct_item.as_ref(),"oven byproduct sample differs");potential(o,1,"witchery:ovenByproduct","nativeJarRollAndOutputAccepts",1)?;}
    else{ensure!(r.outputs.len()==1&&p.byproduct_item.is_none(),"oven invented a byproduct");}
    Ok(())
}
fn potential(o:&super::Output,slot:u32,stat:&str,condition:&str,count:i32)->Result<()>{
    ensure!(o.kind==Kind::Item&&o.slot==slot&&o.role==OutputRole::Result&&o.amount.is_none()&&o.change.is_none()
        &&o.chance.numerator=="1"&&o.chance.denominator=="1"&&matches!(&o.quantity,Some(Quantity::Potential{nominal,stat:s,condition:Some(c),sample:None,parameters:None})if nominal==&count.to_string()&&s==stat&&c==condition),"invalid native oven conditional output");Ok(())
}
