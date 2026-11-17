//! EnhancedLootBags 1.2.8: correlated weighted draws and deferred bag debit.
use super::{Consumption,Item,Kind,Match,OutputRole,Quantity,Recipe};
use crate::identity::Nbt;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Opening {
    pub native_item:String,pub bag:String,pub group:i32,pub allow_fortune:bool,
    pub groups:Vec<Group>,pub buffered:Vec<CachedGroup>,pub retry_limit:u8,
    pub quantity_law:String,pub budget_law:String,pub retry_state:String,pub limit_law:String,
    pub debit_law:String,pub empty_draw:String,pub shuffle_law:String,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Group {
    pub id:i32,pub name:Option<String>,pub min_items:i32,pub max_items:i32,pub trash:i32,
    pub combine:bool,pub cached_weight:i32,pub rarity:i32,pub drops:Vec<Drop>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CachedGroup {pub key:String,pub group:Group}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Drop {
    pub identifier:Option<String>,pub descriptor:Option<String>,pub nbt:Option<String>,pub item_group:String,
    pub amount:i32,pub weight:i32,pub limit:i32,pub random:bool,pub resolved:Option<String>,
    pub resolved_nbt:Option<Nbt>,pub stack_limit:Option<i32>,
}

pub(super) fn validate(p:&Opening,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(r.source.handler=="eu.usrv.enhancedlootbags.integration.nei.LootBagRecipeHandler"
        &&p.native_item=="eu.usrv.enhancedlootbags.core.items.ItemLootBag"
        &&r.grid.is_none()&&r.magic.is_none()&&r.duration.is_none()&&r.energy.is_none(),"Invalid native loot opening");
    ensure!(p.retry_limit==10&&p.quantity_law=="maxExclusivePlusOneWhenGreaterThanMin"
        &&p.budget_law=="splitStackCount"&&p.retry_state=="retainedSelectedAndPending"
        &&p.limit_law=="playerGroupIdentifierBeforeResolution"&&p.debit_law=="afterCompleteOpening"
        &&p.empty_draw=="abortKeepBagAndPriorSpawnedDrops"&&p.shuffle_law=="nativeRandomPermutationPerOpening", "Native loot opening rules changed");
    ensure!(!p.groups.is_empty()&&p.groups.len()<=65536&&p.buffered.len()<=65536,"Loot snapshot exceeds budget");
    let selected=p.groups.iter().find(|g|g.id==p.group).context("Missing native loot group")?;
    // Native getGroupByID is first-match. Do not silently collapse duplicate IDs.
    let mut keys=BTreeSet::new();
    for cache in &p.buffered {
        ensure!(keys.insert(&cache.key),"Duplicate native loot buffer key");
        let valid=p.groups.iter().any(|g|cache.key.strip_prefix(&format!("{}-",g.id))
            .is_some_and(|level|level.parse::<i32>().is_ok_and(|n|n.to_string()==level)));
        ensure!(valid,"Invalid native fortune buffer key");
    }
    for g in p.groups.iter().chain(p.buffered.iter().map(|c|&c.group)) {
        ensure!(g.drops.len()<=65536,"Loot drops exceed budget");
        for d in &g.drops {
            ensure!(d.item_group.len()<=4096&&d.descriptor.as_ref().is_none_or(|s|s.len()<=4096)
                &&d.nbt.as_ref().is_none_or(|s|s.len()<=524288),"Loot descriptor exceeds budget");
            if let Some(id)=&d.resolved {
                let item=items.get(id.as_str()).context("Missing resolved loot item")?;
                ensure!(d.stack_limit.is_some_and(|n|n>0)&&d.resolved_nbt==item.nbt,"Native resolved loot state differs");
                let descriptor=d.descriptor.as_ref().context("Resolved loot lacks descriptor")?;
                let parts:Vec<_>=descriptor.split(':').collect();
                ensure!(parts.len()>=2&&item.registry==format!("{}:{}",parts[0],parts[1]),"Loot resolved registry differs from descriptor");
                let meta=if parts.len()==3 {parts[2].parse::<i32>()?}else{0};
                ensure!(item.meta==meta,"Loot resolved metadata differs from native descriptor");
            }else{ensure!(d.stack_limit.is_none()&&d.resolved_nbt.is_none(),"Unresolved loot fabricated an item");}
        }
    }
    ensure!(r.inputs.len()==1&&r.inputs[0].kind==Kind::Item&&r.inputs[0].slot==0&&r.inputs[0].choices.len()==1,"Loot requires one bag input");
    let c=&r.inputs[0].choices[0];let bag=items.get(p.bag.as_str()).context("Missing bag fact")?;
    ensure!(c.id==p.bag&&c.amount=="1"&&bag.meta==p.group&&bag.nbt.is_none()
        &&matches!(c.rule,Match::Wildcard{meta:false,nbt:true})&&matches!(c.consume,Consumption::Allocated)
        &&c.returns.is_empty(),"Loot bag must preserve fortune NBT and defer debit until completion");
    let mut possible:BTreeSet<&str>=selected.drops.iter().filter_map(|d|d.resolved.as_deref()).collect();
    if selected.combine {
        if let Some(trash)=p.groups.iter().find(|g|g.id==selected.trash){possible.extend(trash.drops.iter().filter_map(|d|d.resolved.as_deref()));}
        for c in &p.buffered {if c.key.starts_with(&format!("{}-",selected.id)) {possible.extend(c.group.drops.iter().filter_map(|d|d.resolved.as_deref()));}}
    }
    let mut outputs=BTreeSet::new();
    for o in &r.outputs {
        ensure!(o.kind==Kind::Item&&matches!(o.role,OutputRole::Result)&&o.amount.is_none()&&o.change.is_none()
            &&matches!(o.quantity,Some(Quantity::LootBag{}))&&o.chance.numerator=="1"&&o.chance.denominator=="1"
            &&outputs.insert(o.id.as_str()),"Loot possibilities cannot become independent guaranteed outputs");
    }
    ensure!(outputs==possible,"Loot output inventory differs from native opening branches");
    Ok(())
}

/// Fortune only modifies copied trash weights. Exactly III may bypass the merge.
pub fn fortune_weight(old:i32,level:i32)->i32 {
    if level<=0 {old} else if level>=3 {0} else {
        old.wrapping_sub((f64::from(old)*(0.33*f64::from(level))).floor() as i32)
    }
}

#[cfg(test)]
mod tests {
    #[test] fn fortune_retains_native_floor_and_java_integer_semantics(){
        assert_eq!(super::fortune_weight(3,1),3);assert_eq!(super::fortune_weight(10,2),4);
        assert_eq!(super::fortune_weight(10,3),0);assert_eq!(super::fortune_weight(10,4),0);
        assert_eq!(super::fortune_weight(-3,1),-2);assert_eq!(super::fortune_weight(3,0),3);
    }
}
