//! Owned single-instant native cache lookup. This precursor is not yet a
//! published program or pool selector; it does not apply access/load/eviction writes.
use crate::identity::Nbt;
use anyhow::{ensure,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};

#[derive(Clone,Debug,Deserialize,Serialize,JsonSchema)]
#[schemars(rename="BeePoolCachedAllele")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Allele {pub uid:String,pub dominant:bool,pub bee_species:bool,pub source_identity:bool}
#[derive(Clone,Debug,Deserialize,Serialize,JsonSchema)]
#[schemars(rename="BeePoolCachedChromosome")]
#[serde(deny_unknown_fields)]
pub struct Chromosome {
    #[serde(deserialize_with="super::nei_information::required_nullable")]
    #[schemars(with="super::nei_information::RequiredNullable<Allele>")]pub primary:Option<Allele>,
    #[serde(deserialize_with="super::nei_information::required_nullable")]
    #[schemars(with="super::nei_information::RequiredNullable<Allele>")]pub secondary:Option<Allele>,
}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Entry {pub hash:i32,pub key:Nbt,pub access_nanos:String,pub chromosome:Option<Chromosome>}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {pub count:u32,pub buckets:Vec<Vec<Entry>>}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {pub shift:u32,pub mask:u32,pub segments:Vec<Segment>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="lowercase")]
pub enum Read {Miss,Hit {chromosome:Option<Chromosome>}}

fn nanos(value:&str)->Result<i64>{
    let parsed=value.parse::<i64>()?;
    ensure!(parsed.to_string()==value,"Invalid native pool cache nanos");Ok(parsed)
}
fn rehash(mut hash:u32)->u32 {
    hash=hash.wrapping_add((hash<<15)^0xffffcd7d);hash^=hash>>10;
    hash=hash.wrapping_add(hash<<3);hash^=hash>>6;
    hash=hash.wrapping_add(hash<<2).wrapping_add(hash<<14);hash^(hash>>16)
}
impl Snapshot {
    pub fn validate(&self)->Result<()> {
        let size=self.segments.len();
        ensure!(size.is_power_of_two()&&size<=128&&self.mask==size as u32-1
            &&self.shift==32-size.trailing_zeros(),"Invalid native pool cache routing");
        let mut total=0usize;
        for (segment_at,segment) in self.segments.iter().enumerate() {
            let length=segment.buckets.len();
            ensure!(length.is_power_of_two()&&length<=65536&&segment.count<=4096,"Invalid native pool cache table");
            let mut count=0usize;
            for (slot,bucket) in segment.buckets.iter().enumerate() {
                for entry in bucket {
                    total+=1;count+=1;ensure!(total<=4096,"Native pool cache exceeds its entry budget");
                    ensure!(((entry.hash as u32).wrapping_shr(self.shift)&self.mask) as usize==segment_at
                        &&entry.hash as u32 as usize&(length-1)==slot,"Invalid native pool cache entry routing");
                    ensure!(matches!(entry.key,Nbt::Compound{..}),"Native pool cache key is not a compound");
                    entry.key.validate()?;nanos(&entry.access_nanos)?;
                    if let Some(cell)=&entry.chromosome {
                        for allele in [&cell.primary,&cell.secondary].into_iter().flatten() {
                            ensure!(!allele.uid.is_empty()&&allele.uid.encode_utf16().count()<=16384,"Invalid native pool cached allele");
                        }
                    }
                }
            }
            ensure!(count==segment.count as usize,"Invalid native pool cache count");
        }
        ensure!(serde_json::to_vec(self)?.len()<=16*1024*1024,"Native pool cache exceeds its byte budget");
        Ok(())
    }
    pub fn lookup(&self,query:&Nbt,now:i64)->Result<Read> {
        self.validate()?;ensure!(matches!(query,Nbt::Compound{..}),"Native pool query is not a compound");
        let hash=rehash(query.native_hash()? as u32);
        let segment=&self.segments[(hash.wrapping_shr(self.shift)&self.mask) as usize];
        if segment.count==0 {return Ok(Read::Miss);}
        for entry in &segment.buckets[hash as usize&(segment.buckets.len()-1)] {
            if entry.hash as u32==hash&&query.native_equal(&entry.key)? {
                if now.wrapping_sub(nanos(&entry.access_nanos)?)>=60_000_000_000 {return Ok(Read::Miss);}
                return Ok(Read::Hit{chromosome:entry.chromosome.clone()});
            }
        }
        Ok(Read::Miss)
    }
}
