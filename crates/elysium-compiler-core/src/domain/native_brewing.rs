//! Brewing inventory transformation. One shared ingredient handles up to three potion slots.
use super::{Consumption,Item,Kind,Match,Quantity,Recipe};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="NativeBrewingProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Brewing {
    pub version:String,pub ingredient:u32,pub potion:u32,pub input_metadata:i32,pub output_metadata:i32,
    pub result_registry:String,pub expression:String,pub lingering:bool,pub preserve_nbt:bool,pub work_ticks:u32,pub maximum_potions:u32,
}
pub(super) fn validate(p:&Brewing,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    ensure!(p.version=="1.7.10"&&r.source.handler=="codechicken.nei.recipe.BrewingRecipeHandler"&&p.ingredient==0&&p.potion==1&&p.work_ticks==400&&p.maximum_potions==3
        &&p.expression.len()<=16384&&p.preserve_nbt!=p.lingering&&r.inputs.len()==2&&r.outputs.len()==1&&r.grid.is_none()&&r.magic.is_none()&&r.duration.is_none()&&r.energy.is_none()&&r.properties.is_empty(),"Native brewing inventory law differs");
    for (slot,input) in r.inputs.iter().enumerate(){ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&input.choices.len()==1,"Brewing input role differs");let c=&input.choices[0];let item=items.get(c.id.as_str()).context("Brewing input fact missing")?;
        ensure!(c.amount=="1"&&matches!(c.consume,Consumption::Consume)&&matches!(c.rule,Match::Wildcard{meta:false,nbt:true}),"Brewing uses unit stock and literal metadata, ignoring NBT matching");
        if slot==1 {ensure!(item.registry=="minecraft:potion"&&item.meta==p.input_metadata&&c.returns.is_empty(),"Brewing potion metadata differs");}
        if p.lingering&&slot==0{ensure!(item.registry=="etfuturum:dragon_breath"&&p.expression.is_empty()&&p.input_metadata&0x4000!=0&&p.output_metadata==p.input_metadata,"Native lingering branch must retain the splash metadata bit");}
    }
    let out=&r.outputs[0];let item=items.get(out.id.as_str()).context("Brewing result prototype missing")?;
    ensure!(out.kind==Kind::Item&&out.slot==0&&out.change.is_none()&&out.amount.is_none()&&out.chance.numerator=="1"&&out.chance.denominator=="1"
        &&matches!(&out.quantity,Some(Quantity::Potential{stat,nominal,..})if stat=="minecraft:brewingInventory"&&nominal=="1")
        &&item.registry==p.result_registry&&item.meta==p.output_metadata&&item.nbt.is_none()
        &&p.result_registry==if p.lingering{"etfuturum:lingering_potion"}else{"minecraft:potion"},"Brewing output must remain a conditional native per-slot prototype");
    Ok(())
}
