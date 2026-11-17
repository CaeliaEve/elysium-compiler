//! Internal reagent conversion, never a forged item-output recipe.
use super::{Consumption,Item,Kind,Match,Recipe};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BloodCalcinatorProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Calcinator {
    pub version:String,pub input:u32,pub orb:u32,pub reagent:String,pub name:String,pub amount:i32,pub color:[i32;4],
    pub work_ticks:u32,pub lp_per_tick:u32,pub buffer_capacity:u32,pub main_capacity:u32,pub transfer_per_tick:u32,
}
pub(super) fn validate(p:&Calcinator,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    ensure!(p.version=="1.7.52"&&r.source.handler=="WayofTime.alchemicalWizardry.client.nei.NEICalcinatorHandler"&&r.source.owner=="AWWayofTime"
        &&p.input==1&&p.orb==0&&!p.reagent.is_empty()&&p.reagent.len()<=512&&!p.name.is_empty()&&p.name.len()<=16384
        &&p.work_ticks==200&&p.lp_per_tick==10&&p.buffer_capacity==2000&&p.main_capacity==4000&&p.transfer_per_tick==20,"Calcinator native source/state law differs");
    ensure!(r.outputs.is_empty()&&r.inputs.len()==2&&r.grid.is_none()&&r.magic.is_none()&&r.duration.is_none()&&r.energy.is_none()&&r.properties.is_empty(),"Calcinator produces internal reagent, without fixed item outputs or elapsed duration");
    for input in &r.inputs{ensure!(input.kind==Kind::Item&&(input.slot==p.input||input.slot==p.orb)&&!input.choices.is_empty(),"Calcinator native input role missing");for c in &input.choices{
        let _=items.get(c.id.as_str()).context("Calcinator item fact missing")?;
        ensure!(c.amount=="1"&&c.returns.is_empty()&&matches!(c.rule,Match::Wildcard{meta,nbt:true}if meta==(input.slot==p.orb))
            &&if input.slot==p.orb{matches!(c.consume,Consumption::Keep)}else{matches!(c.consume,Consumption::Consume)},"Calcinator unit material / reusable orb differs");
    }}Ok(())
}
