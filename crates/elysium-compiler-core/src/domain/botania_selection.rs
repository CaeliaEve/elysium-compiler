//! Ordered selection with explicit resolved read traces, not a published
//! process/program or a simulation of live cache writes and clock progression.
use super::botania_bee::{Outcome,Source,Stack};
use super::botania_cache::Read;
use crate::identity::Nbt;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,Deserialize,Serialize,JsonSchema)]
#[schemars(rename="BeePoolItemPredicate")]
#[serde(deny_unknown_fields)]
pub struct Predicate {pub registry:String,pub metadata:i32}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(tag="kind",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub enum Entry {
    Bee{alchemy:bool,conjuration:bool,mana:i32,form:String,input_species:String,output_species:String},
    Item{alchemy:bool,conjuration:bool,mana:i32,predicates:Vec<Predicate>,output:Stack},
    Mini{alchemy:bool,conjuration:bool,mana:i32,key:String,output:Stack},
}
#[derive(Clone,Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {pub index:u32,pub reads:Vec<Read>}
#[derive(Clone,Debug,Serialize)]
#[serde(tag="state",rename_all="camelCase")]
pub enum SelectionResult {NotMatched,NativeError{index:u32},InsufficientMana{index:u32},Selected{index:u32,output:Stack}}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Selection {pub result:SelectionResult,pub offered_after:Option<Stack>,pub visited:u32}

impl Entry {
    fn conditions(&self)->(bool,bool,i32) {
        match self {Self::Bee{alchemy,conjuration,mana,..}|Self::Item{alchemy,conjuration,mana,..}|
            Self::Mini{alchemy,conjuration,mana,..}=>(*alchemy,*conjuration,*mana)}
    }
}
impl Source {
    pub fn select(&self,entries:&[Entry],offered:Option<Stack>,alchemy:bool,conjuration:bool,mana:i32,traces:&[Trace])->Result<Selection> {
        ensure!(entries.len()<=4096&&traces.len()<=entries.len()&&serde_json::to_vec(entries)?.len()<=16*1024*1024,
            "Native pool selection exceeds its budget");
        let Some(mut owned)=offered else {
            ensure!(traces.is_empty(),"Unused native pool selection trace");
            return Ok(Selection{result:SelectionResult::NotMatched,offered_after:None,visited:0});
        };
        if let Some(nbt)=&owned.nbt {nbt.validate()?;}
        let mut result=SelectionResult::NotMatched;let mut at=0;let mut visited=0;let mut predicates_seen=0;
        for (index,entry) in entries.iter().enumerate() {
            visited+=1;
            let output=match entry {
                Entry::Bee{form,input_species,output_species,..}=>{
                    let trace=traces.get(at).context("Missing native pool selection trace")?;at+=1;
                    ensure!(trace.index as usize==index,"Out-of-order native pool selection trace");
                    let transition=self.transition(form,input_species,output_species,Some(owned),trace.reads.clone())?;
                    owned=transition.offered_after.context("Native pool selection lost the offered stack")?;
                    match transition.result {
                        Outcome::NativeError=>{
                            ensure!(at==traces.len(),"Unused native pool selection trace");
                            return Ok(Selection{result:SelectionResult::NativeError{index:index as u32},offered_after:Some(owned),visited});
                        }
                        Outcome::NotMatched=>None,
                        Outcome::Matched{output}=>Some(output),
                    }
                }
                Entry::Item{predicates,output,..}=>{
                    let mut accepted=false;
                    for predicate in predicates {
                        predicates_seen+=1;ensure!(predicates_seen<=65536,"Native pool selection predicate budget exceeded");
                        if predicate.registry==owned.registry&&(predicate.metadata==32767||predicate.metadata==owned.meta){accepted=true;break;}
                    }
                    accepted.then(||output.clone())
                }
                Entry::Mini{key,output,..}=>{
                    let tag=if let Some(Nbt::Compound{value})=&owned.nbt {value.get("type")}else{None};
                    let observed=match tag {None=>"",Some(Nbt::String{value})=>value,
                        _=>anyhow::bail!("Native pool selection lacks a non-string subtile observation")};
                    (key==observed).then(||output.clone())
                }
            };
            let (needs_alchemy,needs_conjuration,cost)=entry.conditions();
            // Match effects precede catalyst checks in TilePool.
            if let Some(output)=output.filter(|_|(!needs_alchemy||alchemy)&&(!needs_conjuration||conjuration)) {
                result=if mana<cost {SelectionResult::InsufficientMana{index:index as u32}}
                    else{SelectionResult::Selected{index:index as u32,output}};
                break;
            }
        }
        ensure!(at==traces.len(),"Unused native pool selection trace");
        Ok(Selection{result,offered_after:Some(owned),visited})
    }
}
