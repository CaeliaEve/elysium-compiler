//! Pinned Forestry container filling, including partial in-place callbacks.
use super::{fabricator::{NativeFluid, NativeStack}, Consumption, Fluid, Item, Kind, Match, Quantity, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="lowercase")]
pub enum CallbackKind { Forge, Ic2 }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct BottleCallback { pub kind: CallbackKind, pub capacity: u32, pub nbt_key: String, pub owner: String }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct BottlerProcess { pub callback: Option<BottleCallback>, pub fluid: String, pub retained_input: NativeStack, pub requirement: NativeFluid, pub result: NativeStack }

pub(super) fn validate_recipe(process:&BottlerProcess, recipe:&Recipe, items:&BTreeMap<&str,&Item>, fluids:&BTreeMap<&str,&Fluid>) -> Result<()> {
    ensure!(recipe.source.handler=="forestry.factory.recipes.nei.NEIHandlerBottler" && recipe.duration.is_none()
        && recipe.energy.is_none() && recipe.grid.is_none() && recipe.magic.is_none(), "Bottler cannot invent fixed duration or crafting");
    process.retained_input.validate()?;process.requirement.validate()?;process.result.validate()?;
    ensure!(process.fluid==process.requirement.registry && recipe.inputs.len()==2 && recipe.outputs.len()==1,"Bottler omitted native container/fluid/result");
    let input=recipe.inputs.iter().find(|i|i.kind==Kind::Item&&i.slot==0).context("Bottler container missing")?;
    ensure!(input.choices.len()==1,"Bottler container predicate widened");let c=&input.choices[0];
    let item=items.get(c.id.as_str()).context("Bottler input fact missing")?;
    ensure!(item.registry==process.retained_input.registry && item.meta==process.retained_input.meta && c.amount=="1"
        && c.returns.is_empty() && matches!(c.consume,Consumption::Consume) && matches!(c.rule,Match::Wildcard{meta:false,nbt:true}),"Bottler requires one literal-meta container, ignoring item NBT");
    let fluid=recipe.inputs.iter().find(|i|i.kind==Kind::Fluid&&i.slot==0).context("Bottler fluid missing")?;
    ensure!(fluid.choices.len()==1,"Bottler fluid predicate widened");let f=&fluid.choices[0];
    let fact=fluids.get(f.id.as_str()).context("Bottler fluid fact missing")?;
    ensure!(fact.registry==process.fluid && f.amount==process.requirement.amount.to_string() && f.returns.is_empty()
        && matches!(f.consume,Consumption::Consume),"Bottler native fluid requirement differs");
    let output=&recipe.outputs[0];super::fabricator::guaranteed(output)?;
    let result=items.get(output.id.as_str()).context("Bottler output fact missing")?;
    ensure!(output.kind==Kind::Item && output.slot==0 && result.registry==process.result.registry
        && result.meta==process.result.meta && result.nbt==process.result.nbt,"Bottler native output identity differs");
    if let Some(callback)=&process.callback {
        ensure!(callback.capacity>0 && callback.capacity<=i32::MAX as u32 && callback.nbt_key=="Fluid"
            && match callback.kind { CallbackKind::Forge=>callback.owner=="net.minecraftforge.fluids.ItemFluidContainer",
                CallbackKind::Ic2=>callback.owner=="ic2.core.item.ItemIC2FluidContainer" },"Unadapted fluid container callback");
        ensure!(matches!(f.rule,Match::Wildcard{meta:false,nbt:true}) && output.amount.is_none()
            && matches!(&output.quantity,Some(Quantity::Potential{stat,nominal,condition:Some(_),sample:None,parameters:None})
                if stat=="forestry:bottlerCompletion"&&nominal=="1"),"Partial filling cannot become guaranteed full output");
    } else {
        ensure!(fact.nbt==process.requirement.nbt && matches!(f.rule,Match::Exact)
            && output.amount.as_deref()==Some("1") && output.quantity.is_none(),"Fixed Bottler must preserve exact fluid amount/NBT and one result");
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn native_bottler_process_is_typed() {
        let stack = serde_json::json!({"registry":"minecraft:bucket","meta":0,"amount":1,"nbt":null,"ores":[],"subtypes":false,"maxDamage":0,"containerOwner":"net.minecraft.item.Item","container":{"kind":"none"}});
        let fluid = serde_json::json!({"registry":"water","amount":1000,"nbt":null,"viscosity":1000});
        let value = serde_json::json!({"kind":"forestryBottler","callback":null,"fluid":"water","retainedInput":stack,"requirement":fluid,"result":stack});
        assert!(serde_json::from_value::<crate::domain::Process>(value).is_ok(), "Native Bottler process contract is missing");
    }
}
