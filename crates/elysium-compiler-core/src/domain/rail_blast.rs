//! Independent Railcraft blast fuel supply; never a fixed per-recipe ingredient.
use super::{Consumption, Item, Kind, OutputRole, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

macro_rules! policy {
    ($name:ident,$value:literal) => {
        #[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
        pub enum $name { #[serde(rename=$value)] Native }
    };
}
policy!(RailFuelFilter,"nativeItemSubtypeExactNbtEitherWildcard");
policy!(RailFuelCache,"semiStrictOfferedNbtAbsentReusesLastItemMetaValue");
policy!(RailFuelBurn,"scaffoldZeroBlaze1000LavaAmountScrapZeroNativeFurnaceExceptionZero");
policy!(RailFuelTick,"masterServerBurnBudgetMinusFiveEvenIdle");
policy!(RailFuelReload,"positiveInputRecipeOutputRoomUnpausedBudgetAtMostTenPositiveBurnAddSignedInt");
policy!(RailFuelDepletion,"nonPositiveNullUnitNativeContainerOtherwiseDecrement");

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="RailBlastFuelTemplate")]
pub struct Template {
    pub item:String,
    pub metadata:i32,
    pub subtypes:bool,
    pub template_amount:String,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename="RailBlastFuelSupply")]
pub struct Supply {
    pub allowed:Vec<Template>, pub filter:RailFuelFilter, pub cache:RailFuelCache,
    pub burn:RailFuelBurn, pub tick:RailFuelTick, pub reload:RailFuelReload, pub depletion:RailFuelDepletion,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
#[schemars(rename="RailBlastProcess")]
pub struct Process { pub registered_time:i32, pub fuel:Supply }

pub(super) fn validate(p:&Process,recipe:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    let duration=p.registered_time.max(1).to_string();
    ensure!(recipe.source.handler=="tonius.neiintegration.mods.railcraft.RecipeHandlerBlastFurnace"
        && recipe.duration.as_deref()==Some(duration.as_str()) && recipe.grid.is_none()
        && recipe.energy.is_none() && recipe.magic.is_none(),"Invalid native Railcraft blast process");
    ensure!(recipe.inputs.len()==1 && recipe.inputs[0].kind==Kind::Item
        && recipe.inputs[0].choices.iter().all(|choice| choice.amount=="1"
            && matches!(choice.consume,Consumption::Consume) && choice.returns.is_empty()),
        "Railcraft blast material consumption differs from its independent fuel supply");
    ensure!(recipe.outputs.len()==1 && recipe.outputs[0].kind==Kind::Item
        && matches!(recipe.outputs[0].role,OutputRole::Result)
        && recipe.outputs[0].amount.is_some() && recipe.outputs[0].quantity.is_none()
        && recipe.outputs[0].change.is_none() && recipe.outputs[0].chance.numerator=="1"
        && recipe.outputs[0].chance.denominator=="1","Invalid native Railcraft blast result");
    ensure!(p.fuel.allowed.len()<=262144,"Native Railcraft fuel table exceeds budget");
    for template in &p.fuel.allowed {
        let item=items.get(template.item.as_str()).context("Native Railcraft fuel item fact absent")?;
        ensure!(template.metadata>=-1 && item.meta>=0 && item.meta!=32767
            && (matches!(template.metadata,-1|32767) || item.meta==template.metadata),
            "Native Railcraft fuel predicate and concrete display differ");
        crate::identity::integer(&template.template_amount,1,i32::MAX.into())?;
    }
    Ok(())
}

pub(crate) fn uses(p:&Process,items:&BTreeMap<&str,&Item>)->Result<Vec<String>> {
    let mut used=BTreeSet::new();
    for template in &p.fuel.allowed {
        let base=items.get(template.item.as_str()).context("Native Railcraft fuel item absent")?;
        for offered in items.values().filter(|item|item.registry==base.registry) {
            let metadata=!template.subtypes || matches!(template.metadata,-1|32767)
                || matches!(offered.meta,-1|32767) || template.metadata==offered.meta;
            let tags=match (&base.nbt,&offered.nbt) {
                (None,None)=>true,(Some(a),Some(b))=>a.native_equal(b)?,_=>false,
            };
            if metadata && tags {used.insert(offered.id.clone());}
        }
    }
    Ok(used.into_iter().collect())
}
