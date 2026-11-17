//! RandomThings keeps identity-key map order and cached output behavior, not generic allocation.
use super::{Consumption,Item,Kind,Match,OutputRole,Quantity,Recipe};
use super::inventory_process::Prototype;
use crate::identity::{integer,Nbt};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="ImbuingPredicate")]
#[serde(deny_unknown_fields)]
pub struct Predicate {pub registry:String,pub metadata:i32,pub nbt:Option<Nbt>,pub ores:Vec<String>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="ImbuingDefinition")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Definition {pub base:Predicate,pub ingredients:Vec<Predicate>,pub output_alias:u32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="RandomThingsImbuingProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Imbuing {
    pub version:String,pub threshold_bits:String,pub selected:u32,pub definitions:Vec<Definition>,
    pub base_input:u32,pub base_slot:u32,pub reagent_slots:u32,pub output_slot:u32,pub result:Prototype,
}
fn predicate(p:&Predicate)->Result<()>{ensure!(!p.registry.is_empty()&&p.registry.len()<=512&&p.metadata>=-1&&p.ores.len()<=4096
    &&p.ores.iter().all(|s|!s.is_empty()&&s.len()<=512)&&p.ores.iter().collect::<BTreeSet<_>>().len()==p.ores.len(),"Invalid imbuing predicate");if let Some(nbt)=&p.nbt{ensure!(matches!(nbt,Nbt::Compound{..}),"Imbuing NBT root is not a compound");nbt.validate()?;}Ok(())}
pub(super) fn validate(p:&Imbuing,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    ensure!(p.version=="2.6.6"&&r.source.owner=="RandomThings"&&r.source.handler=="lumien.randomthings.Handler.ModCompability.NEI.ImbuingStationRecipeHandler"
        &&p.threshold_bits.len()==8&&p.threshold_bits.bytes().all(|c|c.is_ascii_digit()||(b'a'..=b'f').contains(&c))&&p.base_input==65535&&p.base_slot==3&&p.reagent_slots==3&&p.output_slot==4
        &&!p.definitions.is_empty()&&p.definitions.len()<=4096&&r.duration.is_none()&&r.energy.is_none()&&r.grid.is_none()&&r.magic.is_none()&&r.properties.is_empty(),"Imbuing native state/source law differs");
    let mut terms=0;for d in &p.definitions{predicate(&d.base)?;ensure!(d.ingredients.len()<=65536&&d.output_alias<(p.definitions.len()as u32),"Imbuing source budget or alias invalid");terms+=d.ingredients.len();ensure!(terms<=262144,"Imbuing cumulative predicates exceed budget");for s in &d.ingredients{predicate(s)?;}}
    let selected=p.definitions.get(p.selected as usize).context("Selected imbuing registration missing")?;
    ensure!(r.inputs.len()==selected.ingredients.len()+1,"Imbuing predicate roles differ");
    for (at,input) in r.inputs.iter().enumerate(){let base=at==selected.ingredients.len();let s=if base{&selected.base}else{&selected.ingredients[at]};
        ensure!(input.kind==Kind::Item&&input.slot==if base{65535}else{at as u32}&&!input.choices.is_empty(),"Imbuing predicate example missing");
        for c in &input.choices{let item=items.get(c.id.as_str()).context("Imbuing item fact missing")?;ensure!(c.amount=="1"&&c.returns.is_empty()&&matches!(c.consume,Consumption::Allocated),"Imbuing depletes actual stock without crafting containers");
            ensure!(match &c.rule{Match::Exact=>item.registry==s.registry&&item.meta==s.metadata&&item.nbt==s.nbt,Match::Ore{name,exclusive:false}=>s.ores.contains(name)&&item.tags.contains(name),_=>false},"Imbuing candidate lost content/ore predicate");
        }
    }
    let amount=integer(&p.result.amount,i32::MIN as i64,i32::MAX as i64)?;ensure!(items.contains_key(p.result.id.as_str())&&r.outputs.len()==usize::from(amount>0),"Imbuing native result prototype missing");
    if amount>0{let o=&r.outputs[0];ensure!(o.kind==Kind::Item&&o.role==OutputRole::Result&&o.slot==0&&o.id==p.result.id&&o.amount.is_none()&&o.change.is_none()&&o.chance.numerator=="1"&&o.chance.denominator=="1"
        &&matches!(&o.quantity,Some(Quantity::Potential{stat,nominal,sample:None,condition:Some(condition),parameters:None})if stat=="randomthings:imbuingState"&&nominal==&amount.min(64).to_string()&&!condition.is_empty()),"Imbuing cached output must remain conditional");}Ok(())
}
