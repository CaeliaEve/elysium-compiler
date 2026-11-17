//! Gadomancy's cache is an invocation protocol, not a source of recipe definitions.
use super::{Consumption,Edit,Item,Kind,MagicKind,Match,OutputRole,Recipe};
use super::infusion::{OreMode,Template};
use crate::identity::Nbt;
use anyhow::{bail,ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Comparison {
    pub item:Template,pub amount:i32,pub limit:i32,pub ores:Vec<String>,
    pub default_ores:Vec<String>,pub tags:BTreeMap<String,String>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct State {
    pub id:String,pub comparison:Comparison,pub once_tags:BTreeMap<String,String>,pub twice_tags:BTreeMap<String,String>,
    pub armor_position:i32,pub potion:bool,pub invisible:Option<bool>,pub serialized:Nbt,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct DisguiseInfusion {
    pub native_recipe:String,pub mode:OreMode,pub components:Vec<Comparison>,pub states:Vec<State>,pub pedestal_limit:i32,
}
fn root(tag:Option<&Nbt>)->Result<Option<&BTreeMap<String,Nbt>>>{match tag{None=>Ok(None),Some(Nbt::Compound{value})=>Ok(Some(value)),_=>bail!("disguise item tag must be a compound")}}
fn disguised(item:&Item)->Result<bool>{Ok(root(item.nbt.as_ref())?.and_then(|r|r.get("gadomancy")).and_then(|t|root(Some(t)).ok().flatten()).is_some_and(|r|r.contains_key("disguise")))}
fn equal(mode:OreMode,a:&Comparison,a_tags:&BTreeMap<String,String>,b:&Comparison,b_tags:&BTreeMap<String,String>)->bool{
    if b.item.nbt.is_some()&&(a.item.nbt.is_none()||b_tags.iter().any(|(k,v)|a_tags.get(k)!=Some(v))){return false;}
    let fuzzy=match mode {
        OreMode::Default=>a.ores.first().is_some_and(|name|b.default_ores.contains(name)),
        OreMode::Strict=>a.ores.iter().collect::<BTreeSet<_>>()==b.ores.iter().collect::<BTreeSet<_>>(),
        OreMode::Relaxed=>a.ores.iter().any(|name|b.ores.contains(name)),
        OreMode::None=>a.item.registry==b.item.registry&&a.item.metadata==b.item.metadata&&match (&a.item.nbt,&b.item.nbt){
            (None,None)=>true,(Some(a),Some(b))=>super::matching::native_tag(a,b,true),_=>false},
    };
    fuzzy||a.item.registry==b.item.registry&&(a.item.metadata==b.item.metadata||b.item.metadata==32767)&&a.amount<=a.limit
}
fn state<'a>(p:&'a DisguiseInfusion,id:&str)->Result<&'a State>{p.states.binary_search_by(|s|s.id.as_str().cmp(id)).ok().map(|i|&p.states[i]).context("disguise comparison observation missing")}
/// Returns the residual disguise's physical input index. Both startup and cycle preserve their first match.
pub fn select(p:&DisguiseInfusion,center:&Item,outer:&[&Item])->Result<Option<usize>>{
    if outer.len()!=p.components.len()+1||disguised(center)?||state(p,&center.id)?.armor_position==0{return Ok(None);}
    let mut remaining:Vec<usize>=(0..outer.len()).collect();let mut selected=Vec::new();
    for required in &p.components {
        let mut found=None;for (at,index) in remaining.iter().enumerate(){let s=state(p,&outer[*index].id)?;
            if equal(p.mode,required,&required.tags,&s.comparison,&s.once_tags){found=Some(at);break;}}
        let Some(at)=found else{return Ok(None);};selected.push(remaining.remove(at));
    }
    let disguise=remaining[0];let item=outer[disguise];let s=state(p,&item.id)?;
    if disguised(item)?{return Ok(None);}
    let invis=if s.potion{s.invisible.context("native disguise potion effects are null")?}else{false};
    if !invis&&s.armor_position!=state(p,&center.id)?.armor_position{return Ok(None);}
    selected.insert(0,disguise);let mut available:Vec<usize>=(0..outer.len()).collect();
    for index in selected {
        let required=state(p,&outer[index].id)?;let mut found=None;
        for (at,index) in available.iter().enumerate(){let offered=state(p,&outer[*index].id)?;
            if equal(p.mode,&offered.comparison,&offered.comparison.tags,&required.comparison,&required.twice_tags){found=Some(at);break;}}
        let Some(at)=found else{return Ok(None);};available.remove(at);
    }
    Ok(Some(disguise))
}
pub fn transform(p:&DisguiseInfusion,center:&Item,disguise:&Item)->Result<Nbt>{
    let mut result=root(center.nbt.as_ref())?.cloned().unwrap_or_default();
    let mut data=match result.get("gadomancy"){Some(Nbt::Compound{value})=>value.clone(),_=>BTreeMap::new()};
    let s=state(p,&disguise.id)?;
    data.insert("disguise".into(),if s.potion{Nbt::Byte{value:"1".into()}}else{s.serialized.clone()});
    result.insert("gadomancy".into(),Nbt::Compound{value:data});Ok(Nbt::Compound{value:result})
}
fn comparison(c:&Comparison)->Result<()> {
    crate::identity::registry_name(&c.item.registry)?;
    if let Some(tag)=&c.item.nbt{tag.validate()?;root(Some(tag))?;}
    ensure!(c.limit>0&&c.ores.len()<=65536&&c.ores.iter().collect::<BTreeSet<_>>().len()==c.ores.len()
        &&c.default_ores.len()<=65536&&c.default_ores.windows(2).all(|p|p[0]<p[1]),"invalid disguise ore/limit observations");
    let keys:BTreeSet<_>=root(c.item.nbt.as_ref())?.into_iter().flat_map(|r|r.keys()).collect();
    ensure!(c.tags.keys().collect::<BTreeSet<_>>()==keys&&c.tags.values().all(|v|v.len()<=1048576),"disguise native NBT strings differ");Ok(())
}
pub(super) fn validate(p:&DisguiseInfusion,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(p.native_recipe=="makeo.gadomancy.common.crafting.InfusionDisguiseArmor"
        &&super::magic::ari_source(&r.source,super::magic::ARI_INFUSION)
        &&p.components.len()==4&&p.pedestal_limit>0&&r.inputs.len()==6&&r.outputs.len()==1
        &&r.grid.is_none()&&r.duration.is_none()&&r.energy.is_none(),"invalid disguise infusion shape");
    ensure!(p.states.len()<=65536&&p.states.windows(2).all(|s|s[0].id<s[1].id),"invalid disguise state table");
    for c in &p.components{comparison(c)?;}
    let mut native_ids=BTreeMap::new();
    for s in &p.states {
        comparison(&s.comparison)?;let item=items.get(s.id.as_str()).context("disguise observed item missing")?;
        ensure!(s.comparison.item==(Template{registry:item.registry.clone(),metadata:item.meta,nbt:item.nbt.clone()})
            &&s.comparison.limit==item.stack_limit as i32&&s.comparison.amount==1&&(0..=4).contains(&s.armor_position)
            &&s.comparison.ores.iter().collect::<BTreeSet<_>>()==item.tags.iter().collect::<BTreeSet<_>>()
            &&s.potion==(item.registry=="minecraft:potion")&&(s.potion||s.invisible.is_none()),"disguise item observation differs");
        if ["minecraft:pumpkin","minecraft:skull"].contains(&item.registry.as_str()){ensure!(s.armor_position==4,"native headgear position differs");}
        else if !item.armor{ensure!(s.armor_position==0,"nonarmor has an invented armor position");}
        let keys=s.comparison.tags.keys().collect::<BTreeSet<_>>();
        for tags in [&s.once_tags,&s.twice_tags]{ensure!(tags.keys().collect::<BTreeSet<_>>()==keys&&tags.values().all(|v|v.len()<=1048576),"invalid disguise copy observations");}
        s.serialized.validate()?;let serialized=root(Some(&s.serialized))?.context("missing native disguise serialization")?;
        ensure!(matches!(serialized.get("id"),Some(Nbt::Short{..}))&&serialized.get("Count")==Some(&Nbt::Byte{value:"1".into()})
            &&serialized.get("Damage")==Some(&Nbt::Short{value:(item.meta as i16).to_string()})
            &&serialized.get("tag")==item.nbt.as_ref()&&serialized.len()==if item.nbt.is_some(){4}else{3},"native disguise serialization differs");
        if let Some(prior)=native_ids.insert(&item.registry,serialized.get("id")){ensure!(prior==serialized.get("id"),"disguise native item id differs between variants");}
    }
    let magic=r.magic.as_ref().context("disguise magic missing")?;
    ensure!(magic.kind==MagicKind::Infusion&&magic.central==Some(0)&&magic.instability==Some(-20)
        &&magic.payment.is_none()&&!magic.creative&&magic.research.len()==1&&magic.research[0].key=="GADOMANCY.ARMORDISGUISE","disguise research or instability differs");
    for (slot,input) in r.inputs.iter().enumerate(){ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty(),"disguise input missing");
        for c in &input.choices{ensure!(c.amount=="1"&&matches!(c.rule,Match::Exact)&&matches!(c.consume,Consumption::Stack),"disguise choices are single-item whole-slot specimens");state(p,&c.id)?;
            if slot==0{ensure!(c.returns.is_empty(),"central disguise input has no container return");}
            for returned in &c.returns{ensure!(returned.kind==Kind::Item&&crate::identity::integer(&returned.amount,1,i64::from(p.pedestal_limit))?>0,"disguise container exceeds pedestal limit");}
        }
    }
    let out=&r.outputs[0];let change=out.change.as_ref().context("disguise result rule missing")?;
    ensure!(out.kind==Kind::Item&&out.slot==0&&out.role==OutputRole::Result&&out.quantity.is_none()&&out.amount.as_deref()==Some("1")
        &&out.chance.numerator=="1"&&out.chance.denominator=="1"&&matches!(change.action,Edit::DisguiseInfusion{})&&change.input==0
        &&!change.samples.is_empty()&&change.samples.len()<=65536&&out.id==change.samples[0].id,"invalid disguise result law");
    let bindings=change.bindings.as_ref().context("disguise input tuples missing")?;
    ensure!(bindings.len()==change.samples.len(),"disguise sample/binding count differs");
    let mut seen=BTreeSet::new();let mut covered=BTreeSet::new();
    for (tuple,sample) in bindings.iter().zip(&change.samples){ensure!(tuple.len()==6&&seen.insert(tuple.clone()),"duplicate/incomplete disguise tuple");
        let mut offered=Vec::new();for (slot,index) in tuple.iter().enumerate(){let index=index.context("empty disguise input binding")?;
            let choice=r.inputs[slot].choices.get(index as usize).context("disguise choice out of bounds")?;covered.insert((slot,index));offered.push(*items.get(choice.id.as_str()).context("disguise item fact missing")?);}
        let residual=select(p,offered[0],&offered[1..])?.context("native disguise tuple does not match startup and cycle")?;
        let result=items.get(sample.id.as_str()).context("disguise result item missing")?;
        ensure!(sample.amount=="1"&&result.registry==offered[0].registry&&result.meta==offered[0].meta
            &&result.nbt==Some(transform(p,offered[0],offered[residual+1])?),"disguise output does not preserve native serialized disguise");
    }
    ensure!(r.inputs.iter().enumerate().all(|(slot,i)|(0..i.choices.len()).all(|index|covered.contains(&(slot,index as u32)))),"disguise candidate has no valid full tuple");Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c(registry:&str,meta:i32,tag:Option<Nbt>)->Comparison{Comparison{item:Template{registry:registry.into(),metadata:meta,nbt:tag},amount:1,limit:64,ores:vec![],default_ores:vec![],tags:BTreeMap::new()}}
    #[test]
    fn reversed_comparison_uses_second_tags_and_second_wildcard(){
        let first=c("native:armor",0,None);let wildcard=c("native:armor",32767,None);
        assert!(equal(OreMode::Default,&first,&first.tags,&wildcard,&wildcard.tags));
        assert!(!equal(OreMode::Default,&wildcard,&wildcard.tags,&first,&first.tags));
        let mut tagged=c("native:armor",0,Some(Nbt::Compound{value:BTreeMap::from([("k".into(),Nbt::Int{value:"1".into()})])}));tagged.tags.insert("k".into(),"1".into());
        assert!(!equal(OreMode::Default,&first,&first.tags,&tagged,&tagged.tags));
        assert!(equal(OreMode::Default,&tagged,&tagged.tags,&first,&first.tags));
    }
}
