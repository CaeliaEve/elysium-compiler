//! Current runic sample costs and source-bound startup/cycle component predicates.
use super::{Aspect,Consumption,Edit,Item,Kind,MagicKind,Match,OutputRole,Recipe};
use super::infusion::{LegacyPattern,StartupMatcher,legacy_phase,validate_pattern};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NativeRunic {
    pub native_recipe:String,
    /// Observed final charge of the supplied center, never a constant for its whole item class.
    pub charge:i32,
    pub pattern:LegacyPattern,
    pub pedestal_limit:i32,
}

pub(super) fn validate(p:&NativeRunic,r:&Recipe,items:&BTreeMap<&str,&Item>,aspects:&[Aspect])->Result<()> {
    ensure!(p.native_recipe=="thaumcraft.common.lib.crafting.InfusionRunicAugmentRecipe"
        &&super::magic::ari_source(&r.source,super::magic::ARI_INFUSION)
        &&r.grid.is_none()&&r.duration.is_none()&&r.energy.is_none()
        &&p.pedestal_limit>0&&p.pattern.startup==StartupMatcher::Legacy&&p.pattern.predicates.is_none()
        &&p.pattern.central.is_none()&&!p.pattern.allow_extra&&p.pattern.components.len()==2
        &&r.inputs.len()==3&&r.outputs.len()==1,"invalid native runic process");
    validate_pattern(&p.pattern,items)?;
    let diamond=&p.pattern.components[0].template;
    let resource=&p.pattern.components[1].template;
    ensure!(diamond.registry=="minecraft:diamond"&&diamond.metadata==0&&diamond.nbt.is_none()
        &&resource.metadata==14&&resource.nbt.is_none(),"native runic generated templates differ");
    let (pedestals,vis,instability)=super::process::runic_costs(p.charge);
    let magic=r.magic.as_ref().context("native runic magic sample missing")?;
    ensure!(magic.kind==MagicKind::Infusion&&magic.central==Some(0)&&magic.instability==Some(instability)
        &&magic.payment.is_none()&&!magic.creative&&magic.research.len()<=1,"invalid native runic magic sample");
    let mut actual=BTreeMap::new();
    for cost in &magic.aspects {
        let aspect=aspects.iter().find(|a|a.id==cost.aspect).context("native runic aspect missing")?;
        ensure!(aspect.source.owner=="Thaumcraft"&&aspect.source.handler=="thaumcraft.api.aspects.Aspect"
            &&actual.insert(aspect.source.key.as_str(),cost.amount.as_str()).is_none(),"native runic aspect source differs");
    }
    let expected=if vis>0{BTreeMap::from([("tutamen",(vis/2).to_string()),("praecantatio",(vis/2).to_string()),("potentia",vis.to_string())])}else{BTreeMap::new()};
    ensure!(actual.len()==expected.len()&&expected.iter().all(|(key,value)|actual.get(key)==Some(&value.as_str())),"native runic sample costs differ");
    for (slot,input) in r.inputs.iter().enumerate(){
        ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty()
            &&(slot!=0||input.choices.len()==1),"invalid native runic input groups");
        for choice in &input.choices {
            let item=items.get(choice.id.as_str()).context("native runic specimen missing")?;
            ensure!(p.pattern.states.iter().any(|s|s.id==choice.id),"native runic comparison observation missing");
            ensure!(choice.amount==if slot==2{pedestals.to_string()}else{"1".into()},"native runic pedestal sample count differs");
            if slot==0 {
                ensure!(matches!(choice.rule,Match::Wildcard{meta:true,nbt:true})&&matches!(choice.consume,Consumption::Stack)
                    &&choice.returns.is_empty(),"native runic central sample differs");
            }else{
                ensure!(matches!(choice.rule,Match::Exact)
                    &&(if slot==2{matches!(choice.consume,Consumption::Pedestals)}else{matches!(choice.consume,Consumption::Stack)})
                    &&legacy_phase(&p.pattern,&p.pattern.components[slot-1],item,1,false)?
                    &&legacy_phase(&p.pattern,&p.pattern.components[slot-1],item,1,true)?,"runic specimen fails startup/cycle comparison");
                ensure!(choice.returns.len()<=1,"native runic has multiple containers per pedestal");
                for returned in &choice.returns{ensure!(returned.kind==Kind::Item
                    &&crate::identity::integer(&returned.amount,1,i64::from(p.pedestal_limit))?>0,"native runic return exceeds pedestal limit");}
            }
        }
    }
    let output=&r.outputs[0];let change=output.change.as_ref().context("native runic edit missing")?;
    ensure!(output.kind==Kind::Item&&output.slot==0&&output.role==OutputRole::Result&&output.quantity.is_none()
        &&output.amount.as_deref()==Some("1")&&output.chance.numerator=="1"&&output.chance.denominator=="1"
        &&matches!(change.action,Edit::Runic)&&change.input==0&&change.bindings.is_none()&&change.samples.len()==1
        &&change.samples[0].amount=="1"&&change.samples[0].id==output.id,"invalid native runic result");
    Ok(())
}
