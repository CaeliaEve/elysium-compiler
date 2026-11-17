//! ProjectRed's fixed physical cells and ordered greedy predicates, independent of NEI caches.
use super::{Consumption,Item,Kind,Match,OutputRole,Recipe};
use crate::identity::Nbt;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="ProjectRedPredicateKind")]
#[serde(rename_all="camelCase")]
pub enum PredicateKind{Item,Ore,Micro}

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="ProjectRedPredicate")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Predicate {
    pub kind:PredicateKind,pub registry:Option<String>,pub metadata:Option<i32>,pub nbt:Option<Nbt>,
    pub check_nbt:bool,pub ores:Vec<String>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="ProjectRedCrafting")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Crafting {
    pub version:String,pub shaped:bool,pub minimum_slots:u32,pub predicates:Vec<Option<Predicate>>,pub ignored_outputs:u32,
}
impl Predicate {
    fn validate(&self)->Result<()>{
        if self.kind==PredicateKind::Ore {
            ensure!(self.registry.is_none()&&self.metadata.is_none()&&self.nbt.is_none()&&!self.check_nbt
                &&self.ores.len()<=4096&&self.ores.iter().all(|s|!s.is_empty()&&s.len()<=512)
                &&self.ores.iter().collect::<BTreeSet<_>>().len()==self.ores.len(),"Invalid ProjectRed ore predicate");
        }else{
            ensure!(self.registry.as_ref().is_some_and(|s|!s.is_empty()&&s.len()<=512)&&self.metadata.is_some()
                &&self.ores.is_empty()&&(self.kind!=PredicateKind::Micro||self.check_nbt),"Invalid ProjectRed key predicate");
        }Ok(())
    }
    fn accepts(&self,item:&Item)->Result<bool>{
        if self.kind==PredicateKind::Ore{return Ok(self.ores.iter().any(|s|item.tags.contains(s)));}
        let tags=match(self.nbt.as_ref(),item.nbt.as_ref()){(None,None)=>true,(Some(a),Some(b))=>a.native_equal(b)?,_=>false};
        Ok(self.registry.as_deref()==Some(item.registry.as_str())
            &&(self.metadata==Some(item.meta)||self.kind==PredicateKind::Item&&(self.metadata==Some(32767)||item.meta==32767))
            &&(!self.check_nbt||tags))
    }
}
pub(super) fn validate(p:&Crafting,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    ensure!(p.version=="4.12.6-GTNH"&&r.source.handler==format!("mrtjp.projectred.core.libmc.recipe.PR{}RecipeHandler",if p.shaped{"Shaped"}else{"Shapeless"})
        &&r.magic.is_none()&&r.duration.is_none()&&r.energy.is_none()&&r.properties.is_empty(),"ProjectRed native family or workbench semantics differ");
    ensure!(p.minimum_slots<=9&&p.ignored_outputs<65536&&p.predicates.len()<=9
        &&if p.shaped{p.predicates.len()==9}else{p.predicates.len()==p.minimum_slots as usize&&p.predicates.iter().all(Option::is_some)},"ProjectRed physical predicate budget or holes differ");
    let mut slot=0u32;let mut cells=Vec::new();
    for predicate in &p.predicates {
        let Some(predicate)=predicate else{cells.push(None);continue;};predicate.validate()?;cells.push(Some(slot));
        let input=r.inputs.get(slot as usize).context("ProjectRed native input is missing")?;
        ensure!(input.kind==Kind::Item&&input.slot==slot&&!input.choices.is_empty(),"ProjectRed native input role differs");
        for choice in &input.choices {
            let item=items.get(choice.id.as_str()).context("ProjectRed input fact missing")?;
            ensure!(choice.amount=="1"&&matches!(choice.consume,Consumption::Consume|Consumption::Keep|Consumption::Damage{..}|Consumption::GtTool{..} | Consumption::NativeContainer { .. })
                &&predicate.accepts(item)?,"ProjectRed displayed input does not satisfy its native predicate");
            if predicate.kind==PredicateKind::Ore {
                ensure!(matches!(&choice.rule,Match::Ore{name,exclusive:false}if predicate.ores.contains(name)&&item.tags.contains(name)),"ProjectRed ore source became a fixed preview list");
            }else{
                ensure!(matches!(choice.rule,Match::Wildcard{meta,nbt}if meta==(predicate.kind==PredicateKind::Item&&predicate.metadata==Some(32767))&&nbt==!predicate.check_nbt),"ProjectRed key wildcard/NBT policy differs");
            }
        }slot+=1;
    }
    ensure!(r.inputs.len()==slot as usize&&r.outputs.len()==1,"ProjectRed workbench arity differs");
    if p.shaped{let grid=r.grid.as_ref().context("ProjectRed physical cells missing")?;ensure!(grid.width==3&&grid.height==3&&!grid.mirror&&grid.cells==cells,"ProjectRed absolute cells changed");}
    else{ensure!(r.grid.is_none(),"ProjectRed shapeless rule acquired a fixed grid");}
    let output=&r.outputs[0];ensure!(output.kind==Kind::Item&&output.slot==0&&matches!(output.role,OutputRole::Result)&&output.amount.is_some()
        &&output.change.is_none()&&output.quantity.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1","ProjectRed returns only its first fixed output");
    Ok(())
}
