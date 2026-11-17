//! AdvancedSolarPanel native ordered molecular transformation, not an EU/t rate.
use super::{Consumption, Item, Kind, Match, OutputRole, Quantity, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "SolarPriority")]
pub enum Priority {
    FirstItemMetadataNoFallback,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "SolarInputDebit")]
pub enum Debit {
    OnWorkStartWithoutContainers,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "SolarOutputPolicy")]
pub enum OutputPolicy {
    EmptyCopiesPrototypeMergeKeepsExistingTags,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "SolarEnergyPolicy")]
pub enum EnergyPolicy {
    TotalEuBufferedAndOutputSpace,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "MolecularTransformerProcess")]
pub struct SolarProcess {
    pub native_record: String,
    pub native_tile: String,
    pub version: String,
    pub native_index: u32,
    pub input: String,
    pub output: String,
    pub input_amount: String,
    pub output_amount: String,
    pub energy_per_operation: i32,
    pub priority: Priority,
    pub input_debit: Debit,
    pub output_policy: OutputPolicy,
    pub energy_policy: EnergyPolicy,
}
pub(super) fn validate(p: &SolarProcess, r: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(
        r.source.handler == "advsolar.client.nei.MTRecipeHandler"
            && p.native_record == "advsolar.utils.MTRecipeRecord"
            && p.native_tile == "advsolar.common.tiles.TileEntityMolecularTransformer"
            && p.version == "1.7.10-3.5.1"
            && p.native_index < 262144
            && p.energy_per_operation >= 0,
        "Molecular transformer native definition/energy changed"
    );
    ensure!(
        r.properties.is_empty()
            && r.notes.len() == 1
            && r.notes[0].key == "advancedsolar:processing"
            && r.notes[0].source
                == "advsolar.common.tiles.TileEntityMolecularTransformer.updateEntity",
        "Molecular transformer must retain its source note outside semantic identity"
    );
    ensure!(
        r.duration.is_none()
            && r.energy.is_none()
            && r.grid.is_none()
            && r.magic.is_none()
            && r.inputs.len() == 1
            && r.outputs.len() == 1,
        "Molecular transformer cannot become a timed crafting GUI"
    );
    let amount = count(&p.input_amount, true)?;
    let output_amount = count(&p.output_amount, false)?;
    let input = &r.inputs[0];
    let anchor = items
        .get(p.input.as_str())
        .context("Molecular transformer input missing")?;
    ensure!(
        input.kind == Kind::Item && input.slot == 0 && input.choices.len() == 1 && anchor.meta >= 0,
        "Molecular transformer input changed"
    );
    let choice = &input.choices[0];
    ensure!(choice.id==p.input&&choice.amount==amount.max(1).to_string()&&choice.returns.is_empty()
        &&matches!(choice.rule,Match::Wildcard{meta:false,nbt:true})
        &&(if amount==0{matches!(choice.consume,Consumption::Keep)}else{matches!(choice.consume,Consumption::Consume)}),
        "Molecular transformer lost literal metadata, ignored NBT, exact stock or no-container start debit");
    let output = &r.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && output.id == p.output
            && items.contains_key(p.output.as_str())
            && matches!(output.role, OutputRole::Result)
            && output.amount.is_none()
            && output.change.is_none()
            && output.chance.numerator == "1"
            && output.chance.denominator == "1"
            && matches!(&output.quantity,Some(Quantity::Potential{stat,nominal,condition,sample,parameters})
            if stat=="advancedsolar:outputState"&&nominal==&output_amount.to_string()&&sample.as_ref()==Some(nominal)
                &&condition.as_deref()==Some("nativeEnergyAndOutputAccepts")&&parameters.is_none()),
        "Molecular transformer output must retain energy, output-space and existing-tag conditions"
    );
    Ok(())
}
fn count(value: &str, zero: bool) -> Result<i32> {
    let number: i32 = value.parse()?;
    ensure!(
        value == number.to_string() && number >= if zero { 0 } else { 1 },
        "Invalid native molecular count"
    );
    Ok(number)
}
