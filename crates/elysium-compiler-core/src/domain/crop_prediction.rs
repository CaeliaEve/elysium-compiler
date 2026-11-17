//! IC2 Crop Plugin's conditional crossing predictor; never a timed seed recipe.
use super::{Consumption, Kind, Match, OutputRole, Recipe};
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CropKey {
    pub owner: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Prediction {
    pub parents: Vec<CropKey>,
    pub result: CropKey,
    pub points: u32,
    pub total: u32,
    pub crafting_listed: bool,
    pub usage_listed_for: Vec<CropKey>,
}

pub(super) fn validate(prediction: &Prediction, recipe: &Recipe) -> Result<()> {
    ensure!(recipe.source.handler == "speiger.src.crops.prediction.NEIPlugin",
        "crop predictor requires its native handler");
    ensure!(recipe.duration.is_none() && recipe.energy.is_none() && recipe.grid.is_none()
        && recipe.magic.is_none(), "crop prediction cannot promise timed production");
    ensure!((2..=4).contains(&prediction.parents.len())
        && prediction.parents.len() == recipe.inputs.len(), "invalid crop parent count");
    ensure!(prediction.points > 0 && prediction.points <= prediction.total
        && prediction.total <= i32::MAX as u32, "invalid crop prediction weights");
    for crop in prediction.parents.iter().chain(std::iter::once(&prediction.result)) {
        ensure!(!crop.owner.is_empty() && crop.owner.len() <= 1024
            && !crop.name.is_empty() && crop.name.len() <= 1024, "invalid native crop identity");
    }
    ensure!(!prediction.parents.contains(&prediction.result), "native predictor omits existing parents");
    ensure!(prediction.crafting_listed||!prediction.usage_listed_for.is_empty(),"Prediction has no native query membership");
    for (index,parent) in prediction.usage_listed_for.iter().enumerate(){
        ensure!(prediction.parents.contains(parent)&&!prediction.usage_listed_for[..index].contains(parent),"Invalid native prediction usage membership");
    }
    for (index, parent) in prediction.parents.iter().enumerate() {
        ensure!(!prediction.parents[..index].contains(parent), "duplicate native crop parent");
    }
    for (slot, input) in recipe.inputs.iter().enumerate() {
        ensure!(matches!(input.kind, Kind::Item) && input.slot == slot as u32
            && input.choices.len() == 1, "invalid crop parent display");
        let choice = &input.choices[0];
        ensure!(choice.amount == "1" && matches!(choice.consume, Consumption::Keep)
            && matches!(choice.rule, Match::Exact) && choice.returns.is_empty(),
            "crop parents must remain plants, not consumed seed ingredients");
    }
    ensure!(recipe.outputs.len() == 1, "prediction requires one conditional result");
    let output = &recipe.outputs[0];
    ensure!(matches!(output.kind, Kind::Item) && output.slot == 0
        && output.amount.as_deref() == Some("1") && output.quantity.is_none()
        && output.change.is_none() && matches!(output.role, OutputRole::Result),
        "invalid predicted crop display");
    super::check::chance(&output.chance)?;
    let numerator: u64 = output.chance.numerator.parse()?;
    let denominator: u64 = output.chance.denominator.parse()?;
    ensure!((numerator as u128) * u128::from(prediction.total)
        == (denominator as u128) * u128::from(prediction.points),
        "crop chance differs from native weights");
    Ok(())
}
