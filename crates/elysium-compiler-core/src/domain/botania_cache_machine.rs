//! Owned serial Guava-17 transitions. This kernel requests every ticker read
//! explicitly; runtime/clock admission and the pool program are separate.
use super::botania_cache::Chromosome;
use crate::identity::Nbt;
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(rename="BeePoolCacheEntry")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    pub hash: i32,
    #[schemars(with="u32")]
    pub key: usize,
    pub access_nanos: String,
    #[schemars(with="super::nei_information::RequiredNullable<Chromosome>")]
    #[serde(deserialize_with="super::nei_information::required_nullable")]
    pub chromosome: Option<Chromosome>,
}
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(rename="BeePoolCacheSegment")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Segment {
    pub count: u32,
    pub threshold: u32,
    pub max_weight: String,
    pub total_weight: u32,
    pub read_count: i32,
    #[schemars(with="Vec<Vec<u32>>")]
    pub buckets: Vec<Vec<usize>>,
    #[schemars(with="Vec<u32>")]
    pub access: Vec<usize>,
    #[schemars(with="Vec<u32>")]
    pub recency: Vec<usize>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub shift: u32,
    pub mask: u32,
    pub keys: Vec<Nbt>,
    pub entries: Vec<Entry>,
    pub segments: Vec<Segment>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Read {
    NativeError,
    Hit { chromosome: Option<Chromosome> },
    Loaded { chromosome: Option<Chromosome> },
}
pub enum Load {
    NativeError,
    Loaded(Option<Chromosome>),
}
#[derive(Clone)]
struct Node {
    hash: i32,
    key: usize,
    access: i64,
    chromosome: Option<Chromosome>,
    loading: Option<u64>,
}
struct Route {
    count: u32,
    threshold: u32,
    max_weight: u32,
    reads: i32,
    buckets: Vec<Vec<usize>>,
    access: Vec<usize>,
    recency: Vec<usize>,
}
pub struct Machine {
    shift: u32,
    mask: u32,
    keys: Vec<Nbt>,
    entries: Vec<Node>,
    segments: Vec<Route>,
    load_id: u64,
}
const EXPIRE: i64 = 60_000_000_000;
fn nanos(value: &str) -> Result<i64> {
    let parsed = value.parse::<i64>()?;
    ensure!(parsed.to_string() == value, "Invalid native cache nanos");
    Ok(parsed)
}
fn rehash(mut hash: u32) -> u32 {
    hash = hash.wrapping_add((hash << 15) ^ 0xffffcd7d);
    hash ^= hash >> 10;
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 6;
    hash = hash.wrapping_add(hash << 2).wrapping_add(hash << 14);
    hash ^ (hash >> 16)
}
fn last(queue: &mut Vec<usize>, id: usize) {
    if let Some(at) = queue.iter().position(|&entry| entry == id) {
        queue.remove(at);
    }
    queue.push(id);
}
impl State {
    pub fn validate(&self) -> Result<()> {
        let size = self.segments.len();
        ensure!(size.is_power_of_two() && size <= 128 && self.mask == size as u32 - 1
            && self.shift == 32 - size.trailing_zeros(), "Invalid native cache transition routing");
        ensure!(self.keys.len() <= 4096 && self.entries.len() <= 4096
            && serde_json::to_vec(self)?.len() <= 16 * 1024 * 1024, "Native cache transition budget exceeded");
        for key in &self.keys {
            ensure!(matches!(key, Nbt::Compound { .. }), "Native cache key is not compound");
            key.validate()?;
        }
        for entry in &self.entries {
            ensure!(entry.key < self.keys.len(), "Invalid native cache key reference");
            nanos(&entry.access_nanos)?;
            if let Some(cell) = &entry.chromosome {
                for allele in [&cell.primary, &cell.secondary].into_iter().flatten() {
                    ensure!(!allele.uid.is_empty() && allele.uid.encode_utf16().count() <= 16384,
                        "Invalid native cached allele");
                }
            }
        }
        let mut live = BTreeSet::new();
        let mut max_weight = 0i64;
        for (i, route) in self.segments.iter().enumerate() {
            let length = route.buckets.len();
            let max = nanos(&route.max_weight)?;
            ensure!(length.is_power_of_two() && length <= 65536 && route.count <= 4096
                && route.threshold as usize <= length && max >= 0 && max <= 128
                && route.count as i64 <= max && route.total_weight == route.count, "Invalid native cache segment");
            max_weight += max;
            let mut own = BTreeSet::new();
            for (slot, bucket) in route.buckets.iter().enumerate() {
                for &id in bucket {
                    let entry = self.entries.get(id).ok_or_else(|| anyhow::anyhow!("Invalid native cache entry reference"))?;
                    ensure!(live.insert(id) && own.insert(id)
                        && ((entry.hash as u32).wrapping_shr(self.shift) & self.mask) as usize == i
                        && (entry.hash as u32 as usize & (length - 1)) == slot, "Invalid native cache entry routing");
                }
            }
            ensure!(own.len() == route.count as usize && route.access.len() == own.len()
                && route.access.iter().copied().collect::<BTreeSet<_>>() == own
                && route.recency.len() <= 65536, "Invalid native cache queues or counts");
            for &id in &route.recency {
                let entry = self.entries.get(id).ok_or_else(|| anyhow::anyhow!("Invalid native recency reference"))?;
                ensure!(((entry.hash as u32).wrapping_shr(self.shift) & self.mask) as usize == i, "Invalid native recency routing");
            }
        }
        ensure!(max_weight == 128, "Native cache segment weights do not sum to 128");
        Ok(())
    }
}
impl Machine {
    pub fn new(state: State) -> Result<Self> {
        state.validate()?;
        Ok(Self {
            shift: state.shift, mask: state.mask, keys: state.keys, load_id: 0,
            entries: state.entries.into_iter().map(|v| Ok(Node {
                hash: v.hash, key: v.key, access: nanos(&v.access_nanos)?,
                chromosome: v.chromosome, loading: None,
            })).collect::<Result<_>>()?,
            segments: state.segments.into_iter().map(|v| Ok(Route {
                count: v.count, threshold: v.threshold, max_weight: nanos(&v.max_weight)? as u32,
                reads: v.read_count, buckets: v.buckets, access: v.access, recency: v.recency,
            })).collect::<Result<_>>()?,
        })
    }
    pub fn key(&self, id: usize) -> Result<&Nbt> {
        self.keys.get(id).ok_or_else(|| anyhow::anyhow!("Invalid native cache query key"))
    }
    pub(crate) fn bind_owned(&mut self, key: Nbt) -> Result<usize> {
        ensure!(matches!(key,Nbt::Compound{..}) && self.keys.len()<4096,"Invalid native owned cache key or key budget exceeded");
        key.validate()?;
        let id=self.keys.len();self.keys.push(key);Ok(id)
    }
    pub fn get<F, L>(&mut self, key: usize, clock: &mut F, loader: &mut L) -> Result<Read>
    where F: FnMut() -> Result<i64>, L: FnMut(&mut Nbt) -> Result<Load> {
        let hash = rehash(self.key(key)?.native_hash()? as u32) as i32;
        let segment = ((hash as u32).wrapping_shr(self.shift) & self.mask) as usize;
        let result = self.read(segment, key, hash, clock, loader);
        self.segments[segment].reads = self.segments[segment].reads.wrapping_add(1);
        if self.segments[segment].reads & 63 == 0 {
            self.cleanup(segment, clock()?)?;
        }
        self.compact();
        result
    }
    fn read<F, L>(&mut self, segment: usize, key: usize, hash: i32, clock: &mut F, loader: &mut L) -> Result<Read>
    where F: FnMut() -> Result<i64>, L: FnMut(&mut Nbt) -> Result<Load> {
        if self.segments[segment].count != 0 {
            if let Some(id) = self.find(segment, hash, key)? {
                let now = clock()?;
                if now.wrapping_sub(self.entries[id].access) < EXPIRE {
                    self.entries[id].access = now;
                    self.segments[segment].recency.push(id);
                    return Ok(Read::Hit { chromosome: self.entries[id].chromosome.clone() });
                }
                self.expire(segment, now)?;
            }
        }
        let now = clock()?;
        self.cleanup(segment, now)?;
        let placeholder = if let Some(id) = self.find(segment, hash, key)? {
            if now.wrapping_sub(self.entries[id].access) >= EXPIRE {
                self.segments[segment].access.retain(|&entry| entry != id);
                self.segments[segment].count -= 1;
                id
            } else {
                self.entries[id].access = now;
                last(&mut self.segments[segment].access, id);
                return Ok(Read::Hit { chromosome: self.entries[id].chromosome.clone() });
            }
        } else { self.insert(segment, hash, key) };
        self.load_id += 1;
        let loading = self.load_id;
        self.entries[placeholder].loading = Some(loading);
        let loaded = loader(&mut self.keys[key])?;
        self.keys[key].validate()?;
        let Load::Loaded(chromosome) = loaded else {
            if let Some(id) = self.find(segment, hash, key)? {
                if self.entries[id].loading == Some(loading) { self.remove(segment, id, false)?; }
            }
            return Ok(Read::NativeError);
        };
        let write = clock()?;
        self.cleanup(segment, write)?;
        if self.segments[segment].count + 1 > self.segments[segment].threshold { self.expand(segment)?; }
        let id = if let Some(id) = self.find(segment, hash, key)? {
            ensure!(self.entries[id].loading == Some(loading),
                "Native mutable-key collision retained an unadapted loading reference");
            id
        } else { self.insert(segment, hash, key) };
        self.entries[id].loading = None;
        self.entries[id].chromosome = chromosome.clone();
        self.drain(segment);
        self.entries[id].access = write;
        last(&mut self.segments[segment].access, id);
        self.segments[segment].count += 1;
        self.drain(segment);
        while self.segments[segment].count > self.segments[segment].max_weight {
            let id = self.segments[segment].access[0];
            self.remove(segment, id, true)?;
        }
        Ok(Read::Loaded { chromosome })
    }
    fn find(&self, segment: usize, hash: i32, key: usize) -> Result<Option<usize>> {
        let buckets = &self.segments[segment].buckets;
        for &id in &buckets[hash as u32 as usize & (buckets.len() - 1)] {
            let entry = &self.entries[id];
            if entry.hash == hash && (entry.key == key || self.keys[key].native_equal(&self.keys[entry.key])?) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }
    fn insert(&mut self, segment: usize, hash: i32, key: usize) -> usize {
        let id = self.entries.len();
        self.entries.push(Node { hash, key, access: i64::MAX, chromosome: None, loading: None });
        let route = &mut self.segments[segment];
        let slot = hash as u32 as usize & (route.buckets.len() - 1);
        route.buckets[slot].insert(0, id);
        id
    }
    fn cleanup(&mut self, segment: usize, now: i64) -> Result<()> {
        self.expire(segment, now)?;
        self.segments[segment].reads = 0;
        Ok(())
    }
    fn expire(&mut self, segment: usize, now: i64) -> Result<()> {
        self.drain(segment);
        while let Some(&id) = self.segments[segment].access.first() {
            if now.wrapping_sub(self.entries[id].access) < EXPIRE { break; }
            self.remove(segment, id, true)?;
        }
        Ok(())
    }
    fn drain(&mut self, segment: usize) {
        let route = &mut self.segments[segment];
        for id in std::mem::take(&mut route.recency) {
            if route.access.contains(&id) { last(&mut route.access, id); }
        }
    }
    fn copy_entry(&mut self, segment: usize, old: usize) -> usize {
        let id = self.entries.len();
        self.entries.push(self.entries[old].clone());
        if let Some(at) = self.segments[segment].access.iter().position(|&entry| entry == old) {
            self.segments[segment].access[at] = id;
        }
        id
    }
    fn remove(&mut self, segment: usize, id: usize, active: bool) -> Result<()> {
        let slot = self.entries[id].hash as u32 as usize & (self.segments[segment].buckets.len() - 1);
        let bucket = self.segments[segment].buckets[slot].clone();
        let at = bucket.iter().position(|&entry| entry == id).ok_or_else(|| anyhow::anyhow!("Missing native entry during removal"))?;
        let mut replacement = bucket[at + 1..].to_vec();
        for &old in &bucket[..at] { replacement.insert(0, self.copy_entry(segment, old)); }
        self.segments[segment].buckets[slot] = replacement;
        self.segments[segment].access.retain(|&entry| entry != id);
        if active { self.segments[segment].count -= 1; }
        Ok(())
    }
    fn expand(&mut self, segment: usize) -> Result<()> {
        let length = self.segments[segment].buckets.len();
        ensure!(length < 65536, "Native cache expansion exceeds table budget");
        let mut grown = vec![Vec::new(); length * 2];
        let mask = length * 2 - 1;
        for bucket in self.segments[segment].buckets.clone() {
            if bucket.is_empty() { continue; }
            let mut last_run = 0;
            let mut route = self.entries[bucket[0]].hash as u32 as usize & mask;
            for (i, &id) in bucket.iter().enumerate().skip(1) {
                let next = self.entries[id].hash as u32 as usize & mask;
                if next != route { last_run = i; route = next; }
            }
            grown[route].extend_from_slice(&bucket[last_run..]);
            for &old in &bucket[..last_run] {
                let id = self.copy_entry(segment, old);
                grown[self.entries[id].hash as u32 as usize & mask].insert(0, id);
            }
        }
        self.segments[segment].buckets = grown;
        self.segments[segment].threshold = (length * 2 * 3 / 4) as u32;
        Ok(())
    }
    // Native entry copies become unreachable except for queued stale references.
    // Keep that identity graph, rather than retaining every historical arena slot.
    fn compact(&mut self) {
        let mut ids = BTreeMap::new();
        let mut nodes = Vec::new();
        for route in &mut self.segments {
            for id in route.buckets.iter_mut().flatten().chain(route.access.iter_mut()).chain(route.recency.iter_mut()) {
                *id = *ids.entry(*id).or_insert_with(|| {
                    let next = nodes.len(); nodes.push(self.entries[*id].clone()); next
                });
            }
        }
        self.entries = nodes;
    }
    pub fn snapshot(&self) -> State {
        let mut keys = vec![];
        let mut entries = vec![];
        let mut entry_ids = BTreeMap::new();
        let mut key_ids = BTreeMap::new();
        let mut encode = |id: usize| {
            *entry_ids.entry(id).or_insert_with(|| {
                let entry = &self.entries[id];
                let key = *key_ids.entry(entry.key).or_insert_with(|| {
                    let id = keys.len(); keys.push(self.keys[entry.key].clone()); id
                });
                let id = entries.len();
                entries.push(Entry { hash: entry.hash, key, access_nanos: entry.access.to_string(), chromosome: entry.chromosome.clone() });
                id
            })
        };
        let segments = self.segments.iter().map(|v| Segment {
            count: v.count, threshold: v.threshold, max_weight: v.max_weight.to_string(), total_weight: v.count,
            read_count: v.reads, buckets: v.buckets.iter().map(|b| b.iter().map(|&id| encode(id)).collect()).collect(),
            access: v.access.iter().map(|&id| encode(id)).collect(), recency: v.recency.iter().map(|&id| encode(id)).collect(),
        }).collect();
        State { shift: self.shift, mask: self.mask, keys, entries, segments }
    }
}
