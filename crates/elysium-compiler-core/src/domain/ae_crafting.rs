//! Pinned AE2/WCT native cell predicates. Item counts and NBT do not take part in matching.
use super::{Item,Kind,Match,OutputRole,Recipe};
use anyhow::{ensure,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AeCrafting {
    pub version:Version,
    pub native_recipe:String,
    pub shaped:bool,
    pub width:u8,
    pub height:u8,
    pub mirror:bool,
    pub predicates:Vec<Option<Vec<Selector>>>,
    pub matching:Matching,
    pub consumption:Consumption,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
pub enum Version {
    #[serde(rename="rv3-beta-695-GTNH")] Ae2,
    #[serde(rename="1.12.7")] Wireless,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
pub enum Matching {
    #[serde(rename="columnOffsetsUnmirroredThenOptionalMirror")] Shaped,
    #[serde(rename="rowMajorFirstRemainingPredicate")] Shapeless,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum Consumption {
    #[serde(rename="workbenchOnePerOccupiedCellThenNativeContainer")] Native,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Selector {pub registry:String,pub metadata:i32}
impl Selector {
    pub fn accepts(&self,item:&Item)->bool {
        self.registry==item.registry&&(self.metadata==32767||self.metadata==item.meta)
    }
}
pub(super) fn validate(p:&AeCrafting,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    let (owner,handler,native,version,shaped)=if p.native_recipe=="appeng.recipes.game.ShapedRecipe" {
        ("appliedenergistics2","appeng.integration.modules.NEIHelpers.NEIAEShapedRecipeHandler","appeng.recipes.game.ShapedRecipe",Version::Ae2,true)
    }else if p.native_recipe=="appeng.recipes.game.ShapelessRecipe" {
        ("appliedenergistics2","appeng.integration.modules.NEIHelpers.NEIAEShapelessRecipeHandler","appeng.recipes.game.ShapelessRecipe",Version::Ae2,false)
    }else{
        ("ae2wct","net.p455w0rd.wirelesscraftingterminal.integration.modules.NEIHelpers.NEIAEShapedRecipeHandler","net.p455w0rd.wirelesscraftingterminal.api.recipes.game.ShapedRecipe",Version::Wireless,true)
    };
    ensure!(r.source.owner==owner&&r.source.handler==handler&&p.native_recipe==native&&p.version==version&&p.shaped==shaped,"AE native source/version mismatch");
    ensure!(r.duration.is_none()&&r.energy.is_none()&&r.magic.is_none()&&r.properties.is_empty(),"AE native crafting cannot contain GUI or invented machine timing");
    ensure!(!p.predicates.is_empty()&&p.predicates.len()<=9,"AE native cell budget");
    let mut slots=Vec::new();let mut count=0u32;
    for group in &p.predicates {
        if let Some(group)=group {
            ensure!(!group.is_empty()&&group.len()<=65536&&group.iter().all(|s|!s.registry.is_empty()&&s.registry.len()<=512),"AE native predicate budget");
            slots.push(Some(count));count+=1;
        }else{ensure!(p.shaped,"AE shapeless cannot contain empty predicates");slots.push(None);}
    }
    if p.shaped {
        ensure!((1..=3).contains(&p.width)&&(1..=3).contains(&p.height)&&p.predicates.len()==usize::from(p.width)*usize::from(p.height)&&p.matching==Matching::Shaped,"AE native shaped dimensions/matching");
        let grid=r.grid.as_ref().ok_or_else(||anyhow::anyhow!("AE shaped cells are missing"))?;
        ensure!(grid.width==u32::from(p.width)&&grid.height==u32::from(p.height)&&grid.cells==slots&&grid.mirror==p.mirror,"AE logical cells differ from native predicates");
    }else{ensure!(p.width==0&&p.height==0&&!p.mirror&&r.grid.is_none()&&p.matching==Matching::Shapeless,"AE native greedy matching is missing");}
    ensure!(r.inputs.len()==count as usize&&r.outputs.len()==1,"AE native input/output arity");
    for (slot,group) in p.predicates.iter().filter_map(Option::as_ref).enumerate() {
        let input=&r.inputs[slot];ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty(),"AE native input role");
        for choice in &input.choices {
            let item=items.get(choice.id.as_str()).ok_or_else(||anyhow::anyhow!("AE input fact is missing"))?;
            ensure!(choice.amount=="1"&&matches!(choice.rule,Match::Wildcard{meta:_,nbt:true})&&group.iter().any(|s|s.accepts(item))
                &&matches!(choice.consume,super::Consumption::Consume|super::Consumption::Keep|super::Consumption::Damage{..}|super::Consumption::GtTool{..} | super::Consumption::NativeContainer { .. }),"AE displayed input does not satisfy native matcher");
            if let Match::Wildcard{meta,..}=choice.rule {
                ensure!(group.iter().any(|s|s.registry==item.registry&&if meta{s.metadata==32767}else{s.metadata==item.meta}),"AE wildcard policy differs from native selector");
            }
        }
        ensure!(group.iter().all(|s|input.choices.iter().any(|c|items.get(c.id.as_str()).is_some_and(|item|s.accepts(item)))),"AE native selector omitted all representative choices");
    }
    let output=&r.outputs[0];
    ensure!(output.kind==Kind::Item&&output.slot==0&&matches!(output.role,OutputRole::Result)&&output.amount.is_some()&&output.change.is_none()&&output.quantity.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1","AE native output prototype");
    Ok(())
}
