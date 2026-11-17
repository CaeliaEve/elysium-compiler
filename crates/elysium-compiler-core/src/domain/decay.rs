//! GT++ pinned decay chest, explicitly scoped to fresh root-untagged dust.
use super::{Consumption, Item, Kind, Match, OutputRole, Quantity, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "DecayChestScope")]
pub enum Scope {
    FreshUntagged,
    NativeStateLaw,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "DecayChestStateLaw")]
pub enum StateLaw {
    TickableItemBatchV1,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "DecayChestScheduler")]
pub enum Scheduler {
    LoadedServerInt32Modulo20,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "DecayChestReplacement")]
pub enum Replacement {
    WholeStoredStackToOne,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "DecayChestProcess")]
pub struct DecayProcess {
    pub native_recipe: String,
    pub native_tile: String,
    pub native_item: String,
    pub version: String,
    pub native_index: u32,
    pub scope: Scope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_law: Option<StateLaw>,
    pub input: String,
    pub output: String,
    pub max_ticks: i32,
    pub displayed_ticks: i32,
    pub fresh_batches: String,
    pub batch_steps: u8,
    pub inventory_limit: u8,
    pub scheduler: Scheduler,
    pub replacement: Replacement,
}

pub(super) fn validate(p: &DecayProcess, r: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(
        r.properties.is_empty() && r.notes.len() == 4,
        "Decay typed rules cannot be replaced by translated property values or lose source notes"
    );
    for (key, source) in [
        (
            "decay:scope",
            "gtPlusPlus.core.tileentities.general.TileEntityDecayablesChest.tryUpdateDecayable",
        ),
        (
            "decay:scheduler",
            "gtPlusPlus.core.tileentities.general.TileEntityDecayablesChest.updateEntity",
        ),
        (
            "decay:progress",
            "gtPlusPlus.core.item.base.BaseItemTickable.tickItemTag",
        ),
        (
            "decay:replacement",
            "gtPlusPlus.core.tileentities.general.TileEntityDecayablesChest.tryUpdateDecayable",
        ),
    ] {
        ensure!(
            r.notes
                .iter()
                .any(|note| note.key == key && note.source == source),
            "Decay native explanatory source missing or changed"
        );
    }
    let all_states = matches!(p.scope, Scope::NativeStateLaw);
    ensure!(
        p.state_law.is_some() == all_states,
        "Decay saved-state scope requires its explicit pinned transition law"
    );
    ensure!(
        r.source.handler == "gtPlusPlus.nei.DecayableRecipeHandler"
            && p.native_recipe == "gtPlusPlus.core.handler.Recipes.DecayableRecipe"
            && p.native_tile == "gtPlusPlus.core.tileentities.general.TileEntityDecayablesChest"
            && p.native_item == "gtPlusPlus.core.item.materials.DustDecayable"
            && p.version == "5.09.51.482"
            && p.native_index < 262144
            && p.max_ticks > 0
            && p.displayed_ticks > 0
            && p.fresh_batches == ((i64::from(p.max_ticks) + 21) / 20).to_string()
            && p.batch_steps == 20
            && p.inventory_limit == 1,
        "Decay chest lost its native definition/batch law"
    );
    ensure!(
        r.grid.is_none()
            && r.magic.is_none()
            && r.energy.is_none()
            && r.duration.is_none()
            && r.inputs.len() == 1
            && r.outputs.len() == 1,
        "Decay cannot use crafting cells or a fixed native display duration"
    );
    let input = &r.inputs[0];
    ensure!(
        input.kind == Kind::Item && input.slot == 0 && input.choices.len() == 1,
        "Decay requires one stored input stack"
    );
    let choice = &input.choices[0];
    let item = items
        .get(p.input.as_str())
        .context("Decay dust fact missing")?;
    ensure!(
        choice.id == p.input
            && choice.amount == "1"
            && item.meta == 0
            && item.nbt.is_none()
            && matches!(choice.rule,Match::Wildcard{meta:true,nbt} if nbt==all_states)
            && matches!(choice.consume, Consumption::Stack)
            && choice.returns.is_empty(),
        "Decay scope changed metadata/NBT/whole-stack replacement"
    );
    let output = &r.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && output.id == p.output
            && items.contains_key(p.output.as_str())
            && (if all_states {
                output.amount.is_none()&&matches!(&output.quantity,Some(Quantity::Potential{stat,nominal,condition,sample,parameters})
                    if stat=="miscutils:decayState"&&nominal=="1"&&condition.as_deref()==Some("nativeBatchCompletes")&&sample.as_deref()==Some("1")&&parameters.is_none())
            } else {output.amount.as_deref()==Some("1")&&output.quantity.is_none()})
            && output.change.is_none()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "Decay must retain conditional completion and replace the stored stack with one native result"
    );
    Ok(())
}
