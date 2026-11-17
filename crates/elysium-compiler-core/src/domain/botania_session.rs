//! Serial detached pool session: cache/repair, registry selection and recipe
//! output caches use one state. Published program/admission remains separate.
use super::botania_bee::{Outcome, Source, Stack};
use super::botania_cache::Read as GenomeRead;
use super::botania_cache_machine::{Machine, Read, State};
use super::botania_selection::{Predicate, Selection, SelectionResult};
use crate::identity::Nbt;
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(rename="BeePoolSessionEntry")]
#[serde(tag="kind",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub enum Entry {
    Bee { alchemy:bool,conjuration:bool,mana:i32,form:String,
        #[serde(rename="inputSpecies")] input_species:String,
        #[serde(rename="outputSpecies")] output_species:String,
        #[serde(rename="displayInput")] display_input:Stack,
        prototype:Stack,#[serde(rename="cachedOutput")] cached_output:Stack },
    Item { alchemy:bool,conjuration:bool,mana:i32,predicates:Vec<Predicate>,output:Stack },
    Mini { alchemy:bool,conjuration:bool,mana:i32,key:String,output:Stack,
        #[serde(default,skip_serializing_if="Vec::is_empty")] coercions:Vec<SubtileObservation> },
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BeePoolSubtileObservation")]
#[serde(deny_unknown_fields)]
pub struct SubtileObservation {pub input:Stack,pub value:String}
pub struct Session {
    source:Source,entries:Vec<Entry>,cached:Vec<Option<Stack>>,cache:Machine,valid:bool,
}
impl Session {
    pub fn new(source:Source,entries:Vec<Entry>,cache:State)->Result<Self>{
        ensure!(entries.len()<=4096&&serde_json::to_vec(&entries)?.len()<=16*1024*1024,"Native pool session exceeds registry budget");
        let cached=entries.iter().map(|entry|match entry {Entry::Bee{cached_output,..}=>Some(cached_output.clone()),_=>None}).collect();
        Ok(Self{source,entries,cached,cache:Machine::new(cache)?,valid:true})
    }
    pub fn outputs(&self)->Result<&[Option<Stack>]>{
        ensure!(self.valid,"Native pool session was invalidated by an aborted transition");Ok(&self.cached)
    }
    pub fn cache_state(&self)->Result<State>{
        ensure!(self.valid,"Native pool session was invalidated by an aborted transition");Ok(self.cache.snapshot())
    }
    pub fn take_output(&mut self,index:usize)->Result<Stack>{
        ensure!(self.valid,"Native pool session was invalidated by an aborted transition");
        let Some(Entry::Bee{prototype,..})=self.entries.get(index) else{anyhow::bail!("Invalid native recipe output-cache reference");};
        Ok(self.cached[index].replace(prototype.clone()).ok_or_else(||anyhow::anyhow!("Missing native recipe output cache"))?)
    }
    pub fn select<F>(&mut self,offered:Option<Stack>,alchemy:bool,conjuration:bool,mana:i32,clock:&mut F)->Result<Selection>
    where F:FnMut()->Result<i64>{
        ensure!(self.valid,"Native pool session was invalidated by an aborted transition");
        let result=self.select_owned(offered,alchemy,conjuration,mana,clock);
        // A detached invocation relinquishes noncached keys. Live entries and stale
        // recency references retain their key identities in the canonical state.
        if result.is_err(){self.valid=false;return result;}
        match Machine::new(self.cache.snapshot()){
            Ok(cache)=>self.cache=cache,
            Err(error)=>{self.valid=false;return Err(error);}
        }
        result
    }
    fn select_owned<F>(&mut self,offered:Option<Stack>,alchemy:bool,conjuration:bool,mana:i32,clock:&mut F)->Result<Selection>
    where F:FnMut()->Result<i64>{
        let Some(mut owned)=offered else{return Ok(Selection{result:SelectionResult::NotMatched,offered_after:None,visited:0});};
        if let Some(nbt)=&owned.nbt{ensure!(matches!(nbt,Nbt::Compound{..}),"Native pool offered NBT is not a compound");nbt.validate()?;}
        let mut visited=0;let mut predicates=0;let mut result=SelectionResult::NotMatched;
        // Genome and Mate are distinct owned roots. Repeated callbacks on this
        // invocation reuse their key handles, even when repair changes their hash.
        let mut handles=[None,None];
        for index in 0..self.entries.len(){
            visited+=1;let entry=self.entries[index].clone();
            let (accepted,needs_alchemy,needs_conjuration,cost,output)=match entry {
                Entry::Bee{alchemy,conjuration,mana,form,input_species,output_species,..}=>{
                    let registry=self.source.forms.get(&form).ok_or_else(||anyhow::anyhow!("Unknown native bee form"))?;
                    if registry!=&owned.registry{continue;}
                    let mut reads=vec![];let mut failed=false;
                    if let Some(Nbt::Compound{value})=&mut owned.nbt {
                        for (role,name) in ["Genome","Mate"].into_iter().enumerate(){
                            let Some(tag)=value.get_mut(name) else{continue;};
                            let stored=matches!(tag,Nbt::Compound{..});
                            let mut empty=Nbt::Compound{value:Default::default()};
                            let query=if stored{&mut *tag}else{&mut empty};
                            let id=if let Some(id)=handles[role].filter(|_|stored){id}else{
                                let id=self.cache.bind_owned(query.clone())?;
                                if stored{handles[role]=Some(id);}id
                            };
                            let source=&self.source;
                            let outcome=self.cache.get(id,clock,&mut |tag|source.load_genome(tag))?;
                            if stored{*tag=self.cache.key(id)?.clone();}
                            match outcome{
                                Read::NativeError=>{failed=true;break;}
                                Read::Hit{chromosome}|Read::Loaded{chromosome}=>reads.push(GenomeRead::Hit{chromosome}),
                            }
                        }
                    }
                    if failed{return Ok(Selection{result:SelectionResult::NativeError{index:index as u32},offered_after:Some(owned),visited});}
                    let transition=self.source.transition(&form,&input_species,&output_species,Some(owned),reads)?;
                    owned=transition.offered_after.ok_or_else(||anyhow::anyhow!("Native pool lost its owned offered stack"))?;
                    match transition.result{
                        Outcome::NativeError=>return Ok(Selection{result:SelectionResult::NativeError{index:index as u32},offered_after:Some(owned),visited}),
                        Outcome::NotMatched=>(false,alchemy,conjuration,mana,None),
                        Outcome::Matched{output}=>{
                            self.cached[index]=Some(output);(true,alchemy,conjuration,mana,None)
                        }
                    }
                }
                Entry::Item{alchemy,conjuration,mana,predicates:patterns,output}=>{
                    let mut accepted=false;
                    for predicate in patterns{
                        predicates+=1;ensure!(predicates<=65536,"Native pool predicate budget exceeded");
                        if predicate.registry==owned.registry&&(predicate.metadata==32767||predicate.metadata==owned.meta){accepted=true;break;}
                    }
                    (accepted,alchemy,conjuration,mana,Some(output))
                }
                Entry::Mini{alchemy,conjuration,mana,key,output,coercions}=>{
                    let tag=if let Some(Nbt::Compound{value})=&owned.nbt{value.get("type")}else{None};
                    let observed=match tag{None=>"",Some(Nbt::String{value})=>value,
                        _=>{
                            let mut observations=coercions.iter().filter(|row|row.input.registry==owned.registry
                                &&row.input.meta==owned.meta&&serde_json::to_value(&row.input.nbt).ok()==serde_json::to_value(&owned.nbt).ok());
                            let first=observations.next().ok_or_else(||anyhow::anyhow!("Native pool session lacks a source-bound subtile observation"))?;
                            ensure!(observations.all(|row|row.value==first.value),"Conflicting native pool subtile observations");
                            &first.value
                        }};
                    (key==observed,alchemy,conjuration,mana,Some(output))
                }
            };
            if !accepted||needs_alchemy&&!alchemy||needs_conjuration&&!conjuration{continue;}
            result=if mana<cost{SelectionResult::InsufficientMana{index:index as u32}}else{
                SelectionResult::Selected{index:index as u32,output:match output{Some(output)=>output,None=>self.take_output(index)?}}
            };
            break;
        }
        Ok(Selection{result,offered_after:Some(owned),visited})
    }
}
