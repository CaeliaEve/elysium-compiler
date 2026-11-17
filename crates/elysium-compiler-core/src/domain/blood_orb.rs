//! Blood Orb workbench predicates, retaining integer early failure instead of rejecting overlap.
use super::{Consumption,Item,Kind,Match,Recipe};
use super::inventory_process::{Orb,Selector};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BloodOrbCraftingPredicate")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Predicate {pub minimum_orb:Option<i32>,pub alternatives:Vec<Selector>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BloodOrbCraftingProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Crafting {
    pub version:String,pub shaped:bool,pub width:u32,pub height:u32,pub mirror:bool,
    pub predicates:Vec<Option<Predicate>>,pub orbs:Vec<Orb>,
}
pub(super) fn validate(p:&Crafting,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    ensure!(p.version=="1.7.52"&&r.source.owner=="AWWayofTime"&&r.source.handler==if p.shaped{"WayofTime.alchemicalWizardry.client.nei.NEIBloodOrbShapedHandler"}else{"WayofTime.alchemicalWizardry.client.nei.NEIBloodOrbShapelessHandler"},"Blood Orb native source differs");
    ensure!(p.predicates.len()<=9&&p.orbs.len()<=4096&&r.duration.is_none()&&r.energy.is_none()&&r.magic.is_none()&&r.properties.is_empty(),"Blood Orb workbench lost native predicate roles");
    let mut orbs=BTreeMap::new();for orb in &p.orbs{ensure!(!orb.registry.is_empty()&&orb.registry.len()<=512&&orbs.insert(&orb.registry,orb.level).is_none(),"Duplicate or invalid blood orb tier");}
    if p.shaped{let grid=r.grid.as_ref().context("Blood Orb shaped grid missing")?;ensure!((1..=3).contains(&p.width)&&(1..=3).contains(&p.height)&&p.predicates.len()==(p.width*p.height)as usize&&grid.width==p.width&&grid.height==p.height&&grid.mirror==p.mirror,"Blood Orb shaped extent/mirror differs");}
    else{ensure!(p.width==0&&p.height==0&&!p.mirror&&r.grid.is_none()&&p.predicates.iter().all(Option::is_some),"Blood Orb shapeless process has a shaped layout or null predicate");}
    let mut slot=0;for (cell,predicate) in p.predicates.iter().enumerate(){let Some(predicate)=predicate else{ensure!(r.grid.as_ref().is_some_and(|g|g.cells.get(cell)==Some(&None)),"Blood Orb empty cell changed");continue;};
        ensure!(predicate.alternatives.len()<=65536&&if predicate.minimum_orb.is_some(){predicate.alternatives.is_empty()}else{!predicate.alternatives.is_empty()},"Blood Orb integer/item predicate domain differs");
        for s in &predicate.alternatives{ensure!(!s.registry.is_empty()&&s.registry.len()<=512&&s.metadata>=-1,"Invalid Blood Orb selector");}
        if let Some(grid)=&r.grid{ensure!(grid.cells.get(cell)==Some(&Some(slot)),"Blood Orb grid lost a predicate");}
        let input=r.inputs.get(slot as usize).context("Blood Orb input role missing")?;ensure!(input.kind==Kind::Item&&input.slot==slot&&!input.choices.is_empty(),"Blood Orb examples missing");
        let mut seen=BTreeSet::new();for c in &input.choices{let item=items.get(c.id.as_str()).context("Blood Orb item fact missing")?;
            let wildcard=if let Some(tier)=predicate.minimum_orb{ensure!(orbs.get(&item.registry).is_some_and(|level|*level>=tier),"Blood Orb minimum tier differs");true}
                else{ensure!(predicate.alternatives.iter().any(|s|s.registry==item.registry&&(s.metadata==item.meta||s.metadata==32767)),"Blood Orb item example rejects predicate");matches!(&c.rule,Match::Wildcard{meta:true,..})};
            ensure!(c.amount=="1"&&matches!(c.consume,Consumption::Consume|Consumption::Keep|Consumption::Damage{..}|Consumption::GtTool{..}|Consumption::NativeContainer{..})&&matches!(c.rule,Match::Wildcard{meta,nbt:true}if meta==wildcard),"Blood Orb one-per-cell / ignored-NBT law differs");
            if orbs.contains_key(&item.registry){ensure!(matches!(c.consume,Consumption::Keep)&&c.returns.is_empty(),"Blood orbs are retained");}
            seen.insert(&c.id);
        }slot+=1;
    }
    ensure!(r.inputs.len()==slot as usize&&r.outputs.len()==1,"Blood Orb role count differs");let out=&r.outputs[0];
    ensure!(out.kind==Kind::Item&&out.slot==0&&out.amount.is_some()&&out.change.is_none()&&out.quantity.is_none()&&out.chance.numerator=="1"&&out.chance.denominator=="1","Blood Orb workbench result differs");Ok(())
}
