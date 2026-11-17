//! Gadomancy caches one random cost table. Its definition is not a fixed sampled price.
use super::AspectAmount;
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag="kind", rename_all="camelCase", deny_unknown_fields)]
pub enum RandomAspects {
    #[serde(rename_all="camelCase")]
    Gadomancy { semantics:CostSemantics, half_cap:bool, refresh_millis:u32, bases:Vec<AspectAmount> },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum CostSemantics { #[serde(rename="gadomancyRandomCostsV1")] Native }

impl RandomAspects {
    pub fn bases(&self)->&[AspectAmount] {let Self::Gadomancy{bases,..}=self;bases}
    /// Inclusive display bounds; zero-valued draws are omitted from the native AspectList.
    pub fn ranges(&self)->Result<Vec<(&str,i32,i32)>> {
        let Self::Gadomancy{half_cap,refresh_millis,bases,..}=self;
        ensure!(*refresh_millis==500&&bases.len()<=4096,"invalid native random aspect cache law");
        let mut seen=BTreeSet::new();let mut result=Vec::new();
        for a in bases {
            ensure!(seen.insert(&a.aspect),"duplicate native random aspect");
            let base=crate::identity::integer(&a.amount,1,i64::from(i32::MAX))? as i32;
            let bound=if *half_cap{base/2}else{base};
            ensure!(bound>0,"native random aspect RNG bound is nonpositive");
            let min=if *half_cap{bound}else{0};
            result.push((a.aspect.as_str(),min,min+bound-1));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::RandomAspects;
    use serde_json::json;

    #[test]
    fn stable_cost_semantics_preserve_native_ranges() {
        for (half, expected) in [(false, (0, 8)), (true, (4, 7))] {
            let value = json!({"kind":"gadomancy","semantics":"gadomancyRandomCostsV1",
                "halfCap":half,"refreshMillis":500,"bases":[{"aspect":"aspect_air","amount":"9"}]});
            let law: RandomAspects = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(law.ranges().unwrap(), vec![("aspect_air", expected.0, expected.1)]);
            let mut bad = value;
            bad["semantics"] = json!("unreviewedRule");
            assert!(serde_json::from_value::<RandomAspects>(bad).is_err());
        }
    }
}
