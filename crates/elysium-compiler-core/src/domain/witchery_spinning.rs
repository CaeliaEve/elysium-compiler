use super::{Consumption,Item,Kind,Match,OutputRole,Recipe};
use super::inventory_process::Selector;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Definition {pub fibre:Selector,pub count:i32,pub modifiers:Vec<Selector>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Spinning {pub version:String,pub selected:u32,pub definitions:Vec<Definition>,pub work_ticks:u32,pub altar_power_bits:String}
pub(super) fn validate(p:&Spinning,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(p.version=="0.24.1"&&p.work_ticks==300&&p.altar_power_bits=="3f19999a"&&p.definitions.len()<=4096
        &&r.source.handler=="alkalus.main.nei.NEI_Handler_SpinningWheel"&&r.grid.is_none()&&r.magic.is_none()&&r.duration.as_deref()==Some("300")&&r.energy.is_none()&&r.properties.is_empty(),"invalid native spinning process");
    let d=p.definitions.get(p.selected as usize).context("spinning registration missing")?;
    for entry in &p.definitions{
        ensure!(entry.count>0&&entry.modifiers.len()<=3,"invalid spinning fibre/modifier definition");
        for s in std::iter::once(&entry.fibre).chain(&entry.modifiers){crate::identity::registry_name(&s.registry)?;ensure!(s.metadata>=0,"invalid literal spinning metadata");}
    }
    for prior in &p.definitions[..p.selected as usize]{
        if prior.fibre.registry!=d.fibre.registry||prior.fibre.metadata!=d.fibre.metadata||prior.count>d.count{continue;}
        let mut available:Vec<_>=d.modifiers.iter().collect();let mut matches=true;
        for need in &prior.modifiers{if let Some(at)=available.iter().position(|s|s.registry==need.registry&&s.metadata==need.metadata){available.remove(at);}else{matches=false;break;}}
        ensure!(!matches,"spinning specimen is claimed by an earlier registration");
    }
    ensure!(r.inputs.len()==1+d.modifiers.len(),"spinning input roles differ");
    for (slot,input) in r.inputs.iter().enumerate(){
        let s=if slot==0{&d.fibre}else{&d.modifiers[slot-1]};
        ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty(),"invalid spinning specimen");
        for c in &input.choices{let item=items.get(c.id.as_str()).context("spinning input fact missing")?;
          ensure!(item.registry==s.registry&&c.amount==(if slot==0{d.count.to_string()}else{"1".into()})
            &&(if s.metadata==32767{matches!(&c.rule,Match::Metadata{value:32767,nbt:true,absent}if absent.is_empty())}else{item.meta==s.metadata&&matches!(c.rule,Match::Wildcard{meta:false,nbt:true})})
            &&matches!(c.consume,Consumption::Consume)&&c.returns.is_empty(),"spinning literal predicate/consumption differs");}
    }
    ensure!(r.outputs.len()==1,"spinning output missing");let o=&r.outputs[0];
    ensure!(o.kind==Kind::Item&&o.slot==0&&o.role==OutputRole::Result&&o.amount.is_some()&&o.change.is_none()&&o.quantity.is_none()
        &&o.chance.numerator=="1"&&o.chance.denominator=="1","invalid empty-slot spinning output specimen");
    Ok(())
}
