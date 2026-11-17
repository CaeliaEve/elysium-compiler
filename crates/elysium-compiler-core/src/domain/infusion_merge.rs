//! Ordered TC4Tweaks tool-query effects before NBT inheritance. Source values
//! are bounded native observations, never executable callbacks or GUI state.
use super::{change::Stack, Item, Kind, MagicKind, Output, Recipe};
use crate::identity::{item_id, integer, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Merge {pub base:Stack,pub keys:Option<Vec<String>>,pub tools:bool,pub queries:Option<BTreeMap<String,Invocation>>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Invocation {pub input:Query,pub output:Option<Query>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum Query {
    Stable{eligible:bool}, Spell{}, DarkSteel{spoon:String},
    Nanotool{capacity:Capacity,mode:Option<i32>,nonempty:Option<bool>,fallback:Option<bool>},
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(tag="kind",rename_all="lowercase",deny_unknown_fields)]
pub enum Capacity {Constant{owner:String,bits:String},Field{owner:String,bits:String},Amun{battery:Option<Box<Battery>>}}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Battery {pub defaulted:bool,pub offered_id:i32,pub offered_registry:Option<String>,pub selected_id:i32,pub selected_registry:String,pub capacity:Capacity}

const GC_BATTERY:&str="micdoodle8.mods.galacticraft.core.items.ItemBattery";
const GC_INFINITE:&str="micdoodle8.mods.galacticraft.core.items.ItemBatteryInfinite";
const GS_LEAD:&str="galaxyspace.core.item.ItemLeadBattery";
const AMUN_FIELD:&str="de.katzenpapst.amunra.item.ItemBaseBattery";

pub(super) fn validate(p:&Merge,r:&Recipe,o:&Output,items:&BTreeMap<&str,&Item>)->Result<()> {
    let change=o.change.as_ref().context("missing native inheritance change")?;
    ensure!(r.magic.as_ref().is_some_and(|m|m.kind==MagicKind::Infusion)&&o.kind==Kind::Item&&change.bindings.is_none(),"native inheritance requires an uncorrelated infusion item output");
    let input=r.inputs.iter().find(|i|i.kind==Kind::Item&&i.slot==change.input).context("missing native inheritance center")?;
    ensure!(!input.choices.is_empty()&&input.choices.len()<=65536&&change.samples.len()==input.choices.len(),"native inheritance samples omit center choices");
    let unique=input.choices.iter().map(|c|c.id.as_str()).collect::<BTreeSet<_>>();
    ensure!(if p.tools {p.queries.as_ref().is_some_and(|q|q.keys().map(String::as_str).collect::<BTreeSet<_>>()==unique)} else {p.queries.is_none()},"native tool invocations omit, add or invent a center query");
    integer(&p.base.amount,1,i32::MAX as i64)?;
    if let Some(keys)=&p.keys {ensure!(!keys.is_empty()&&keys.len()<=4096&&keys.windows(2).all(|pair|pair[0]<pair[1])&&keys.iter().all(|s|s.len()<=65535),"invalid native inheritance key filter");}
    let base=items.get(p.base.id.as_str()).context("missing native inheritance base")?;
    for (choice,sample) in input.choices.iter().zip(&change.samples) {
        integer(&choice.amount,1,i32::MAX as i64)?;
        let offered=items.get(choice.id.as_str()).context("missing native query input")?;
        let expected=project(p,offered,&choice.amount,base)?;
        ensure!(sample.id==expected.id&&sample.amount==expected.amount&&items.contains_key(sample.id.as_str()),"native inheritance sample differs from ordered query effects");
    }
    let first=change.samples.first().context("missing native inheritance first sample")?;
    ensure!(o.id==first.id&&o.amount.as_deref()==Some(first.amount.as_str()),"native inheritance default differs from first center choice");
    Ok(())
}

fn project(p:&Merge,input:&Item,amount:&str,base:&Item)->Result<Stack> {
    let mut source=State{meta:input.meta,nbt:input.nbt.clone()};
    let mut dest=State{meta:base.meta,nbt:base.nbt.clone()};
    let result=|state:&State,count:&str|->Result<Stack>{Ok(Stack{id:item_id(&base.registry,state.meta,state.nbt.as_ref())?,amount:count.to_owned()})};
    if p.tools {
        let invocation=p.queries.as_ref().and_then(|q|q.get(&input.id)).context("missing native tool invocation")?;
        let eligible=query(&invocation.input,&mut source)?;
        ensure!(eligible==(input.armor||!input.tools.is_empty()),"center tool query disagrees with native item traits");
        if !eligible {ensure!(invocation.output.is_none(),"native short-circuit still queried the output");return result(&dest,&p.base.amount);}
        let eligible=query(invocation.output.as_ref().context("eligible center omitted its output query")?,&mut dest)?;
        ensure!(eligible==(base.armor||!base.tools.is_empty()),"output tool query disagrees with native item traits");
        if !eligible {return result(&dest,&p.base.amount);}
    }
    let Some(Nbt::Compound{value:from})=&source.nbt else {ensure!(source.nbt.is_none(),"native inheritance input tag is not compound");return result(&dest,&p.base.amount);};
    if from.is_empty(){return result(&dest,&p.base.amount);}
    match &mut dest.nbt {
        Some(Nbt::Compound{value:to})=>{
            let filter=p.keys.as_ref().map(|keys|keys.iter().map(String::as_str).collect::<BTreeSet<_>>());
            super::change::merge(from,to,filter.as_ref());result(&dest,&p.base.amount)
        },
        None=>{ensure!(p.keys.is_none(),"native filtered inheritance would dereference an untagged output");dest.meta=source.meta;dest.nbt=source.nbt;result(&dest,amount)},
        _=>anyhow::bail!("native inheritance output tag is not compound"),
    }
}
struct State {meta:i32,nbt:Option<Nbt>}
fn root(nbt:&mut Option<Nbt>)->Result<&mut BTreeMap<String,Nbt>> {
    if nbt.is_none(){*nbt=Some(Nbt::Compound{value:BTreeMap::new()});}
    if let Some(Nbt::Compound{value})=nbt {Ok(value)}else{anyhow::bail!("tool query NBT root is not a compound")}
}
fn query(q:&Query,state:&mut State)->Result<bool> {
    match q {
        Query::Stable{eligible}=>Ok(*eligible),
        Query::Spell{}=>{
            let tags=root(&mut state.nbt)?;
            let Some(Nbt::Compound{value:stats})=tags.get("BloodMagicTool")else{return Ok(false);};
            for key in ["harvestLvlpickaxe","harvestLvlaxe","harvestLvlshovel"]{if stats.contains_key(key)&&super::change::number(stats.get(key))?>-1{return Ok(true);}}
            Ok(false)
        },
        Query::DarkSteel{spoon}=>{
            ensure!(spoon.len()<=256,"native spoon key exceeds budget");
            if let Some(Nbt::Compound{value})=&state.nbt {ensure!(value.get(&format!("enderio.darksteel.upgrade.{spoon}")).is_none_or(|v|matches!(v,Nbt::Compound{..})),"native spoon tag would fail its compound cast");}
            else {ensure!(state.nbt.is_none(),"native spoon query NBT root is invalid");}
            Ok(true)
        },
        Query::Nanotool{capacity:rule,mode,nonempty,fallback}=>{
            let energy=match root(&mut state.nbt)?.get("electricity"){Some(value@Nbt::Float{..})|Some(value@Nbt::Double{..})=>float(Some(value))?,_=>0.0};
            let maximum=capacity(rule,&mut state.nbt,0,&mut 0)?;
            state.meta=((100.0_f32-(energy/maximum)*100.0_f32) as i32).max(0);
            if energy>0.0 {
                let expected=super::change::number(root(&mut state.nbt)?.get("toolMode"))?;
                ensure!((0..7).contains(&expected)&&*mode==Some(expected)&&fallback.is_none(),"native nanotool mode observation is missing or stale");
                nonempty.context("native nanotool mode lacks its class-set observation")
            }else{ensure!(mode.is_none()&&nonempty.is_none(),"nonpositive native energy still queried a mode");fallback.context("native nanotool fallback was omitted")}
        },
    }
}
fn capacity(rule:&Capacity,nbt:&mut Option<Nbt>,depth:usize,nodes:&mut usize)->Result<f32> {
    *nodes+=1;ensure!(depth<=16&&*nodes<=1024,"native battery program exceeds budget");
    match rule {
        Capacity::Constant{owner,bits:value}=>{
            let expected=match owner.as_str(){GC_BATTERY=>15000.0_f32,GC_INFINITE=>f32::INFINITY,GS_LEAD=>50000.0_f32,_=>anyhow::bail!("unknown constant native capacity owner")};
            let actual=bits(value)?;ensure!(actual.to_bits()==expected.to_bits(),"constant native battery capacity changed");Ok(actual)
        },
        Capacity::Field{owner,bits:value}=>{ensure!(owner==AMUN_FIELD,"unknown native capacity field owner");bits(value)},
        Capacity::Amun{battery}=>{
            let tags=root(nbt)?;
            if tags.contains_key("maxEnergy"){ensure!(battery.is_none(),"cached native capacity still loaded a battery");return float(tags.get("maxEnergy"));}
            let binding=battery.as_ref().context("uncached AmunRa capacity omitted its battery source")?;
            let stored=match tags.get("batteryStack"){Some(Nbt::Compound{value})=>Some(value),_=>None};
            let id=super::change::number(stored.and_then(|v|v.get("id")))? as i16 as i32;
            ensure!(id==binding.offered_id&&binding.selected_id>=0&&!binding.selected_registry.is_empty()&&binding.selected_registry.len()<=256,"invalid native numeric battery binding");
            ensure!(if binding.defaulted{binding.offered_registry.is_none()}else{binding.offered_id==binding.selected_id&&binding.offered_registry.as_deref()==Some(binding.selected_registry.as_str())},"native battery default/registry observation disagrees");
            let attached=!binding.defaulted&&matches!(stored.and_then(|v|v.get("tag")),Some(Nbt::Compound{..}));
            let mut child=if attached{stored.and_then(|v|v.get("tag")).cloned()}else{None};
            let value=capacity(&binding.capacity,&mut child,depth+1,nodes)?;
            if attached {if let Some(Nbt::Compound{value:stored})=tags.get_mut("batteryStack"){stored.insert("tag".into(),child.context("native loaded child lost its attached compound")?);}}
            tags.insert("maxEnergy".into(),Nbt::Float{value:format!("{:08x}",value.to_bits())});Ok(value)
        },
    }
}
fn bits(value:&str)->Result<f32>{ensure!(value.len()==8&&value.bytes().all(|v|v.is_ascii_digit()||(b'a'..=b'f').contains(&v)),"invalid native float bits");Ok(f32::from_bits(u32::from_str_radix(value,16)?))}
fn float(value:Option<&Nbt>)->Result<f32>{Ok(match value {
    Some(Nbt::Byte{value})|Some(Nbt::Short{value})|Some(Nbt::Int{value})|Some(Nbt::Long{value})=>value.parse::<i64>()? as f32,
    Some(Nbt::Float{value})=>bits(value)?,Some(Nbt::Double{value})=>f64::from_bits(u64::from_str_radix(value,16)?) as f32,_=>0.0,
})}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn item(registry:&str,nbt:serde_json::Value,tool:bool)->Item{let tag:Option<Nbt>=serde_json::from_value(nbt.clone()).unwrap();serde_json::from_value(json!({
        "id":item_id(registry,4,tag.as_ref()).unwrap(),"registry":registry,"meta":4,"nbt":nbt,"name":"test","tooltip":[],"stackLimit":64,"durability":0,"tools":if tool{json!({"axe":0})}else{json!({})},"armor":false,"tags":[]
    })).unwrap()}
    #[test]fn ineligible_output_keeps_its_query_writes_without_merging(){
        let input=item("fixture:input",json!({"type":"compound","value":{"BloodMagicTool":{"type":"compound","value":{"harvestLvlaxe":{"type":"int","value":"0"}}}}}),true);
        let base=item("fixture:output",serde_json::Value::Null,false);
        let p=Merge{base:Stack{id:base.id.clone(),amount:"1".into()},keys:Some(vec!["unrelated".into()]),tools:true,queries:Some(BTreeMap::from([(input.id.clone(),Invocation{input:Query::Spell{},output:Some(Query::Spell{})})]))};
        let got=project(&p,&input,"3",&base).unwrap();let empty=Nbt::Compound{value:BTreeMap::new()};
        assert_eq!(got.id,item_id("fixture:output",4,Some(&empty)).unwrap());assert_eq!(got.amount,"1");
        let mut wrong=p.clone();wrong.queries.as_mut().unwrap().get_mut(&input.id).unwrap().output=None;assert!(project(&wrong,&input,"3",&base).is_err());
    }
    #[test]fn false_input_never_evaluates_or_mutates_output(){
        let input=item("fixture:input",serde_json::Value::Null,false);let base=item("fixture:output",serde_json::Value::Null,false);
        let mut p=Merge{base:Stack{id:base.id.clone(),amount:"2".into()},keys:Some(vec!["filter".into()]),tools:true,queries:Some(BTreeMap::from([(input.id.clone(),Invocation{input:Query::Spell{},output:None})]))};
        assert_eq!(project(&p,&input,"1",&base).unwrap().id,base.id);
        p.queries.as_mut().unwrap().get_mut(&input.id).unwrap().output=Some(Query::Spell{});assert!(project(&p,&input,"1",&base).is_err());
    }
    #[test]fn nested_capacity_preserves_attached_native_tag_alias(){
        let rule:Query=serde_json::from_value(json!({"kind":"nanotool","mode":1,"nonempty":true,"fallback":null,"capacity":{"kind":"amun","battery":{
            "defaulted":false,"offeredId":30001,"offeredRegistry":"fixture:nano","selectedId":30001,"selectedRegistry":"fixture:nano","capacity":{"kind":"amun","battery":{
                "defaulted":true,"offeredId":0,"offeredRegistry":null,"selectedId":30000,"selectedRegistry":"fixture:battery","capacity":{"kind":"constant","owner":GC_BATTERY,"bits":"466a6000"}
            }}}}})).unwrap();
        let mut state=State{meta:7,nbt:Some(serde_json::from_value(json!({"type":"compound","value":{"electricity":{"type":"float","value":"3f800000"},"toolMode":{"type":"int","value":"1"},"batteryStack":{"type":"compound","value":{"id":{"type":"short","value":"30001"},"tag":{"type":"compound","value":{}}}}}})).unwrap())};
        assert!(query(&rule,&mut state).unwrap());assert_eq!(state.meta,99);
        let tags=root(&mut state.nbt).unwrap();assert!(matches!(tags.get("maxEnergy"),Some(Nbt::Float{value})if value=="466a6000"));
        let Nbt::Compound{value:stored}=&tags["batteryStack"]else{panic!()};let Nbt::Compound{value:child}=&stored["tag"]else{panic!()};
        assert!(matches!(child.get("maxEnergy"),Some(Nbt::Float{value})if value=="466a6000"));
    }
}
