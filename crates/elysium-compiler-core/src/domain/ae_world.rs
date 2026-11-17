//! AE2's native acquisition information and facade grid have different contracts.
use super::{Consumption, Item, Kind, Match, OutputRole, Quantity, Recipe};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "AeFacadeProcess")]
pub struct FacadeProcess {
    pub native_recipe: String,
    pub texture: String,
    pub output_count: u32,
    /// Native item-to-block metadata normalization may change the displayed texture metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 0))]
    pub texture_metadata: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "AeWorldMaterial")]
pub enum Material {
    CertusQuartzCrystalCharged, LogicProcessorPress, CalcProcessorPress, EngProcessorPress,
    FluixCrystal, #[serde(rename="qESingularity")] QeSingularity,
    PurifiedCertusQuartzCrystal, PurifiedNetherQuartzCrystal, PurifiedFluixCrystal,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "AeWorldAction")]
pub enum Action {
    ChargedOre,
    MeteoriteLoot,
    Fluix { #[serde(rename="liquidTicks")] liquid_ticks: u32 },
    Singularity { explosion: bool, #[serde(rename="freshFrequency")] fresh_frequency: bool },
    Growth { #[serde(rename="progressStart")] progress_start: i32,
             #[serde(rename="progressEnd")] progress_end: i32,
             #[serde(rename="liquidMaterial")] liquid_material: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[derive(PartialEq)]
pub struct NativeIdentity {
    pub registry: String,
    pub meta: i32,
    pub nbt: Option<Nbt>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "AeWorldProcess")]
pub struct WorldProcess {
    pub material: Material,
    pub action: Action,
    pub result: String,
    pub sources: Vec<Vec<String>>,
    pub result_identity: NativeIdentity,
    pub source_identities: Vec<Vec<NativeIdentity>>,
    /// Sealed role-to-definition map copied from the native callback. This
    /// keeps material roles bound to concrete identities across compilation.
    pub native_definitions: BTreeMap<String, NativeIdentity>,
}

pub(super) fn validate_world(p: &WorldProcess, recipe: &Recipe, items: &BTreeMap<&str,&Item>) -> Result<()> {
    ensure!(recipe.source.handler == "appeng.integration.modules.NEIHelpers.NEIWorldCraftingHandler"
        && recipe.grid.is_none() && recipe.duration.is_none() && recipe.energy.is_none()
        && recipe.magic.is_none(), "invalid native world acquisition");
    let (material, text_source) = match p.material {
        Material::CertusQuartzCrystalCharged => ("certusQuartzCrystalCharged", "ChargedQuartz,ChargedQuartzFind"),
        Material::LogicProcessorPress => ("logicProcessorPress", "inWorldCraftingPresses"),
        Material::CalcProcessorPress => ("calcProcessorPress", "inWorldCraftingPresses"),
        Material::EngProcessorPress => ("engProcessorPress", "inWorldCraftingPresses"),
        Material::FluixCrystal => ("fluixCrystal", "inWorldFluix"),
        Material::QeSingularity => ("qESingularity", "inWorldSingularity"),
        Material::PurifiedCertusQuartzCrystal => ("purifiedCertusQuartzCrystal", "inWorldPurificationCertus"),
        Material::PurifiedNetherQuartzCrystal => ("purifiedNetherQuartzCrystal", "inWorldPurificationNether"),
        Material::PurifiedFluixCrystal => ("purifiedFluixCrystal", "inWorldPurificationFluix"),
    };
    let key=format!("ae2:world/{material}");
    let source=format!("appeng.core.localization.GuiText#{text_source}");
    ensure!(recipe.notes.iter().any(|note|note.key==key&&note.source==source),
        "Native AE2 world explanation is missing its stable field/source binding");
    let (count, slots) = match (&p.material, &p.action) {
        (Material::CertusQuartzCrystalCharged, Action::ChargedOre) => (1,0),
        (Material::LogicProcessorPress|Material::CalcProcessorPress|Material::EngProcessorPress, Action::MeteoriteLoot) => (1,0),
        (Material::FluixCrystal, Action::Fluix{liquid_ticks:61}) => (2,3),
        (Material::QeSingularity, Action::Singularity{explosion:true,fresh_frequency:true}) => (2,2),
        (Material::PurifiedCertusQuartzCrystal, Action::Growth{progress_start:0,progress_end:600,liquid_material})
        | (Material::PurifiedNetherQuartzCrystal, Action::Growth{progress_start:600,progress_end:1200,liquid_material})
        | (Material::PurifiedFluixCrystal, Action::Growth{progress_start:1200,progress_end:1800,liquid_material})
            if liquid_material == "liquid" => (1,1),
        _ => anyhow::bail!("world material differs from its native action"),
    };
    ensure!(recipe.inputs.len()==slots && p.sources.len()==slots && p.source_identities.len()==slots, "native acquisition lost its item entities");
    ensure!(p.native_definitions.get("result")==Some(&p.result_identity), "native result definition is not sealed");
    for (slot,input) in recipe.inputs.iter().enumerate() {
        ensure!(input.slot==slot as u32 && input.kind==Kind::Item && !input.choices.is_empty()
            && input.choices.len()==p.sources[slot].len()
            && input.choices.len()==p.source_identities[slot].len(), "invalid world action input");
        for (index, ((choice,native_id),identity)) in input.choices.iter().zip(&p.sources[slot]).zip(&p.source_identities[slot]).enumerate() {
            ensure!(&choice.id==native_id && items.contains_key(native_id.as_str()), "world entity differs from its native definition");
            let fact=items.get(native_id.as_str()).unwrap();
            ensure!(fact.registry==identity.registry && fact.meta==identity.meta && fact.nbt==identity.nbt, "world input identity differs from native definition");
            let role=format!("source{}_{}",slot,index);
            ensure!(p.native_definitions.get(&role)==Some(identity), "world role identity differs from native definition");
            ensure!(choice.amount=="1" && matches!(choice.consume,Consumption::Consume)
                && choice.returns.is_empty(), "world action consumption changed");
            let valid = match &p.action {
                Action::Singularity{..} if slot==1 => matches!(&choice.rule,Match::AeEnderDust{ore,..} if ore=="dustEnder"||ore=="dustEnderPearl"),
                Action::Growth{progress_start,progress_end,..} => matches!(&choice.rule,Match::AeCrystalSeed{start,end} if start==progress_start&&end==progress_end),
                _ => matches!(choice.rule,Match::Wildcard{meta:false,nbt:true}),
            };
            ensure!(valid, "world entity predicate differs from native source");
        }
    }
    ensure!(recipe.outputs.len()==1, "world acquisition invented a second output");
    let output=&recipe.outputs[0];
    ensure!(output.id==p.result && items.contains_key(p.result.as_str())
        && output.slot==0 && output.kind==Kind::Item && matches!(output.role,OutputRole::Result)
        && output.amount.is_none() && output.change.is_none()
        && output.chance.numerator=="1" && output.chance.denominator=="1"
        && matches!(&output.quantity,Some(Quantity::Potential{stat,nominal,sample,condition,parameters})
            if stat=="ae2:worldAcquisition" && nominal==&count.to_string()
                && sample.as_deref()==Some(nominal.as_str()) && parameters.is_none()
                && condition.as_deref()==Some("worldConditionsAndSourceAction")),
        "native acquisition cannot promise free guaranteed production");
    let result_fact=items.get(p.result.as_str()).unwrap();
    ensure!(result_fact.registry==p.result_identity.registry && result_fact.meta==p.result_identity.meta && result_fact.nbt==p.result_identity.nbt, "world output identity differs from native definition");
    Ok(())
}

pub(super) fn validate_facade(p: &FacadeProcess, recipe: &Recipe, items: &BTreeMap<&str,&Item>) -> Result<()> {
    ensure!(recipe.source.handler=="appeng.integration.modules.NEIHelpers.NEIFacadeRecipeHandler"
        && p.native_recipe=="appeng.recipes.game.FacadeRecipe" && p.output_count==4
        && recipe.duration.is_none() && recipe.energy.is_none() && recipe.magic.is_none(), "invalid native facade source");
    let grid=recipe.grid.as_ref().context("facade requires its native grid")?;
    ensure!(grid.width==3 && grid.height==3 && !grid.mirror
        && grid.cells==vec![None,Some(0),None,Some(1),Some(2),Some(3),None,Some(4),None]
        && recipe.inputs.len()==5, "facade requires four anchors and empty corners");
    let mut anchor:Option<&str>=None;
    for (slot,input) in recipe.inputs.iter().enumerate() {
        ensure!(input.slot==slot as u32 && input.kind==Kind::Item && input.choices.len()==1, "facade cell changed");
        let choice=&input.choices[0];
        ensure!(choice.amount=="1" && matches!(choice.consume,Consumption::Consume) && choice.returns.is_empty(), "facade cell consumption changed");
        if slot==2 {
            ensure!(choice.id==p.texture && matches!(choice.rule,Match::FacadeTexture), "facade texture admission changed");
        } else {
            ensure!(matches!(choice.rule,Match::Wildcard{meta:false,nbt:true}) && anchor.is_none_or(|id|id==choice.id), "facade anchor definition changed");
            anchor=Some(&choice.id);
        }
    }
    ensure!(recipe.outputs.len()==1, "facade invented additional results");
    let result=&recipe.outputs[0];
    ensure!(result.kind==Kind::Item && result.slot==0 && matches!(result.role,OutputRole::Result)
        && result.amount.as_deref()==Some("4") && result.quantity.is_none() && result.change.is_none()
        && result.chance.numerator=="1" && result.chance.denominator=="1", "native facade output count changed");
    let texture=items.get(p.texture.as_str()).context("facade texture fact missing")?;
    let item=items.get(result.id.as_str()).context("facade output fact missing")?;
    let effective=p.texture_metadata.unwrap_or(texture.meta);
    ensure!(effective>=0 && effective!=32767, "facade normalized texture metadata must be concrete");
    let Some(Nbt::Compound{value:tags})=&item.nbt else {anyhow::bail!("facade has no native texture identity")};
    let (modid,name)=texture.registry.split_once(':').context("facade registry has no owner")?;
    ensure!(matches!(tags.get("modid"),Some(Nbt::String{value}) if value==modid)
        && matches!(tags.get("itemname"),Some(Nbt::String{value}) if value==name)
        && matches!(tags.get("x"),Some(Nbt::IntArray{value}) if value.len()==2&&value[1]==effective.to_string()), "facade texture NBT differs from selected native block");
    Ok(())
}
