//! Extra Utilities 1.2.12 personal-grid acquisition, before ItemSoul.onCreated.
use super::{Consumption,Item,Kind,Match,OutputRole,Quantity,Recipe};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="ExtraUtilsSoulProcess")]
pub struct SoulProcess {
    pub native_recipe:String,pub native_lifecycle:String,pub inventory_size:u8,
    pub sword:String,pub pending:String,pub display:String,
    pub health_decrement:String,pub minimum_health:String,pub reference_health:String,pub modifier_uuid:String,
}

pub(super) fn validate(p:&SoulProcess,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(r.source.handler=="com.rwtema.extrautils.nei.SoulHandler"
        &&p.native_recipe=="com.rwtema.extrautils.crafting.RecipeSoul"
        &&p.native_lifecycle=="com.rwtema.extrautils.item.ItemSoul.onCreated"
        &&p.inventory_size==4&&p.health_decrement=="0.1"&&p.minimum_health=="6"&&p.reference_health=="20"
        &&p.modifier_uuid=="12345678-9182-3532-aaaa-aaabacadabaa",
        "Soul acquisition lost its native grid/lifecycle law");
    ensure!(r.grid.is_none()&&r.energy.is_none()&&r.duration.is_none()&&r.magic.is_none()
        &&r.inputs.len()==1&&r.outputs.len()==1,"Soul requires one occupied personal-inventory cell");
    let sword=items.get(p.sword.as_str()).context("Soul sword fact missing")?;
    let pending=items.get(p.pending.as_str()).context("Pending Soul fact missing")?;
    let display=items.get(p.display.as_str()).context("NEI Soul display fact missing")?;
    ensure!(sword.meta==0&&sword.nbt.is_none()&&pending.meta==2&&pending.nbt.is_none()
        &&display.meta==0&&display.nbt.is_none()&&pending.registry==display.registry
        &&pending.registry!=sword.registry,"Soul pending result and NEI display were conflated");
    let input=&r.inputs[0];ensure!(input.slot==0&&input.kind==Kind::Item&&input.choices.len()==1,"Soul sword input changed");
    let c=&input.choices[0];ensure!(c.id==p.sword&&c.amount=="1"&&matches!(c.rule,Match::Wildcard{meta:true,nbt:true})
        &&matches!(c.consume,Consumption::Consume)&&c.returns.is_empty(),"Soul consumes a sword irrespective of damage/NBT");
    let output=&r.outputs[0];ensure!(output.kind==Kind::Item&&output.slot==0&&output.id==p.pending
        &&matches!(output.role,OutputRole::Result)&&output.amount.is_none()&&output.change.is_none()
        &&output.chance.numerator=="1"&&output.chance.denominator=="1"
        &&matches!(&output.quantity,Some(Quantity::Potential{stat,nominal,condition,sample,parameters})
            if stat=="extrautils:soulLifecycle"&&nominal=="1"&&condition.as_deref()==Some("playerWorldCreationLifecycle")
            &&sample.as_deref()==Some("1")&&parameters.is_none()),"Soul was promoted to a guaranteed owned final result");
    Ok(())
}
