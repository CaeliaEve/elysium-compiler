//! Inventory-dependent arcane recipes: templates alone do not describe their laws.
use super::{Aspect,Item,Kind,MagicKind,Match,OutputRole,Recipe};
use super::magic::{ari_source,ARI_SHAPED as SHAPED,ARI_SHAPELESS as SHAPELESS};
use crate::identity::Nbt;
use anyhow::{bail,ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum Operation {PrimalCharm,PhotoDeveloping,NodeAttunement}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ArcaneSpecial {
    pub operation:Operation,pub native_recipe:String,pub count:u8,
    /// Native item roles, not a complete enumeration of admissible NBT states.
    pub roles:BTreeMap<String,String>,pub ink_names:Vec<String>,
    /// Native aspect tag -> stable aspect identity for interpreting essence NBT.
    pub aspects:BTreeMap<String,String>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct StickyJar {pub native_recipe:String,pub jars:Vec<super::inventory_process::Selector>}

pub(super) fn validate_sticky(p:&StickyJar,r:&Recipe,items:&BTreeMap<&str,&Item>,aspects:&[Aspect])->Result<()> {
    ensure!(p.native_recipe=="makeo.gadomancy.common.crafting.RecipeStickyJar"&&ari_source(&r.source,SHAPED)&&r.inputs.len()==2&&r.outputs.len()==1&&r.duration.is_none()&&r.energy.is_none()&&r.properties.is_empty()&&!p.jars.is_empty()&&p.jars.len()<=4096,"invalid sticky jar branch");
    let grid=r.grid.as_ref().context("sticky jar vertical relation missing")?;
    ensure!(grid.width==1&&grid.height==2&&!grid.mirror&&grid.cells==vec![Some(0),Some(1)],"sticky jar requires slime immediately below jar");
    for selector in &p.jars{crate::identity::registry_name(&selector.registry)?;ensure!(selector.metadata>=0,"invalid jar predicate metadata");}
    for (slot,input) in r.inputs.iter().enumerate(){ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty(),"sticky jar inputs missing");
        for c in &input.choices{let item=items.get(c.id.as_str()).context("sticky jar input missing")?;
            ensure!(c.amount=="1"&&matches!(c.consume,super::Consumption::Consume)&&c.returns.is_empty(),"sticky jar uses ordinary unit consumption");
            let Match::Wildcard{meta,nbt:true}=c.rule else {bail!("sticky jar ignores NBT except native boolean predicate");};
            if slot==1{ensure!(item.registry=="minecraft:slime_ball"&&meta,"sticky jar second input is slime");}
            else{ensure!(p.jars.iter().any(|s|s.registry==item.registry&&(if meta{s.metadata==32767}else{s.metadata==item.meta})),"jar examples differ from registry predicate");
                ensure!((super::change::number(compound(item.nbt.as_ref())?.and_then(|t|t.get("isStickyJar")))? as i8)==0,"already-sticky specimen cannot match");}
        }
    }
    let out=&r.outputs[0];let change=out.change.as_ref().context("sticky jar patch missing")?;
    ensure!(out.kind==Kind::Item&&out.slot==0&&out.role==OutputRole::Result&&out.quantity.is_none()&&out.amount.as_deref()==Some("1")
        &&out.chance.numerator=="1"&&out.chance.denominator=="1"&&change.input==0,"invalid sticky jar result");
    let super::Edit::Patch{set,limits}=&change.action else {bail!("sticky jar must copy input and set one boolean");};
    ensure!(set==&BTreeMap::from([("isStickyJar".into(),Nbt::Byte{value:"1".into()})])&&limits.is_empty(),"sticky jar patch changes unrelated tags");
    let cost=r.magic.as_ref().context("sticky jar costs missing")?;
    ensure!(cost.kind==MagicKind::Arcane&&cost.central.is_none()&&cost.instability.is_none()&&cost.payment.is_none()&&cost.research.len()==1
        &&cost.aspects.len()==2&&cost.aspects.iter().all(|a|a.amount=="10"),"sticky jar magic facts differ");
    let expected:BTreeSet<_>=aspects.iter().filter(|a|["aqua","terra"].contains(&a.source.key.as_str())).map(|a|a.id.as_str()).collect();
    ensure!(expected.len()==2&&cost.aspects.iter().map(|a|a.aspect.as_str()).collect::<BTreeSet<_>>()==expected,"sticky jar requires water and earth");
    Ok(())
}
fn compound(tag:Option<&Nbt>)->Result<Option<&BTreeMap<String,Nbt>>>{match tag {None=>Ok(None),Some(Nbt::Compound{value})=>Ok(Some(value)),_=>bail!("arcane root tag is not a compound")}}

/// LinkedHashMap accumulation retains zero, signed values and Java int wrapping.
/// A null aspect is retained internally and skipped only by writeToNBT, as in TC4.
fn read_aspects(tag:Option<&Nbt>,known:&BTreeMap<String,String>)->Result<Vec<(Option<String>,i32)>>{
    let mut result=Vec::new();
    if let Some(Nbt::List{element,value})=compound(tag)?.and_then(|v|v.get("Aspects")) {
        if element!="compound" {return Ok(result);}
        ensure!(value.len()<=4096,"arcane aspect list exceeds budget");
        for entry in value {
            let Nbt::Compound{value}=entry else {bail!("invalid aspect entry");};
            let Some(key)=value.get("key") else {continue;};
            // Non-string keys need native NBT stringification; do not guess compound order.
            let Nbt::String{value:key}=key else {bail!("non-string aspect key requires a native observation");};
            let key=known.contains_key(key).then(||key.clone());
            add(&mut result,key,super::change::number(value.get("amount"))?);
        }
    }
    Ok(result)
}
fn add(values:&mut Vec<(Option<String>,i32)>,key:Option<String>,amount:i32){
    if let Some((_,value))=values.iter_mut().find(|(name,_)|*name==key){*value=value.wrapping_add(amount);}else{values.push((key,amount));}
}
fn write_aspects(values:Vec<(Option<String>,i32)>)->Nbt{
    let value:Vec<_>=values.into_iter().filter_map(|(key,amount)|key.map(|key|Nbt::Compound{value:BTreeMap::from([
        ("key".into(),Nbt::String{value:key}),("amount".into(),Nbt::Int{value:amount.to_string()})])})).collect();
    Nbt::List{element:if value.is_empty(){"end"}else{"compound"}.into(),value}
}

pub(super) fn validate(p:&ArcaneSpecial,r:&Recipe,items:&BTreeMap<&str,&Item>,aspects:&[Aspect])->Result<()> {
    let (native,handler,role_names,research)=match p.operation {
        Operation::PrimalCharm=>("dev.rndmorris.salisarcana.common.recipes.RecipeForgivingPrimalCharm",SHAPED,vec!["gold","result","shard"],"BASICARTIFACE"),
        Operation::PhotoDeveloping=>("witchinggadgets.common.util.recipe.PhotoDevelopingRecipe",SHAPELESS,vec!["paper","plate","result"],"SCANCAMERA"),
        Operation::NodeAttunement=>("shukaro.nodalmechanics.recipe.RecipeAttune",SHAPED,vec!["matrix","phial"],"NODECATALYZATION"),
    };
    ensure!(p.native_recipe==native&&ari_source(&r.source,handler)&&r.duration.is_none()&&r.energy.is_none()&&r.properties.is_empty()
        &&p.roles.keys().map(String::as_str).eq(role_names.into_iter())&&p.aspects.len()<=4096,"invalid special arcane source or roles");
    let expected_aspects:BTreeMap<_,_>=aspects.iter().map(|a|(a.source.key.clone(),a.id.clone())).collect();
    ensure!(p.aspects==expected_aspects,"special arcane aspect registry binding differs");
    let role=|name:&str|->Result<&Item>{items.get(p.roles.get(name).context("missing arcane role")?.as_str()).copied().context("arcane role item missing")};
    let cost=r.magic.as_ref().context("arcane costs missing")?;
    ensure!(cost.kind==MagicKind::Arcane&&cost.central.is_none()&&cost.instability.is_none()&&cost.payment.is_none()
        &&cost.research.len()==1&&cost.research[0].key==research,"arcane research or payment differs");
    ensure!(r.outputs.len()==1,"arcane result specimen missing");let out=&r.outputs[0];
    ensure!(out.kind==Kind::Item&&out.slot==0&&out.role==OutputRole::Result&&out.quantity.is_none()&&out.change.is_none()
        &&out.chance.numerator=="1"&&out.chance.denominator=="1","invalid arcane result specimen");
    for (slot,input) in r.inputs.iter().enumerate(){ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty()
        &&input.choices.iter().all(|c|c.amount=="1"),"invalid arcane input specimens");}
    let first=|slot:usize|->Result<&Item>{items.get(r.inputs.get(slot).context("arcane input missing")?.choices[0].id.as_str()).copied().context("arcane input fact missing")};
    let mut expected_cost=BTreeMap::new();
    let mut need=|tag:&str,n:u32|->Result<()>{expected_cost.insert(p.aspects.get(tag).context("missing primal aspect")?.clone(),n.to_string());Ok(())};
    let (registry,meta,amount,nbt)=match p.operation {
        Operation::PrimalCharm=>{
            ensure!(p.count==1&&p.ink_names.is_empty()&&r.inputs.len()==9,"invalid primal charm branch");
            let grid=r.grid.as_ref().context("primal charm requires physical cells")?;
            ensure!(grid.width==3&&grid.height==3&&!grid.mirror&&grid.cells.iter().copied().eq((0..9).map(Some)),"primal charm physical roles differ");
            let shard=role("shard")?;let gold=role("gold")?;let mut found=BTreeSet::new();
            for (slot,input) in r.inputs.iter().enumerate(){
                let expected=if slot==3||slot==5{gold}else{shard};
                let values: BTreeSet<_>=input.choices.iter().map(|c|items.get(c.id.as_str()).context("missing charm candidate")).collect::<Result<Vec<_>>>()?.iter().map(|i|i.meta).collect();
                ensure!(if slot==4{values==BTreeSet::from([6])}else if slot==3||slot==5{true}else{values==BTreeSet::from([0,1,2,3,4,5])},"primal shard domain differs");
                for c in &input.choices{let item=items[c.id.as_str()];ensure!(item.registry==expected.registry&&matches!(c.rule,Match::Wildcard{meta,nbt:true}if meta==(slot==3||slot==5)),"primal cell rule differs");}
                if slot!=3&&slot!=4&&slot!=5{found.insert(first(slot)?.meta);}
            }
            ensure!(found==BTreeSet::from([0,1,2,3,4,5]),"primal result specimen repeats a shard");
            for tag in ["aer","aqua","ignis","ordo","perditio","terra"]{need(tag,25)?;}
            let base=role("result")?;(&base.registry,base.meta,"1".into(),base.nbt.clone())
        },
        Operation::PhotoDeveloping=>{
            let count=p.count as usize;ensure!((1..=4).contains(&count)&&r.grid.is_none()&&r.inputs.len()==1+count*2
                &&!p.ink_names.is_empty()&&p.ink_names.len()<=4096&&p.ink_names.iter().all(|s|s.eq_ignore_ascii_case("dyeBlack")),"invalid photo branch");
            let plate=role("plate")?;let paper=role("paper")?;
            for (slot,input) in r.inputs.iter().enumerate(){for c in &input.choices{
                let item=items.get(c.id.as_str()).context("photo input missing")?;
                ensure!(if slot==0 {item.registry==plate.registry&&item.meta==9&&item.nbt.is_some()}
                    else if slot<=count {item.registry==paper.registry}
                    else {matches!(&c.rule,Match::Ore{name,exclusive:false}if p.ink_names.contains(name))},"photo input role differs");
            }}
            for tag in ["aer","aqua","ordo"]{need(tag,5*p.count as u32)?;}
            let base=role("result")?;ensure!(plate.meta==9&&base.registry==plate.registry&&base.meta==10,"photo item roles differ");
            (&base.registry,10,p.count.to_string(),first(0)?.nbt.clone())
        },
        Operation::NodeAttunement=>{
            let count=p.count as usize;ensure!((1..=8).contains(&count)&&p.ink_names.is_empty()&&r.grid.is_none()&&r.inputs.len()==1+count,"invalid node branch");
            let matrix=role("matrix")?;let phial=role("phial")?;ensure!(matrix.registry!=phial.registry&&phial.meta==1,"node role identities differ");
            for (slot,input) in r.inputs.iter().enumerate(){let expected=if slot==0{matrix}else{phial};for c in &input.choices{let item=items.get(c.id.as_str()).context("node input missing")?;
                ensure!(item.registry==expected.registry&&item.meta==expected.meta&&matches!(c.rule,Match::Wildcard{meta:false,nbt:true}),"node input predicate differs");
                if slot>0{ensure!(!read_aspects(item.nbt.as_ref(),&p.aspects)?.is_empty(),"essence phial must contain an aspect entry");}
            }}
            let mut values=Vec::new();for slot in 1..r.inputs.len(){for(key,amount)in read_aspects(first(slot)?.nbt.as_ref(),&p.aspects)?{add(&mut values,key,amount/4);}}
            ensure!(!values.is_empty(),"attunement needs at least one registered/null aspect entry");
            let input=first(0)?;for(key,amount)in read_aspects(input.nbt.as_ref(),&p.aspects)?{add(&mut values,key,amount);}
            let mut tags=compound(input.nbt.as_ref())?.cloned().unwrap_or_default();tags.insert("Aspects".into(),write_aspects(values));
            for tag in ["aer","aqua","ignis","ordo","perditio","terra"]{need(tag,2)?;}
            (&input.registry,input.meta,"1".into(),Some(Nbt::Compound{value:tags}))
        },
    };
    let got:BTreeMap<_,_>=cost.aspects.iter().map(|a|(a.aspect.clone(),a.amount.clone())).collect();
    ensure!(got==expected_cost&&got.len()==cost.aspects.len(),"inventory-dependent arcane cost differs");
    let result=items.get(out.id.as_str()).context("arcane result fact missing")?;
    ensure!(result.registry==*registry&&result.meta==meta&&result.nbt==nbt&&out.amount.as_deref()==Some(amount.as_str()),"arcane result specimen differs from native inventory law");
    Ok(())
}
