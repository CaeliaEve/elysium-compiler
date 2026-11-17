//! Registry dispatch and getRecipeResult, explicitly after native cauldron admission.
//! This does not simulate world callbacks, nullification, or promise a valid brewing sequence.
use super::{Consumption,Fluid,Item,Kind,Match,OutputRole,Quantity,Recipe};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="NativeRitualActionKey")]
#[serde(deny_unknown_fields)]
pub struct Key {pub registry:String,pub metadata:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="NativeRitualDispatch")]
#[serde(deny_unknown_fields)]
pub struct Dispatch {pub registry:String,pub metadata:i32,pub hash:i32,pub bucket:u32,pub action:Key,pub target:u32,pub power:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="NativeRitualProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NativeRitual {
    pub version:String,pub scope:String,pub action_index:u32,pub selected:u32,
    pub trigger:Key,pub trigger_target:u32,pub trigger_power:i32,pub water_millibuckets:u32,pub capacity:u32,
    pub patterns:Vec<Vec<Key>>,pub dispatch:Vec<Dispatch>,pub names:BTreeMap<String,String>,
}
/// Java String.hashCode is over UTF-16 code units with signed 32-bit wrapping.
fn java_hash(name:&str)->i32{name.encode_utf16().fold(0i32,|hash,unit|hash.wrapping_mul(31).wrapping_add(unit.into()))}
pub fn resolve<'a>(p:&'a NativeRitual,item:&Item)->Option<&'a Dispatch>{
    let hash=java_hash(p.names.get(&item.registry)?).wrapping_add(629);
    if p.capacity==0{return None;}let bucket=(hash as u32&0x7fffffff)%p.capacity;
    p.dispatch.iter().find(|d|d.bucket==bucket&&d.hash==hash&&d.registry==item.registry
        &&(d.metadata==32767||item.meta==32767||d.metadata==item.meta))
}
/// Each offered action removes ONE literal requirement; extras and duplicate needs are retained.
pub fn select(p:&NativeRitual,actions:&[Key])->Option<usize>{
    p.patterns.iter().position(|pattern|{
        let mut needed=pattern.clone();
        for key in actions{if let Some(at)=needed.iter().position(|need|need==key){needed.remove(at);}}
        needed.is_empty()
    })
}
pub fn select_items(p:&NativeRitual,items:&[&Item])->Option<usize>{
    let actions:Option<Vec<Key>>=items.iter().map(|item|resolve(p,item).map(|d|d.action.clone())).collect();
    select(p,&actions?)
}
pub fn action_power(p:&NativeRitual,items:&[&Item])->Option<i32>{
    items.iter().try_fold(0i32,|sum,item|Some(sum.wrapping_add(resolve(p,item)?.power)))
}
fn key(k:&Key)->Result<()>{crate::identity::registry_name(&k.registry)?;ensure!(k.registry.len()<=4096,"ritual key exceeds budget");Ok(())}
pub(super) fn validate(p:&NativeRitual,r:&Recipe,items:&BTreeMap<&str,&Item>,fluids:&BTreeMap<&str,&Fluid>)->Result<()>{
    ensure!(p.version=="0.24.1"&&p.scope=="postAdmissionActions"&&p.water_millibuckets==3000
        &&r.source.owner=="WitcheryExtras"&&r.source.handler=="alkalus.main.nei.NEI_Handler_Cauldron"
        &&r.duration.is_none()&&r.energy.is_none()&&r.grid.is_none()&&r.magic.is_none()&&r.properties.is_empty(),"invalid native ritual origin, scope or fixed costs");
    ensure!(p.action_index<4096&&p.patterns.len()<=4096&&p.dispatch.len()<=65536&&p.names.len()<=65536
        &&p.trigger_target<65536&&p.capacity>0&&p.capacity<=262144,"native ritual context exceeds budget");
    key(&p.trigger)?;
    let mut total=0usize;
    for pattern in &p.patterns{ensure!(pattern.len()<=255,"ritual requirements exceed budget");total+=pattern.len();for k in pattern{key(k)?;}}
    ensure!(total<=262144,"ritual cumulative requirements exceed budget");
    let mut bytes=0usize;
    for(registry,name)in &p.names{crate::identity::registry_name(registry)?;let count=name.encode_utf16().count();bytes+=count*2;
        ensure!(registry.len()<=4096&&count<=65536&&bytes<=8*1024*1024,"ritual name observations exceed budget");}
    let mut targets=BTreeMap::new();
    for d in &p.dispatch{
        crate::identity::registry_name(&d.registry)?;key(&d.action)?;
        ensure!(d.registry.len()<=4096&&d.target<65536&&d.bucket<p.capacity&&p.names.contains_key(&d.registry),"ritual dispatch lacks an observed native query name or bucket");
        if let Some((action,power))=targets.insert(d.target,(&d.action,d.power)){
            ensure!(action==&d.action&&power==d.power,"one native action identity has conflicting key or power");
        }
    }
    ensure!(targets.get(&p.trigger_target)==Some(&(&p.trigger,p.trigger_power)),"ritual trigger identity differs from registry dispatch");
    let pattern=p.patterns.get(p.selected as usize).context("ritual selected definition missing")?;
    let mut expected=pattern.clone();expected.push(p.trigger.clone());
    ensure!(select(p,&expected)==Some(p.selected as usize),"ritual specimen is shadowed by an earlier original definition");
    ensure!(r.inputs.len()==expected.len()+1,"ritual sample/water roles differ");
    for(slot,need)in expected.iter().enumerate(){
        let input=&r.inputs[slot];ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty(),"ritual item specimen missing");
        for c in &input.choices{
            ensure!(c.amount=="1"&&matches!(c.rule,Match::Wildcard{meta:false,nbt:true})
                &&matches!(c.consume,Consumption::Consume)&&c.returns.is_empty(),"ritual specimen predicate/debit differs");
            let item=items.get(c.id.as_str()).context("ritual physical item fact missing")?;
            let d=resolve(p,item).context("ritual physical specimen has no native dispatch")?;
            ensure!(&d.action==need,"ritual physical item maps to a different action key");
            if slot==pattern.len(){ensure!(d.target==p.trigger_target,"ritual trigger resolves to a different native action instance");}
        }
    }
    let water=r.inputs.last().context("ritual full water specimen missing")?;
    ensure!(water.kind==Kind::Fluid&&water.slot==0&&water.choices.len()==1,"ritual water slot differs");
    let c=&water.choices[0];let fluid=fluids.get(c.id.as_str()).context("ritual water fact missing")?;
    ensure!(fluid.registry=="water"&&fluid.nbt.is_none()&&c.amount=="3000"&&matches!(c.rule,Match::Exact)
        &&matches!(c.consume,Consumption::Consume)&&c.returns.is_empty(),"ritual full native water specimen differs");
    ensure!(r.outputs.len()==1,"ritual has invented outputs");let o=&r.outputs[0];
    ensure!(o.kind==Kind::Item&&o.slot==0&&o.role==OutputRole::Result&&o.amount.is_none()&&o.change.is_none()
        &&o.chance.numerator=="1"&&o.chance.denominator=="1","invalid ritual world-item specimen");
    let Some(Quantity::Potential{stat,nominal,condition:Some(condition),sample:None,parameters:None})=&o.quantity else{
        anyhow::bail!("ritual result must remain conditional on admitted state and world");};
    ensure!(stat=="witchery:ritualWorldItem"&&condition=="admittedActionListAndNativeWorld","ritual world condition differs");
    crate::identity::integer(nominal,1,i32::MAX.into())?;
    ensure!(items.contains_key(o.id.as_str()),"ritual result fact missing");Ok(())
}
