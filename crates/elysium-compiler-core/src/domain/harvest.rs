use super::{Consumption, Kind, Match, OutputRole, Quantity, Recipe};
use crate::identity::integer;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HarvestMode { Garden, Fish, Animal }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HarvestDrops {
    pub mode: HarvestMode,
    pub draws: u32,
    /// Native registry order; zero-weight candidates cannot be selected.
    pub weights: Vec<u32>,
}

impl HarvestDrops {
    fn total(&self) -> Result<u64> {
        ensure!(!self.weights.is_empty() && self.weights.len() <= 128, "Invalid Harvest candidate count");
        let total = self.weights.iter().map(|w| u64::from(*w)).sum::<u64>();
        ensure!(total > 0 && total <= i32::MAX as u64, "Native Harvest selection weight overflows or is zero");
        Ok(total)
    }
    pub(super) fn bounds(&self, slot: u32, nominal: &str) -> Result<(i64,i64)> {
        let total = self.total()?;
        ensure!(self.draws > 0 && self.draws <= i32::MAX as u32, "Invalid Harvest draw count");
        let weight = u64::from(*self.weights.get(slot as usize).context("Missing Harvest output weight")?);
        let nominal = integer(nominal,1,i32::MAX as i64)?;
        let high = if weight == 0 {0} else {nominal.checked_mul(i64::from(self.draws)).context("Harvest quantity overflows")?};
        Ok((if weight == total {high} else {0},high))
    }
}

pub(super) fn validate_recipe(recipe: &Recipe, drops: &HarvestDrops) -> Result<()> {
    drops.total()?;
    let handler=match drops.mode {
        HarvestMode::Garden => "com.pam.harvestcraft.GardenRecipeHandler",
        HarvestMode::Fish => "com.pam.harvestcraft.FishTrapRecipeHandler",
        HarvestMode::Animal => "com.pam.harvestcraft.AnimalTrapRecipeHandler",
    };
    ensure!(recipe.source.handler == handler && recipe.source.owner == "harvestcraft", "Harvest process has the wrong native origin");
    ensure!(recipe.duration.is_none() && recipe.energy.is_none() && recipe.grid.is_none() && recipe.magic.is_none(), "Harvest drops cannot invent static timing or crafting");
    ensure!(recipe.inputs.len()==1 && recipe.inputs[0].kind==Kind::Item && recipe.inputs[0].slot==0 && recipe.inputs[0].choices.len()==1, "Harvest requires one native block or bait");
    let choice=&recipe.inputs[0].choices[0];
    ensure!(choice.amount=="1" && matches!(choice.consume,Consumption::Consume) && choice.returns.is_empty()
        && matches!(choice.rule,Match::Wildcard {meta,nbt:true} if meta == (drops.mode!=HarvestMode::Garden)), "Harvest input must retain its native metadata admission");
    ensure!(recipe.outputs.len()==drops.weights.len(), "Harvest weights must correspond to every output slot");
    if drops.mode==HarvestMode::Garden {
        ensure!(drops.weights.iter().all(|w| *w==1), "Garden drops use uniform independent selections with replacement");
    } else {
        ensure!(drops.draws==1, "A trap completes one mutually exclusive selection");
    }
    for (slot,output) in recipe.outputs.iter().enumerate() {
        ensure!(output.kind==Kind::Item && output.slot as usize==slot && matches!(output.role,OutputRole::Result)
            && output.amount.is_none() && output.change.is_none() && output.chance.numerator=="1" && output.chance.denominator=="1", "Harvest candidates cannot be independent fixed outputs");
        let Some(Quantity::Harvest {nominal})=&output.quantity else {anyhow::bail!("Harvest output lacks shared quantity");};
        if drops.mode==HarvestMode::Garden {ensure!(nominal=="1", "Garden emits one item per selection");}
        drops.bounds(output.slot,nominal)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::domain::{quantity_bounds, Recipe};
    use serde_json::{json, Value};

    fn fixture(mode: &str, draws: u32, weights: &[u32]) -> Value {
        let mut value=json!({"id":"recipe_harvest","source":{"owner":"harvestcraft","handler":if mode=="garden" {"com.pam.harvestcraft.GardenRecipeHandler"} else {"com.pam.harvestcraft.FishTrapRecipeHandler"},"key":"harvest"},
            "category":"category_harvest","order":0,"duration":null,"energy":null,"grid":null,"magic":null,"properties":{},
            "process":{"kind":"harvestDrops","mode":mode,"draws":draws,"weights":weights},
            "inputs":[{"kind":"item","slot":0,"choices":[{"id":"item_bait","amount":"1","rule":{"kind":"wildcard","meta":true,"nbt":true},"consume":{"kind":"consume"},"returns":[]}]}],
            "outputs":[{"kind":"item","slot":0,"id":"item_a","amount":null,"quantity":{"kind":"harvest","nominal":if mode=="garden" {"1"}else{"2"}},"change":null,"role":"result","chance":{"numerator":"1","denominator":"1"}},
            {"kind":"item","slot":1,"id":"item_b","amount":null,"quantity":{"kind":"harvest","nominal":if mode=="garden" {"1"}else{"3"}},"change":null,"role":"result","chance":{"numerator":"1","denominator":"1"}}]});
        value["inputs"][0]["choices"][0]["rule"]["meta"]=json!(mode!="garden");
        value
    }

    fn check(value: Value) -> anyhow::Result<Recipe> {
        let recipe: Recipe = serde_json::from_value(value)?;
        super::super::process::validate(&recipe, &Default::default(), &Default::default(), &[])?;
        for output in &recipe.outputs { quantity_bounds(&recipe, output)?; }
        Ok(recipe)
    }

    #[test]
    fn harvest_keeps_integer_weights_and_correlated_native_draws() {
        let trap=check(fixture("fish",1,&[1,2])).unwrap();
        assert_eq!(quantity_bounds(&trap,&trap.outputs[0]).unwrap(),(0,2));
        assert_eq!(quantity_bounds(&trap,&trap.outputs[1]).unwrap(),(0,3));
        let garden=check(fixture("garden",4,&[1,1])).unwrap();
        assert_eq!(quantity_bounds(&garden,&garden.outputs[0]).unwrap(),(0,4));
        let zero=check(fixture("fish",1,&[0,2])).unwrap();
        assert_eq!(quantity_bounds(&zero,&zero.outputs[0]).unwrap(),(0,0));
        assert_eq!(quantity_bounds(&zero,&zero.outputs[1]).unwrap(),(3,3));
    }

    #[test]
    fn harvest_rejects_flattened_outputs_and_invalid_selection_contracts() {
        let value=fixture("fish",1,&[1,2]);
        check(value.clone()).unwrap();
        for (path,bad) in [
            ("/process/weights",json!([1])),("/process/weights",json!([0,0])),
            ("/process/weights",json!([2147483647,1])),("/process/draws",json!(2)),
            ("/source/handler",json!("unrelated")),("/duration",json!("3200")),
            ("/inputs/0/choices/0/rule/meta",json!(false)),
            ("/outputs/0/amount",json!("2")),("/outputs/0/quantity",Value::Null),
            ("/outputs/0/quantity/nominal",json!("0")),("/outputs/0/chance/denominator",json!("3")),
        ] {
            let mut wrong=value.clone(); *wrong.pointer_mut(path).unwrap()=bad;
            assert!(check(wrong).is_err(),"accepted {path}");
        }
        let mut wrong=fixture("garden",4,&[1,2]);
        assert!(check(wrong.clone()).is_err(),"garden cannot use trap weights");
        wrong["process"]["weights"]=json!([1,1]);wrong["outputs"][0]["quantity"]["nominal"]=json!("2");
        assert!(check(wrong).is_err(),"garden drop callback emits one item per draw");
    }
}
