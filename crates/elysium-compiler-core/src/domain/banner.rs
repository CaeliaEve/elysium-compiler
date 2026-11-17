//! Et Futurum 2.6.2.25: ordered native masks, shared dye color and copied banner history.
use super::{Consumption,Edit,Item,Kind,Match,OutputRole,Recipe};
use crate::identity::{item_id,Nbt};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct BannerProcess {
    pub native_recipe:String,pub pattern:u32,pub rules:Vec<PatternRule>,pub mode:BannerMode,
    pub color:u8,pub palette:Vec<Dye>,pub color_ores:Vec<ColorOre>,pub banner:u32,pub dyes:Vec<u32>,pub max_patterns:u8,pub layout:Vec<Option<u32>>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PatternRule {pub id:String,pub mask:Option<Vec<String>>,pub catalyst:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Dye {pub id:String,pub color:u8,pub ores:Vec<String>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ColorOre {pub name:String,pub color:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum BannerMode {Mask,Catalyst,CatalystNoDye}

pub(super) fn validate(p:&BannerProcess,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(r.source.handler=="ganymedes01.etfuturum.compat.nei.BannerPatternHandler"
        &&p.native_recipe=="ganymedes01.etfuturum.recipes.crafting.RecipeAddPattern"
        &&p.max_patterns==6&&p.color<16&&r.magic.is_none()&&r.duration.is_none()&&r.energy.is_none(),"Invalid native banner process");
    ensure!(!p.rules.is_empty()&&p.rules.len()<=4096&&p.pattern<p.rules.len() as u32,"Banner rule outside native inventory");
    let mut ids=BTreeSet::new();
    for rule in &p.rules {
        ensure!(!rule.id.is_empty()&&rule.id.len()<=128&&ids.insert(&rule.id),"Invalid banner pattern identifier");
        match (&rule.mask,&rule.catalyst){
            (Some(mask),None)=>ensure!(mask.len()==3&&mask.iter().all(|s|s.is_ascii()&&s.len()==3)
                &&mask.iter().any(|s|s.contains(' '))&&mask.iter().any(|s|s.chars().any(|c|c!=' ')),"Invalid banner mask"),
            (None,Some(id))=>{let c=items.get(id.as_str()).context("Missing native banner catalyst")?;ensure!(c.meta>=0,"Invalid banner catalyst metadata");},
            _=>anyhow::bail!("Banner rule requires a mask or catalyst"),
        }
    }
    let names=["dyeBlack","dyeRed","dyeGreen","dyeBrown","dyeBlue","dyePurple","dyeCyan","dyeLightGray","dyeGray","dyePink","dyeLime","dyeYellow","dyeLightBlue","dyeMagenta","dyeOrange","dyeWhite"];
    ensure!(p.color_ores.len()==16&&p.color_ores.iter().enumerate().all(|(i,c)|c.name==names[i]&&c.color==15-i as u8),"Banner native color mapping changed");
    ensure!(p.palette.len()<=65536,"Banner palette exceeds budget");
    let mut unique=BTreeSet::new();
    for d in &p.palette {
        let item=items.get(d.id.as_str()).context("Missing banner dye fact")?;
        ensure!(d.color<16&&item.nbt.is_none()&&item.meta>=0&&item.meta!=32767&&unique.insert(&d.id),"Invalid concrete banner dye");
        let ores:BTreeSet<_>=d.ores.iter().collect();
        ensure!(ores.len()==d.ores.len()&&ores==item.tags.iter().collect(),"Banner dye ore order does not describe its native memberships");
        let actual=d.ores.iter().find_map(|ore|p.color_ores.iter().find(|c|&c.name==ore).map(|c|c.color));
        ensure!(actual==Some(d.color),"Banner dye color differs from first native ore membership");
    }
    ensure!(r.grid.is_none()&&p.layout.len()==9,"Banner native process owns dynamic placement; generic fixed grid is invalid");
    let b=r.inputs.iter().find(|s|s.slot==p.banner).context("Banner input missing")?;
    ensure!(b.kind==Kind::Item&&b.choices.len()==1,"Banner requires one history-preserving input");
    let anchor=items.get(b.choices[0].id.as_str()).context("Banner fact missing")?;
    ensure!(matches!(b.choices[0].rule,Match::BannerInput{max_patterns:6})
        &&matches!(b.choices[0].consume,Consumption::Consume)&&b.choices[0].returns.is_empty(),"Banner history admission/debit changed");
    let dye_slots:BTreeSet<_>=p.dyes.iter().copied().collect();
    ensure!(dye_slots.len()==p.dyes.len()&&!dye_slots.contains(&p.banner),"Banner dye slots repeated");
    let mut cells=vec![None;9];
    for input in &r.inputs {
        ensure!(input.kind==Kind::Item&&!input.choices.is_empty()&&input.choices.iter().all(|c|c.amount=="1"),"Banner cell consumes one item");
        let position=p.layout.iter().position(|s|*s==Some(input.slot)).context("Banner input absent from grid")?;
        ensure!(p.layout.iter().filter(|s|**s==Some(input.slot)).count()==1,"Banner input occupies multiple cells");
        cells[position]=Some(items[input.choices[0].id.as_str()]);
        if input.slot==p.banner {continue;}
        for c in &input.choices {
            ensure!(matches!(c.rule,Match::Wildcard{meta:false,nbt:true}),"Banner non-banner cells ignore NBT only");
            if dye_slots.contains(&input.slot){ensure!(p.palette.iter().any(|d|d.id==c.id&&d.color==p.color),"Mixed native dye colors admitted");}
            else {let selected=&p.rules[p.pattern as usize];let catalyst=selected.catalyst.as_ref().context("Unexpected non-dye cell in mask")?;
                let native=items[catalyst.as_str()];let offered=items[c.id.as_str()];ensure!(offered.registry==native.registry&&offered.meta==native.meta
                    &&color(p,offered,items).is_none(),"Banner catalyst must pass item equality after dye classification");}
        }
    }
    ensure!(p.dyes.iter().all(|slot|r.inputs.iter().any(|s|s.slot==*slot)),"Banner dye slot missing");
    let selected=&p.rules[p.pattern as usize];
    ensure!(match p.mode {BannerMode::Mask=>selected.mask.is_some()&&!p.dyes.is_empty(),
        BannerMode::Catalyst=>selected.catalyst.is_some()&&p.dyes.len()==1,
        BannerMode::CatalystNoDye=>selected.catalyst.is_some()&&p.dyes.is_empty()&&p.color==15},"Banner optional dye law changed");
    ensure!(p.rules.iter().position(|rule|matches_rule(p,rule,&cells,&anchor.registry,items))==Some(p.pattern as usize),"Banner sample violates native ordered pattern priority");
    ensure!(r.outputs.len()==1,"Banner must return one updated item");let output=&r.outputs[0];
    ensure!(output.kind==Kind::Item&&matches!(output.role,OutputRole::Result)&&output.amount.as_deref()==Some("1")
        &&output.quantity.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1","Banner fixed count changed");
    let change=output.change.as_ref().context("Banner output lost its copied history")?;
    ensure!(change.input==p.banner&&matches!(&change.action,Edit::BannerPattern{pattern,color} if pattern==&selected.id&&*color==p.color),"Banner transform differs from native selection");
    Ok(())
}
fn color(p:&BannerProcess,item:&Item,items:&BTreeMap<&str,&Item>)->Option<u8>{
    p.palette.iter().find(|d|{let sample=items[d.id.as_str()];sample.registry==item.registry&&sample.meta==item.meta}).map(|d|d.color)
}
fn matches_rule(p:&BannerProcess,rule:&PatternRule,cells:&[Option<&Item>],banner:&str,items:&BTreeMap<&str,&Item>)->bool{
    if let Some(id)=&rule.catalyst {
        let catalyst=items[id.as_str()];let(mut found,mut dye)=(false,false);
        for item in cells.iter().flatten().filter(|i|i.registry!=banner){
            if color(p,item,items).is_some(){if dye{return false;}dye=true;}
            else{if found||item.registry!=catalyst.registry||item.meta!=catalyst.meta{return false;}found=true;}
        }return found;
    }
    let mask=rule.mask.as_ref().unwrap();let mut dye=None;
    for (pos,cell) in cells.iter().enumerate(){
        let required=mask[pos/3].as_bytes()[pos%3]!=b' ';
        match cell.filter(|i|i.registry!=banner){
            Some(item)=>{let Some(c)=color(p,item,items)else{return false;};if !required||dye.is_some_and(|old|old!=c){return false;}dye=Some(c);},
            None=>if required{return false;},
        }
    }true
}
pub(super) fn pattern_count(nbt:Option<&Nbt>)->usize{
    let Some(Nbt::Compound{value:root})=nbt else{return 0;};
    let Some(Nbt::Compound{value:block})=root.get("BlockEntityTag")else{return 0;};
    match block.get("Patterns"){Some(Nbt::List{element,value})if element=="compound"=>value.len(),_=>0}
}
pub(super) fn append(item:&Item,pattern:&str,color:u8)->Result<super::Stack>{
    let mut root=match &item.nbt{Some(Nbt::Compound{value})=>value.clone(),None=>BTreeMap::new(),_=>anyhow::bail!("Banner root is not a compound")};
    let block=root.entry("BlockEntityTag".into()).or_insert_with(||Nbt::Compound{value:BTreeMap::new()});
    if !matches!(block,Nbt::Compound{..}){*block=Nbt::Compound{value:BTreeMap::new()};}
    let Nbt::Compound{value:block}=block else{unreachable!()};
    let history=block.entry("Patterns".into()).or_insert_with(||Nbt::List{element:"end".into(),value:vec![]});
    if !matches!(history,Nbt::List{..}){*history=Nbt::List{element:"end".into(),value:vec![]};}
    let Nbt::List{element,value}=history else{unreachable!()};
    // getTagList(10) on a nonempty list of another type returns a detached empty list.
    if element=="end"||element=="compound" {
        *element="compound".into();value.push(Nbt::Compound{value:BTreeMap::from([
            ("Pattern".into(),Nbt::String{value:pattern.into()}),("Color".into(),Nbt::Int{value:color.to_string()})])});
    }
    Ok(super::Stack{id:item_id(&item.registry,item.meta,Some(&Nbt::Compound{value:root}))?,amount:"1".into()})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrong_typed_history_is_preserved_even_when_empty() {
        let source=crate::source::Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")).unwrap();
        let mut item=crate::domain::Domain::load(&source).unwrap().items[0].clone();
        for value in [vec![],vec![Nbt::Int{value:"4".into()}]] {
            item.nbt=Some(Nbt::Compound{value:BTreeMap::from([("BlockEntityTag".into(),Nbt::Compound{value:BTreeMap::from([
                ("Patterns".into(),Nbt::List{element:"int".into(),value})])})])});
            item.id=item_id(&item.registry,item.meta,item.nbt.as_ref()).unwrap();
            assert_eq!(pattern_count(item.nbt.as_ref()),0);
            assert_eq!(append(&item,"br",14).unwrap().id,item.id,"native wrong-type list is retained, not repaired");
        }
    }
}
