//! Shared, bounded machine rule chunks. Whole-program identity retains native order.
use super::forestry::{
    FilledContainer, SqueezerCallback, SqueezerContainer, SqueezerProgram, SqueezerRecipe,
};
use super::fabricator::{FabricatorProgram, FabricatorRecipe, FabricatorSmeltingRecipe};
use super::forestry_processing::{ForestryProcessingProgram, FermenterRecipe, FermenterFuel, MoistenerRecipe, MoistenerFuel, CarpenterRecipe};
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProgramChunk {
    pub id: String,
    pub program: String,
    pub offset: u32,
    pub data: ProgramData,
}

impl ProgramChunk {
    pub fn content_id(&self) -> Result<String> {
        Ok(format!(
            "{}.{}",
            self.program,
            super::content_id("chunk", self)?
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "kind",
    content = "rows",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum ProgramData {
    BeePoolSettings(Vec<super::botania_program::Settings>),
    BeePoolChromosomes(Vec<String>),
    BeePoolDefinitions(Vec<super::botania_program::Definition>),
    BeePoolAlleles(Vec<super::botania_bee::Registration>),
    BeePoolAliases(Vec<super::botania_program::Alias>),
    BeePoolTemplates(Vec<super::botania_program::Template>),
    BeePoolDefaults(Vec<Option<String>>),
    BeePoolForest(Vec<Option<super::botania_bee::Pair>>),
    BeePoolForms(Vec<super::botania_program::Form>),
    BeePoolEntries(Vec<super::botania_session::Entry>),
    BeePoolKeys(Vec<crate::identity::Nbt>),
    BeePoolCacheEntries(Vec<super::botania_cache_machine::Entry>),
    BeePoolSegments(Vec<super::botania_cache_machine::Segment>),
    XuMaterials(Vec<super::xu_microblock::Material>),
    XuNames(Vec<super::fmp_program::Name>),
    XuAliases(Vec<super::fmp_program::Alias>),
    XuSettings(Vec<super::xu_microblock::Settings>),
    FmpMaterials(Vec<super::fmp_program::Material>),
    FmpNames(Vec<super::fmp_program::Name>),
    FmpAliases(Vec<super::fmp_program::Alias>),
    FmpSaws(Vec<super::fmp::Saw>),
    FmpSettings(Vec<super::fmp_program::Settings>),
    ForestryFermenter(Vec<FermenterRecipe>),
    ForestryFermenterFuels(Vec<FermenterFuel>),
    ForestryMoistener(Vec<MoistenerRecipe>),
    ForestryMoistenerFuels(Vec<MoistenerFuel>),
    ForestryCarpenter(Vec<CarpenterRecipe>),
    FabricatorRecipes(Vec<FabricatorRecipe>),
    FabricatorSmelting(Vec<FabricatorSmeltingRecipe>),
    SqueezerRecipes(Vec<SqueezerRecipe>),
    SqueezerContainers(Vec<SqueezerContainer>),
    SqueezerFluids(Vec<FilledContainer>),
    SqueezerCallbacks(Vec<SqueezerCallback>),
}

impl ProgramData {
    fn section(&self) -> usize {
        match self {
            Self::BeePoolSettings(_)=>0,Self::BeePoolChromosomes(_)=>1,Self::BeePoolDefinitions(_)=>2,
            Self::BeePoolAlleles(_)=>3,Self::BeePoolAliases(_)=>4,Self::BeePoolTemplates(_)=>5,
            Self::BeePoolDefaults(_)=>6,Self::BeePoolForest(_)=>7,Self::BeePoolForms(_)=>8,
            Self::BeePoolEntries(_)=>9,Self::BeePoolKeys(_)=>10,Self::BeePoolCacheEntries(_)=>11,Self::BeePoolSegments(_)=>12,
            Self::XuMaterials(_)=>0,Self::XuNames(_)=>1,Self::XuAliases(_)=>2,Self::XuSettings(_)=>3,
            Self::FmpMaterials(_)=>0,Self::FmpNames(_)=>1,Self::FmpAliases(_)=>2,Self::FmpSaws(_)=>3,Self::FmpSettings(_)=>4,
            Self::ForestryFermenter(_) => 0,
            Self::ForestryFermenterFuels(_) => 1,
            Self::ForestryMoistener(_) => 2,
            Self::ForestryMoistenerFuels(_) => 3,
            Self::ForestryCarpenter(_) => 4,
            Self::FabricatorRecipes(_) => 0,
            Self::FabricatorSmelting(_) => 1,
            Self::SqueezerRecipes(_) => 0,
            Self::SqueezerContainers(_) => 1,
            Self::SqueezerFluids(_) => 2,
            Self::SqueezerCallbacks(_) => 3,
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::BeePoolSettings(v)=>v.len(),Self::BeePoolChromosomes(v)=>v.len(),Self::BeePoolDefinitions(v)=>v.len(),
            Self::BeePoolAlleles(v)=>v.len(),Self::BeePoolAliases(v)=>v.len(),Self::BeePoolTemplates(v)=>v.len(),
            Self::BeePoolDefaults(v)=>v.len(),Self::BeePoolForest(v)=>v.len(),Self::BeePoolForms(v)=>v.len(),
            Self::BeePoolEntries(v)=>v.len(),Self::BeePoolKeys(v)=>v.len(),Self::BeePoolCacheEntries(v)=>v.len(),Self::BeePoolSegments(v)=>v.len(),
            Self::XuMaterials(v)=>v.len(),Self::XuNames(v)=>v.len(),Self::XuAliases(v)=>v.len(),Self::XuSettings(v)=>v.len(),
            Self::FmpMaterials(v)=>v.len(),Self::FmpNames(v)=>v.len(),Self::FmpAliases(v)=>v.len(),Self::FmpSaws(v)=>v.len(),Self::FmpSettings(v)=>v.len(),
            Self::ForestryFermenter(v) => v.len(),
            Self::ForestryFermenterFuels(v) => v.len(),
            Self::ForestryMoistener(v) => v.len(),
            Self::ForestryMoistenerFuels(v) => v.len(),
            Self::ForestryCarpenter(v) => v.len(),
            Self::FabricatorRecipes(v) => v.len(),
            Self::FabricatorSmelting(v) => v.len(),
            Self::SqueezerRecipes(v) => v.len(),
            Self::SqueezerContainers(v) => v.len(),
            Self::SqueezerFluids(v) => v.len(),
            Self::SqueezerCallbacks(v) => v.len(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum SharedProgram { Squeezer(SqueezerProgram), Fabricator(FabricatorProgram), Processing(ForestryProcessingProgram), Fmp(super::fmp_program::Program), Xu(super::xu_microblock::Program), BeePool(super::botania_program::Program) }
impl SharedProgram {
    fn empty(data: &ProgramData) -> Self {
        match data {
            ProgramData::BeePoolSettings(_)|ProgramData::BeePoolChromosomes(_)|ProgramData::BeePoolDefinitions(_)
            |ProgramData::BeePoolAlleles(_)|ProgramData::BeePoolAliases(_)|ProgramData::BeePoolTemplates(_)
            |ProgramData::BeePoolDefaults(_)|ProgramData::BeePoolForest(_)|ProgramData::BeePoolForms(_)
            |ProgramData::BeePoolEntries(_)|ProgramData::BeePoolKeys(_)|ProgramData::BeePoolCacheEntries(_)
            |ProgramData::BeePoolSegments(_)=>Self::BeePool(super::botania_program::Program::default()),
            ProgramData::XuMaterials(_)|ProgramData::XuNames(_)|ProgramData::XuAliases(_)|ProgramData::XuSettings(_)=>Self::Xu(super::xu_microblock::Program::default()),
            ProgramData::FmpMaterials(_)|ProgramData::FmpNames(_)|ProgramData::FmpAliases(_)|ProgramData::FmpSaws(_)|ProgramData::FmpSettings(_)=>Self::Fmp(super::fmp_program::Program::default()),
            ProgramData::ForestryFermenter(_) | ProgramData::ForestryFermenterFuels(_)
            | ProgramData::ForestryMoistener(_) | ProgramData::ForestryMoistenerFuels(_)
            | ProgramData::ForestryCarpenter(_) => Self::Processing(ForestryProcessingProgram::default()),
            ProgramData::FabricatorRecipes(_) | ProgramData::FabricatorSmelting(_) =>
                Self::Fabricator(FabricatorProgram {crafting:vec![],smelting:vec![]}),
            _ => Self::Squeezer(SqueezerProgram {ordinary:vec![],containers:vec![],filled:vec![],dynamic:vec![]}),
        }
    }
    fn section_count(&self) -> usize {match self { Self::Squeezer(_)=>4, Self::Fabricator(_)=>2, Self::Processing(_)=>5,Self::Fmp(_)=>5,Self::Xu(_)=>4,Self::BeePool(_)=>13 }}
    fn append(&mut self, data: &ProgramData) -> Result<()> {
        match (self,data) {
            (Self::BeePool(p),ProgramData::BeePoolSettings(v))=>p.settings.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolChromosomes(v))=>p.chromosomes.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolDefinitions(v))=>p.definitions.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolAlleles(v))=>p.alleles.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolAliases(v))=>p.aliases.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolTemplates(v))=>p.templates.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolDefaults(v))=>p.defaults.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolForest(v))=>p.forest.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolForms(v))=>p.forms.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolEntries(v))=>p.entries.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolKeys(v))=>p.keys.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolCacheEntries(v))=>p.cache_entries.extend_from_slice(v),
            (Self::BeePool(p),ProgramData::BeePoolSegments(v))=>p.segments.extend_from_slice(v),
            (Self::Xu(p),ProgramData::XuMaterials(v))=>p.materials.extend_from_slice(v),
            (Self::Xu(p),ProgramData::XuNames(v))=>p.names.extend_from_slice(v),
            (Self::Xu(p),ProgramData::XuAliases(v))=>p.aliases.extend_from_slice(v),
            (Self::Xu(p),ProgramData::XuSettings(v))=>p.settings.extend_from_slice(v),
            (Self::Fmp(p),ProgramData::FmpMaterials(v))=>p.materials.extend_from_slice(v),
            (Self::Fmp(p),ProgramData::FmpNames(v))=>p.names.extend_from_slice(v),
            (Self::Fmp(p),ProgramData::FmpAliases(v))=>p.aliases.extend_from_slice(v),
            (Self::Fmp(p),ProgramData::FmpSaws(v))=>p.saws.extend_from_slice(v),
            (Self::Fmp(p),ProgramData::FmpSettings(v))=>p.settings.extend_from_slice(v),
            (Self::Processing(p),ProgramData::ForestryFermenter(r)) => p.fermenter.extend_from_slice(r),
            (Self::Processing(p),ProgramData::ForestryFermenterFuels(r)) => p.fermenter_fuels.extend_from_slice(r),
            (Self::Processing(p),ProgramData::ForestryMoistener(r)) => p.moistener.extend_from_slice(r),
            (Self::Processing(p),ProgramData::ForestryMoistenerFuels(r)) => p.moistener_fuels.extend_from_slice(r),
            (Self::Processing(p),ProgramData::ForestryCarpenter(r)) => p.carpenter.extend_from_slice(r),
            (Self::Squeezer(p),ProgramData::SqueezerRecipes(r)) => p.ordinary.extend_from_slice(r),
            (Self::Squeezer(p),ProgramData::SqueezerContainers(r)) => p.containers.extend_from_slice(r),
            (Self::Squeezer(p),ProgramData::SqueezerFluids(r)) => p.filled.extend_from_slice(r),
            (Self::Squeezer(p),ProgramData::SqueezerCallbacks(r)) => p.dynamic.extend_from_slice(r),
            (Self::Fabricator(p),ProgramData::FabricatorRecipes(r)) => p.crafting.extend_from_slice(r),
            (Self::Fabricator(p),ProgramData::FabricatorSmelting(r)) => p.smelting.extend_from_slice(r),
            _ => anyhow::bail!("Shared program mixes different machine families"),
        }
        Ok(())
    }
    fn validate(&self) -> Result<()> {match self {Self::Squeezer(p)=>p.validate(),Self::Fabricator(p)=>p.validate(),Self::Processing(p)=>p.validate(),Self::Fmp(p)=>p.validate(),Self::Xu(p)=>p.validate(),Self::BeePool(p)=>p.validate()}}
    fn kind(&self) -> &'static str {match self{Self::Squeezer(_)=>"forestrySqueezer",Self::Fabricator(_)=>"forestryFabricator",Self::Processing(_)=>"forestryProcessing",Self::Fmp(_)=>"fmpMicroblock",Self::Xu(_)=>"xuMicroblock",Self::BeePool(_)=>"magicBeesPool"}}
}

pub fn reconstruct(chunks: &[ProgramChunk]) -> Result<BTreeMap<String, SharedProgram>> {
    let mut groups = BTreeMap::<&str, Vec<&ProgramChunk>>::new();
    let mut ids = BTreeSet::new();
    for chunk in chunks {
        ensure!(
            chunk
                .program
                .strip_prefix("program_")
                .is_some_and(crate::source::is_digest),
            "Invalid shared program identity"
        );
        ensure!(
            chunk.id == chunk.content_id()? && ids.insert(&chunk.id),
            "Invalid or duplicate program chunk"
        );
        ensure!(
            chunk.data.len() <= 128 && serde_json::to_vec(chunk)?.len() <= 1024 * 1024,
            "Program chunk exceeds the source row budget"
        );
        groups.entry(&chunk.program).or_default().push(chunk);
    }
    let mut result = BTreeMap::new();
    for (id, mut chunks) in groups {
        chunks.sort_by_key(|chunk| (chunk.data.section(), chunk.offset));
        let mut program = SharedProgram::empty(&chunks[0].data);
        let mut lengths = vec![0;program.section_count()];
        let mut seen = vec![false;program.section_count()];
        let mut empty = vec![false;program.section_count()];
        let mut bytes = 0;
        for chunk in chunks {
            let section = chunk.data.section();
            ensure!(section < lengths.len(),"Invalid section for shared program family");
            program.append(&chunk.data)?;
            ensure!(
                chunk.offset as usize == lengths[section] && !empty[section],
                "Program chunk has a gap or duplicate offset"
            );
            ensure!(
                chunk.data.len() > 0 || !seen[section],
                "Program section has an extra empty chunk"
            );
            empty[section] = chunk.data.len() == 0;
            seen[section] = true;
            lengths[section] += chunk.data.len();
            bytes += serde_json::to_vec(chunk)?.len();
            ensure!(
                bytes <= 32 * 1024 * 1024,
                "Shared program transport exceeds its budget"
            );
        }
        ensure!(
            seen.iter().all(|v| *v),
            "Shared program is missing a section"
        );
        program.validate()?;
        ensure!(
            serde_json::to_vec(&program)?.len() <= 16 * 1024 * 1024,
            "Shared rules exceed 16 MiB"
        );
        ensure!(
            super::content_id(
                "program",
                &serde_json::json!({"kind":program.kind(),"rules":program})
            )? == id,
            "Whole-program content hash mismatch"
        );
        result.insert(id.to_owned(), program);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_bee_pool_program_retains_order_source_and_cache_as_bounded_chunks() {
        let native:serde_json::Value=serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/magicbees-pool-session.json")).unwrap();
        let data=serde_json::json!({"kind":"beePoolEntries","rows":native["entries"]});
        let _:ProgramData=serde_json::from_value(data)
            .expect("Shared transport must retain stateful bee callbacks instead of dropping their program");
        let fixture:serde_json::Value=serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/magicbees-pool-program.json")).unwrap();
        let chunks:Vec<ProgramChunk>=serde_json::from_value(fixture["chunks"].clone()).unwrap();
        let id=fixture["program"].as_str().unwrap();
        let groups=reconstruct(&chunks).unwrap();
        assert_eq!(serde_json::to_value(&groups[id]).unwrap(),fixture["rules"],"Java whole-program identity changed during reconstruction");
        let SharedProgram::BeePool(program)=&groups[id] else{panic!("Wrong pool program family");};
        assert_eq!(serde_json::to_value(program.source().unwrap()).unwrap(),native["source"]);
        assert_eq!(serde_json::to_value(&program.entries).unwrap(),native["entries"]);
        assert_eq!(serde_json::to_value(program.cache().unwrap()).unwrap(),native["cache"]);
        let mut session=program.session().unwrap();
        for row in native["observations"].as_array().unwrap(){
            let mut ticks=row["ticks"].as_array().unwrap().iter();
            let actual=session.select(serde_json::from_value(row["input"].clone()).unwrap(),
                row["alchemy"].as_bool().unwrap(),row["conjuration"].as_bool().unwrap(),row["mana"].as_i64().unwrap() as i32,
                &mut ||Ok(ticks.next().ok_or_else(||anyhow::anyhow!("Extra native tick"))?.as_str().unwrap().parse()?)).unwrap();
            assert_eq!(serde_json::to_value(actual).unwrap(),row["selection"]);
            assert!(ticks.next().is_none());
        }
        for index in 0..chunks.len(){
            let mut missing=chunks.clone();missing.remove(index);
            assert!(reconstruct(&missing).is_err(),"Missing section/chunk {index} was accepted");
        }
        let mut duplicate=chunks.clone();duplicate.push(chunks[0].clone());
        assert!(reconstruct(&duplicate).is_err());
        let mut changed=chunks.clone();
        let chunk=changed.iter_mut().find(|v|matches!(v.data,ProgramData::BeePoolEntries(_))).unwrap();
        let ProgramData::BeePoolEntries(entries)=&mut chunk.data else{unreachable!()};
        let super::super::botania_session::Entry::Bee{mana,..}=&mut entries[0] else{panic!()};
        *mana+=1;chunk.id=chunk.content_id().unwrap();
        assert!(reconstruct(&changed).is_err(),"Resealed chunk bypassed the complete-program identity");
        let mut mixed=chunks.clone();mixed[0].data=ProgramData::SqueezerRecipes(vec![]);mixed[0].id=mixed[0].content_id().unwrap();
        assert!(reconstruct(&mixed).is_err());
        for (path,bad) in [
            ("/settings/0/runtime/artifactSha256",serde_json::json!("unreviewed")),
            ("/definitions/0/allele/slots",serde_json::json!([14])),
            ("/entries/0/inputSpecies",serde_json::json!("missing.species")),
            ("/segments/0/totalWeight",serde_json::json!(1)),
        ]{
            let mut rules=fixture["rules"].clone();*rules.pointer_mut(path).unwrap()=bad;
            let invalid:super::super::botania_program::Program=serde_json::from_value(rules).unwrap();
            assert!(invalid.validate().is_err(),"Native pool accepted invalid binding at {path}");
        }
        let mut missing=fixture["rules"].clone();
        missing["forest"][0].as_object_mut().unwrap().remove("primary");
        assert!(serde_json::from_value::<super::super::botania_program::Program>(missing).is_err(),
            "Missing native nullable observations are not equivalent to observed null");
    }
    #[test]
    fn forestry_processing_chunks_keep_five_ordered_sections_and_reject_family_changes() {
        let rules=serde_json::json!({"fermenter":[],"fermenterFuels":[],"moistener":[],"moistenerFuels":[],"carpenter":[]});
        let program=super::super::content_id("program",&serde_json::json!({"kind":"forestryProcessing","rules":rules})).unwrap();
        let mut chunks=Vec::new();
        for kind in ["forestryFermenter","forestryFermenterFuels","forestryMoistener","forestryMoistenerFuels","forestryCarpenter"] {
            let mut chunk:ProgramChunk=serde_json::from_value(serde_json::json!({"id":"pending","program":program,"offset":0,"data":{"kind":kind,"rows":[]}}))
                .expect("Typed Forestry processor transport is missing");
            chunk.id=chunk.content_id().unwrap();chunks.push(chunk);
        }
        let groups=reconstruct(&chunks).unwrap();
        assert_eq!(serde_json::to_value(&groups[&program]).unwrap(),rules);
        for skip in 0..5 {let mut missing=chunks.clone();missing.remove(skip);assert!(reconstruct(&missing).is_err());}
        let mut mixed=chunks.clone();mixed[1].data=ProgramData::SqueezerRecipes(vec![]);mixed[1].id=mixed[1].content_id().unwrap();
        assert!(reconstruct(&mixed).is_err());
        chunks[4].offset=1;chunks[4].id=chunks[4].content_id().unwrap();assert!(reconstruct(&chunks).is_err());
    }
    #[test]
    fn fabricator_chunks_require_both_sections_and_reject_mixed_programs() {
        let rules=serde_json::json!({"crafting":[],"smelting":[]});
        let program=super::super::content_id("program",&serde_json::json!({"kind":"forestryFabricator","rules":rules})).unwrap();
        let mut chunks=Vec::new();
        for kind in ["fabricatorRecipes","fabricatorSmelting"] {
            let mut chunk:ProgramChunk=serde_json::from_value(serde_json::json!({"id":"pending","program":program,"offset":0,"data":{"kind":kind,"rows":[]}}))
                .expect("Typed Fabricator transport is missing");
            chunk.id=chunk.content_id().unwrap();chunks.push(chunk);
        }
        let groups=reconstruct(&chunks).unwrap();
        assert_eq!(serde_json::to_value(&groups[&program]).unwrap(),rules);
        assert!(reconstruct(&chunks[..1]).is_err());
        let mut mixed=chunks.clone();mixed[1].data=ProgramData::SqueezerRecipes(vec![]);mixed[1].id=mixed[1].content_id().unwrap();
        assert!(reconstruct(&mixed).is_err(), "Program cannot mix machine families");
        chunks[1].offset=1;chunks[1].id=chunks[1].content_id().unwrap();
        assert!(reconstruct(&chunks).is_err());
    }
    #[test]
    fn native_chunks_roundtrip_and_reject_missing_reordered_or_changed_rules() {
        let chunks: Vec<ProgramChunk> = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/forestry-program-chunks.json"
        ))
        .unwrap();
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/forestry-program-context.json"
        ))
        .unwrap();
        let groups = reconstruct(&chunks).unwrap();
        assert_eq!(groups.len(), 1, "Compiler dropped the shared native rules");
        assert_eq!(
            serde_json::to_value(&groups[&chunks[0].program]).unwrap(),
            expected
        );
        let mut reversed = chunks.clone();
        reversed.reverse();
        assert_eq!(
            serde_json::to_value(reconstruct(&reversed).unwrap()).unwrap(),
            serde_json::to_value(groups).unwrap()
        );
        let mut split = chunks[1..].to_vec();
        let ProgramData::SqueezerRecipes(rows) = &chunks[0].data else {
            panic!()
        };
        for (index, row) in rows.iter().enumerate() {
            let mut chunk = chunks[0].clone();
            chunk.offset = index as u32;
            chunk.data = ProgramData::SqueezerRecipes(vec![row.clone()]);
            chunk.id = chunk.content_id().unwrap();
            split.push(chunk);
        }
        assert_eq!(
            serde_json::to_value(&reconstruct(&split).unwrap()[&chunks[0].program]).unwrap(),
            expected
        );
        let mut duplicate = chunks.clone();
        duplicate.push(chunks[0].clone());
        assert!(reconstruct(&duplicate).is_err());
        assert!(reconstruct(&chunks[1..]).is_err());
        let mut missing = chunks.clone();
        missing[0].offset = 1;
        missing[0].id = missing[0].content_id().unwrap();
        assert!(reconstruct(&missing).is_err());
        let mut changed = chunks.clone();
        let ProgramData::SqueezerRecipes(rows) = &mut changed[0].data else {
            panic!()
        };
        rows[0].time += 1;
        changed[0].id = changed[0].content_id().unwrap();
        assert!(
            reconstruct(&changed).is_err(),
            "Whole-program hash did not catch a resealed changed chunk"
        );
    }
}
