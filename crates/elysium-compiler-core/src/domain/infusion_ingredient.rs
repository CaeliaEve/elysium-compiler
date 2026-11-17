//! TC4Tweaks Enhanced startup predicates, independently of altar cycle templates.
use super::{infusion::Template,Item};
use anyhow::{ensure,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum Predicate {
    Never,
    Ore {name:String,exclusive:bool},
    Items {templates:Vec<Template>,nbt:bool},
    Any {options:Vec<Predicate>},
    /// Resolved using the source-bound current network mode, without invoking the supplier.
    Deferred {template:Template,rule:Box<Predicate>},
}
pub fn validate(predicates:&[Predicate])->Result<()> {
    ensure!(!predicates.is_empty()&&predicates.len()<=4096,"invalid enhanced predicate count");
    let mut nodes=0;for predicate in predicates{validate_node(predicate,0,&mut nodes)?;}Ok(())
}
fn template(t:&Template)->Result<()> {
    crate::identity::registry_name(&t.registry)?;
    if let Some(nbt)=&t.nbt{nbt.validate()?;ensure!(matches!(nbt,crate::identity::Nbt::Compound{..}),"enhanced template root is not a compound");}Ok(())
}
fn validate_node(p:&Predicate,depth:usize,nodes:&mut usize)->Result<()> {
    *nodes+=1;ensure!(depth<=16&&*nodes<=262144,"enhanced ingredient graph exceeds budget");
    match p {
        Predicate::Never=>{},
        Predicate::Ore{name,..}=>ensure!(name.len()<=4096,"enhanced ore name exceeds budget"),
        Predicate::Items{templates,..}=>{ensure!(templates.len()<=4096,"enhanced item predicate exceeds budget");for t in templates{template(t)?;}},
        Predicate::Any{options}=>{ensure!(options.len()<=4096,"enhanced OR exceeds budget");for p in options{validate_node(p,depth+1,nodes)?;}},
        Predicate::Deferred{template:t,rule}=>{template(t)?;validate_node(rule,depth+1,nodes)?;},
    }Ok(())
}
pub fn matches(p:&Predicate,item:&Item)->Result<bool>{matches_at(p,item,0)}
fn matches_at(p:&Predicate,item:&Item,depth:usize)->Result<bool> {
    ensure!(depth<=16,"enhanced match depth exceeded");
    match p {
        Predicate::Never=>Ok(false),
        Predicate::Ore{name,exclusive}=>Ok((!*exclusive||item.tags.len()==1)&&item.tags.contains(name)),
        Predicate::Items{templates,nbt}=>{
            for t in templates {
                if t.registry!=item.registry||t.metadata!=32767&&t.metadata!=item.meta{continue;}
                if !*nbt||match (&t.nbt,&item.nbt){(None,None)=>true,(Some(a),Some(b))=>a.native_equal(b)?,_=>false}{return Ok(true);}
            }Ok(false)
        },
        Predicate::Any{options}=>{for p in options{if matches_at(p,item,depth+1)?{return Ok(true);}}Ok(false)},
        Predicate::Deferred{rule,..}=>matches_at(rule,item,depth+1),
    }
}
/// Enhanced.matches returns true after removing all requirements, even with leftovers.
pub fn components(predicates:&[Predicate],outer:&[&Item])->Result<bool> {
    let mut available=outer.to_vec();
    for p in predicates{let mut first=None;for (index,item) in available.iter().enumerate(){if matches(p,item)?{first=Some(index);break;}}
        let Some(index)=first else{return Ok(false);};available.remove(index);}
    Ok(true)
}
