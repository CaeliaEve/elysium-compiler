//! AE2FC's upgrade matches physical cells, independently of its shapeless display superclass.
use super::{Consumption, Edit, Item, Kind, Match, OutputRole, Recipe};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Upgrade {
    pub version: Version,
    pub wireless_version: WirelessVersion,
    pub native_recipe: String,
    /// Registered instances of the native terminal/booster classes.
    pub terminals: Vec<String>,
    pub boosters: Vec<String>,
    pub flag: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Version { #[serde(rename="1.4.120-gtnh")] Native }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum WirelessVersion { #[serde(rename="1.12.7")] Native }

pub(super) fn validate(p:&Upgrade,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(r.source.owner=="NEI"&&r.source.handler=="codechicken.nei.recipe.ShapelessRecipeHandler"
        &&p.native_recipe=="com.glodblock.github.loader.recipe.WirelessTerminalRecipe"
        &&p.flag=="infinityBoosterCard","wireless upgrade source/flag differs");
    ensure!(r.duration.is_none()&&r.energy.is_none()&&r.magic.is_none()
        &&r.properties.is_empty()&&r.inputs.len()==2&&r.outputs.len()==1,"wireless upgrade arity or manual costs differ");
    for names in [&p.terminals,&p.boosters] {
        ensure!(!names.is_empty()&&names.len()<=65536&&names.windows(2).all(|v|v[0]<v[1]),"wireless native class census is invalid");
        for name in names {crate::identity::registry_name(name)?;}
    }
    ensure!(!p.terminals.iter().any(|name|p.boosters.binary_search(name).is_ok()),"wireless native class censuses overlap");
    let grid=r.grid.as_ref().context("wireless physical cells missing")?;
    ensure!(grid.width==3&&grid.height==3&&!grid.mirror
        &&grid.cells==[Some(0),Some(1),None,None,None,None,None,None,None],"wireless upgrade changed its physical cells");
    for (slot,input) in r.inputs.iter().enumerate() {
        ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty(),"wireless witness input missing");
        let members=if slot==0{&p.terminals}else{&p.boosters};
        for c in &input.choices {
            let item=items.get(c.id.as_str()).context("wireless witness fact missing")?;
            ensure!(members.binary_search(&item.registry).is_ok()&&c.amount=="1"
                &&matches!(c.rule,Match::Wildcard{meta:true,nbt:true})
                &&matches!(c.consume,Consumption::Consume|Consumption::Keep|Consumption::Damage{..}|Consumption::GtTool{..}|Consumption::NativeContainer{..}),
                "wireless witness differs from native class predicate");
            if slot==0 {
                let tags=match &item.nbt {None=>None,Some(Nbt::Compound{value})=>Some(value),_=>anyhow::bail!("wireless root tag is not a compound")};
                ensure!((super::change::number(tags.and_then(|t|t.get(&p.flag)))? as i8)==0,"wireless witness already has a booster");
            }
        }
    }
    let out=&r.outputs[0];
    let change=out.change.as_ref().context("wireless result must copy its actual offered terminal")?;
    ensure!(out.kind==Kind::Item&&out.slot==0&&out.role==OutputRole::Result&&out.amount.as_deref()==Some("1")
        &&out.quantity.is_none()&&out.chance.numerator=="1"&&out.chance.denominator=="1"
        &&change.input==0&&change.bindings.is_none()&&change.samples.len()==r.inputs[0].choices.len()
        &&change.samples.iter().all(|s|s.amount=="1")&&change.samples.first().is_some_and(|s|s.id==out.id),
        "wireless result specimen or copy binding differs");
    ensure!(matches!(&change.action,Edit::Patch{set,limits} if limits.is_empty()&&set.len()==1
        &&matches!(set.get(&p.flag),Some(Nbt::Byte{value}) if value=="1")),"wireless result flag patch differs");
    ensure!(r.notes.iter().any(|n|n.key=="ae2fc:wirelessUpgrade"
        &&n.source=="com.glodblock.github.loader.recipe.WirelessTerminalRecipe#matches/getCraftingResult"),
        "wireless ignored-cell/whole-stack rule note missing");
    Ok(())
}
