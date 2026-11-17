//! Native writing/soldering inventories have their own allocation and conditional result laws.
use super::{Consumption,Item,Kind,Match,OutputRole,Quantity,Recipe};
use crate::identity::integer;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="InventoryItemSelector")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Selector {pub registry:String,pub metadata:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="InventoryItemPrototype")]
#[serde(deny_unknown_fields)]
pub struct Prototype {pub id:String,pub amount:String}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BloodAlchemyPriority")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AlchemyPriority {pub minimum_orb:i32,pub templates:[Option<Selector>;5]}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BloodAlchemyOrb")]
#[serde(deny_unknown_fields)]
pub struct Orb {pub registry:String,pub level:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BloodAlchemyProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Alchemy {
    pub version:String,pub templates:[Option<Selector>;5],pub earlier:Vec<AlchemyPriority>,pub minimum_orb:i32,
    pub orbs:Vec<Orb>,pub orb:u32,pub lp_per_progress:i32,pub work_progress:u32,pub accelerated_step:u32,
    pub containers:Vec<Vec<Option<Prototype>>>,pub result:Prototype,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="SolderingDefinition")]
#[serde(deny_unknown_fields)]
pub struct SolderingDefinition {pub inputs:[Option<Selector>;9],pub result:Selector}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="SolderingResultTag")]
#[serde(rename_all="camelCase")]
pub enum ResultTag {Prototype,FreshRandomUuid}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="SolderingSolderPredicate")]
#[serde(rename_all="camelCase")]
pub enum SolderPredicate {AnyItem}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="LogisticsSolderingProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Soldering {
    pub version:String,pub selected:u32,pub definitions:Vec<SolderingDefinition>,pub solder_slot:u32,
    pub solder_predicate:SolderPredicate,pub result:Prototype,pub result_tag:ResultTag,pub heat:u32,
    pub work_progress:u32,pub power_disabled:bool,
}
fn selector(s:&Selector)->Result<()>{ensure!(!s.registry.is_empty()&&s.registry.len()<=512&&s.metadata>=-1,"Invalid native inventory selector");Ok(())}
fn base(r:&Recipe)->Result<()>{ensure!(r.duration.is_none()&&r.energy.is_none()&&r.magic.is_none()&&r.properties.is_empty(),"Native inventory uses state rules without fixed elapsed duration or GUI");Ok(())}
fn prototype<'a>(p:&Prototype,items:&BTreeMap<&str,&'a Item>)->Result<&'a Item>{integer(&p.amount,i32::MIN as i64,i32::MAX as i64)?;items.get(p.id.as_str()).copied().context("Inventory prototype fact missing")}
fn output(p:&Prototype,r:&Recipe,stat:&str,items:&BTreeMap<&str,&Item>)->Result<()>{
    prototype(p,items)?;let amount=integer(&p.amount,i32::MIN as i64,i32::MAX as i64)?;
    ensure!(r.outputs.len()==usize::from(amount>0),"Nonpositive native inventory is not a positive recipe output");
    if amount>0{let out=&r.outputs[0];ensure!(out.kind==Kind::Item&&out.role==OutputRole::Result&&out.slot==0&&out.id==p.id&&out.amount.is_none()&&out.change.is_none()
        &&out.chance.numerator=="1"&&out.chance.denominator=="1"&&matches!(&out.quantity,Some(Quantity::Potential{stat:s,nominal,sample:None,condition:Some(condition),parameters:None})if s==stat&&nominal==&p.amount&&!condition.is_empty()),"Native inventory output must remain a conditional prototype");}Ok(())
}
pub(super) fn validate_alchemy(p:&Alchemy,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    base(r)?;ensure!(r.source.owner=="AWWayofTime"&&r.source.handler=="WayofTime.alchemicalWizardry.client.nei.NEIAlchemyRecipeHandler"&&p.version=="1.7.52"
        &&p.orb==5&&p.work_progress==100&&p.accelerated_step==5&&r.grid.is_none()&&p.earlier.len()<=65536&&!p.orbs.is_empty()&&p.orbs.len()<=4096,"Blood alchemy source or work law differs");
    let templates:Vec<_>=p.templates.iter().flatten().collect();for s in &templates{selector(s)?;}for prior in &p.earlier{for s in prior.templates.iter().flatten(){selector(s)?;}}
    ensure!(r.inputs.len()==templates.len()+1&&p.containers.len()==templates.len(),"Alchemy material/container roles differ");
    for (slot,s) in templates.iter().enumerate(){let input=&r.inputs[slot];ensure!(input.slot==slot as u32&&input.kind==Kind::Item&&!input.choices.is_empty()&&p.containers[slot].len()==input.choices.len(),"Alchemy physical predicate examples differ");
        for (index,c) in input.choices.iter().enumerate(){let item=items.get(c.id.as_str()).context("Alchemy material fact missing")?;
            ensure!(item.registry==s.registry&&(s.metadata==32767||s.metadata==item.meta)&&c.amount=="1"&&matches!(c.consume,Consumption::Allocated)&&c.returns.is_empty()
                &&matches!(c.rule,Match::Wildcard{meta,nbt:true}if meta==(s.metadata==32767)),"Alchemy must use template wildcard and whole-slot container allocation");
            if let Some(container)=&p.containers[slot][index]{prototype(container,items)?;integer(&container.amount,1,i32::MAX as i64)?;}
        }
    }
    let orb=r.inputs.last().context("Alchemy orb role missing")?;ensure!(orb.slot==5&&orb.kind==Kind::Item&&!orb.choices.is_empty(),"Alchemy orb missing");
    let mut unique=BTreeSet::new();for o in &p.orbs{ensure!(!o.registry.is_empty()&&o.registry.len()<=512&&o.level>=p.minimum_orb&&unique.insert(&o.registry),"Alchemy orb tier set differs");}
    for c in &orb.choices{let item=items.get(c.id.as_str()).context("Alchemy orb fact missing")?;ensure!(unique.contains(&item.registry)&&c.amount=="1"&&c.returns.is_empty()&&matches!(c.consume,Consumption::Keep)&&matches!(c.rule,Match::Wildcard{meta:true,nbt:true}),"Alchemy tiered orb is retained");}
    output(&p.result,r,"bloodmagic:writingTable",items)
}
pub(super) fn validate_soldering(p:&Soldering,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    base(r)?;ensure!(r.source.owner=="LogisticsPipes"&&r.source.handler=="logisticspipes.nei.NEISolderingStationRecipeManager"&&p.version=="1.4.24-GTNH"
        &&p.solder_slot==9&&p.heat==100&&p.work_progress==100&&!p.definitions.is_empty()&&p.definitions.len()<=65536,"Soldering native source/work law differs");
    for d in &p.definitions{selector(&d.result)?;for s in d.inputs.iter().flatten(){selector(s)?;}}
    let selected=p.definitions.get(p.selected as usize).context("Selected soldering registration missing")?;
    let grid=r.grid.as_ref().context("Soldering absolute grid missing")?;ensure!(grid.width==3&&grid.height==3&&!grid.mirror&&grid.cells.len()==9,"Soldering uses an absolute 3x3 grid");
    let mut slot=0;for (cell,s) in selected.inputs.iter().enumerate(){if let Some(s)=s{ensure!(grid.cells[cell]==Some(slot as u32),"Soldering grid slot differs");let input=r.inputs.get(slot).context("Soldering input omitted")?;
        ensure!(input.slot==slot as u32&&input.kind==Kind::Item&&input.choices.len()==1,"Soldering literal input changed");let c=&input.choices[0];let item=items.get(c.id.as_str()).context("Soldering item fact missing")?;
        ensure!(item.registry==s.registry&&item.meta==s.metadata&&c.amount=="1"&&c.returns.is_empty()&&matches!(c.consume,Consumption::Allocated)&&matches!(c.rule,Match::Wildcard{meta:false,nbt:true}),"Soldering must keep literal metadata and ordered side-effect allocation");slot+=1;
    }else{ensure!(grid.cells[cell].is_none(),"Soldering empty cell filled");}}
    ensure!(slot==r.inputs.len(),"Soldering invented an iron-only solder predicate");let item=prototype(&p.result,items)?;
    ensure!(item.registry==selected.result.registry&&item.meta==selected.result.metadata,"Soldering result selector differs");
    if p.result_tag==ResultTag::FreshRandomUuid{ensure!(item.nbt.is_none(),"A random UUID must not be invented during capture");}
    output(&p.result,r,"logisticspipes:solderingInventory",items)
}
