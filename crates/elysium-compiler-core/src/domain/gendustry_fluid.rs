//! Gendustry native single-item fluid processors, without GUI-derived timing or quantities.
use super::{Consumption,Fluid,Item,Kind,Match,Recipe};
use super::gendustry::DnaSource;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryFluidMachine")]
#[serde(rename_all="camelCase")]
pub enum Machine { Protein,Mutagen }

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryFluidConfig")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Config {
    pub mj_per_item:f32,pub power_use_rate:f32,pub activation_energy:f32,
    pub max_stored_energy:f32,pub max_received_energy:f32,pub tank_size:i32,
}

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryFluidProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct FluidProcess {
    pub version:String,pub machine:Machine,pub input:u32,pub fluid:String,
    pub config:Config,pub sources:Vec<DnaSource>,
}

pub(super) fn validate(p:&FluidProcess,r:&Recipe,items:&BTreeMap<&str,&Item>,fluids:&BTreeMap<&str,&Fluid>)->Result<()> {
    let (handler,registry)=match p.machine {Machine::Protein=>("Liquifier","protein"),Machine::Mutagen=>("MutagenProducer","mutagen")};
    ensure!(p.version=="1.9.4-GTNH"&&r.source.handler==format!("net.bdew.gendustry.nei.{handler}Handler")
        &&r.duration.is_none()&&r.energy.is_none()&&r.grid.is_none()&&r.magic.is_none(),"Native Gendustry fluid process acquired fixed crafting/timing");
    let c=&p.config;
    ensure!([c.mj_per_item,c.power_use_rate,c.activation_energy,c.max_stored_energy,c.max_received_energy].iter().all(|n|n.is_finite())
        &&c.mj_per_item>0.0&&c.power_use_rate>=0.0&&c.activation_energy>=0.0&&c.tank_size>0,"Invalid native Gendustry fluid configuration");
    ensure!(r.inputs.len()==1&&r.outputs.len()==1,"Native Gendustry fluid processor requires one source and one result");
    let input=&r.inputs[0];ensure!(input.kind==Kind::Item&&input.slot==p.input&&input.choices.len()==1,"Native Gendustry source role differs");
    let choice=&input.choices[0];ensure!(choice.amount=="1"&&choice.returns.is_empty()&&matches!(choice.consume,Consumption::Consume),"Native Gendustry uses one item without crafting returns");
    let amount=source_amount(&p.sources,choice,items)?;
    let output=&r.outputs[0];let fluid=fluids.get(output.id.as_str()).context("Native Gendustry fluid fact missing")?;
    ensure!(output.id==p.fluid&&fluid.registry==registry&&fluid.nbt.is_none()&&output.kind==Kind::Fluid
        &&output.slot==0&&output.amount.as_deref()==Some(amount.to_string().as_str())&&amount<=c.tank_size
        &&output.quantity.is_none()&&output.change.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1",
        "Native Gendustry fluid amount, identity or complete-batch capacity differs");
    Ok(())
}

pub(super) fn source_amount(sources:&[DnaSource],choice:&super::Choice,items:&BTreeMap<&str,&Item>)->Result<i32> {
    ensure!(sources.len()<=65536,"Gendustry source table exceeds budget");
    let mut keys=BTreeSet::new();for source in sources {ensure!(!source.registry.is_empty()&&keys.insert((&source.registry,source.meta)),"Duplicate or empty Gendustry source key");}
    let specimen=items.get(choice.id.as_str()).context("Native Gendustry source specimen missing")?;
    let source=sources.iter().find(|s|s.registry==specimen.registry&&s.meta==specimen.meta)
        .or_else(||sources.iter().find(|s|s.registry==specimen.registry&&s.meta==32767)).context("Native source predicate has no matching table entry")?;
    ensure!(source.amount>0,"A rejected source entry cannot become a positive machine operation");
    if source.meta!=32767 {ensure!(matches!(choice.rule,Match::Wildcard{meta:false,nbt:true}),"Native exact source predicate differs");}
    else {
        let overrides:Vec<_>=sources.iter().filter(|s|s.registry==source.registry&&s.meta!=32767).collect();
        match &choice.rule {
            Match::Wildcard{meta:true,nbt:true}=>ensure!(overrides.is_empty(),"Native wildcard omitted exact overrides"),
            Match::Except{base,exclude}=>{
                ensure!(matches!(base.as_ref(),Match::Wildcard{meta:true,nbt:true})&&exclude.len()==overrides.len(),"Native wildcard source partition differs");
                let mut seen=BTreeSet::new();for branch in exclude {
                    let item=items.get(branch.id.as_str()).context("Native source override fact missing")?;
                    ensure!(item.registry==source.registry&&seen.insert(item.meta)&&overrides.iter().any(|s|s.meta==item.meta)
                        &&matches!(branch.rule,Match::Wildcard{meta:false,nbt:true}),"Native exact override predicate differs");
                }
            },_=>anyhow::bail!("Native wildcard source predicate differs"),
        }
    }
    Ok(source.amount)
}
