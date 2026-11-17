use super::{Consumption,Item,Kind,Match,OutputRole,Recipe};
use super::inventory_process::Selector;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum Need { Stack{template:Selector}, List{templates:Vec<Selector>}, Never }
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Definition {pub inputs:Option<Vec<Need>>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Spinning {pub version:String,pub selected:u32,pub definitions:Vec<Definition>}

impl Need {
    fn templates(&self)->&[Selector]{match self{Self::Stack{template}=>std::slice::from_ref(template),Self::List{templates}=>templates,Self::Never=>&[]}}
}
fn accepts(s:&Selector,item:&Item)->bool{s.registry==item.registry&&(s.metadata==32767||s.metadata==item.meta)}

/// The native hit flag is not reset per offer; a list removal does not break the requirement loop.
pub fn matches(definition:&Definition,offered:&[&Item])->bool{
    let Some(inputs)=&definition.inputs else{return false};
    if inputs.len()!=offered.len(){return false;}
    let mut remaining:Vec<_>=inputs.iter().collect();let mut hit=false;
    for item in offered {
        let mut at=0;
        while at<remaining.len(){
            match remaining[at]{
                Need::Stack{template} if accepts(template,item)=>{hit=true;remaining.remove(at);break;}
                Need::List{templates} if templates.iter().any(|s|accepts(s,item))=>{hit=true;remaining.remove(at);}
                _=>at+=1,
            }
        }
        if !hit{return false;}
    }
    remaining.is_empty()
}
pub(super) fn validate(p:&Spinning,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(p.version=="1.7.25-GTNH"&&p.definitions.len()<=4096&&r.source.owner=="WitchingGadgets"
        &&r.source.handler=="witchinggadgets.client.nei.NEISpinningWheelHandler"
        &&r.grid.is_none()&&r.magic.is_none()&&r.duration.is_none()&&r.energy.is_none()&&r.properties.is_empty(),"invalid native Witching Gadgets spinning process");
    let selected=p.definitions.get(p.selected as usize).context("native spinning registration missing")?;
    let requirements=selected.inputs.as_ref().context("native spinning null input array cannot match")?;
    let mut selectors=0usize;
    for definition in &p.definitions{if let Some(inputs)=&definition.inputs{
        ensure!(inputs.len()<=65536,"native spinning requirement budget exceeded");
        for need in inputs{selectors+=need.templates().len();ensure!(selectors<=262144,"native spinning alternative budget exceeded");
            for selector in need.templates(){crate::identity::registry_name(&selector.registry)?;ensure!(selector.metadata>=0,"invalid literal spinning metadata");}
        }
    }}
    ensure!(requirements.len()<=5&&r.inputs.len()==requirements.len(),"spinning physical input count differs");
    for (slot,(need,input)) in requirements.iter().zip(&r.inputs).enumerate(){
        ensure!(!need.templates().is_empty()&&input.slot==slot as u32&&input.kind==Kind::Item&&!input.choices.is_empty(),"unmatchable spinning requirement or missing specimen");
        for choice in &input.choices{let item=items.get(choice.id.as_str()).context("spinning specimen fact missing")?;
            ensure!(choice.amount=="1"&&matches!(choice.consume,Consumption::Consume)&&choice.returns.is_empty()
                &&need.templates().iter().any(|s|accepts(s,item)&&matches!(&choice.rule,Match::Wildcard{meta,nbt:true}if *meta==(s.metadata==32767))),"spinning native predicate or occupied-slot consumption differs");
        }
    }
    // A display tuple may be claimed by an earlier registration. Its process keeps that condition.
    ensure!(r.outputs.len()==1,"spinning result prototype missing");let output=&r.outputs[0];
    ensure!(output.slot==0&&output.kind==Kind::Item&&output.role==OutputRole::Result&&output.amount.is_some()
        &&output.quantity.is_none()&&output.change.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1","invalid empty-output spinning prototype");
    Ok(())
}
