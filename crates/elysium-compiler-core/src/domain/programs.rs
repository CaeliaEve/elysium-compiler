//! Shared, bounded machine rule chunks. Whole-program identity retains native order.
use super::forestry::{FilledContainer, SqueezerContainer, SqueezerProgram, SqueezerRecipe};
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
    SqueezerRecipes(Vec<SqueezerRecipe>),
    SqueezerContainers(Vec<SqueezerContainer>),
    SqueezerFluids(Vec<FilledContainer>),
    SqueezerCallbacks(Vec<String>),
}

impl ProgramData {
    fn section(&self) -> usize {
        match self {
            Self::SqueezerRecipes(_) => 0,
            Self::SqueezerContainers(_) => 1,
            Self::SqueezerFluids(_) => 2,
            Self::SqueezerCallbacks(_) => 3,
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::SqueezerRecipes(v) => v.len(),
            Self::SqueezerContainers(v) => v.len(),
            Self::SqueezerFluids(v) => v.len(),
            Self::SqueezerCallbacks(v) => v.len(),
        }
    }
}

pub fn reconstruct(chunks: &[ProgramChunk]) -> Result<BTreeMap<String, SqueezerProgram>> {
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
        let mut program = SqueezerProgram {
            ordinary: vec![],
            containers: vec![],
            filled: vec![],
            dynamic: vec![],
        };
        let mut lengths = [0; 4];
        let mut seen = [false; 4];
        let mut empty = [false; 4];
        let mut bytes = 0;
        for chunk in chunks {
            let section = chunk.data.section();
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
            match &chunk.data {
                ProgramData::SqueezerRecipes(rows) => program.ordinary.extend_from_slice(rows),
                ProgramData::SqueezerContainers(rows) => program.containers.extend_from_slice(rows),
                ProgramData::SqueezerFluids(rows) => program.filled.extend_from_slice(rows),
                ProgramData::SqueezerCallbacks(rows) => program.dynamic.extend_from_slice(rows),
            }
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
                &serde_json::json!({"kind":"forestrySqueezer","rules":program})
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
