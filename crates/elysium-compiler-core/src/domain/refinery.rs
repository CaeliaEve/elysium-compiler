//! Galacticraft tank conversion, including drain-before-rejected-fill loss.
use super::{Consumption, Fluid, Kind, Match, OutputRole, Quantity, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
macro_rules! law {
    ($name:ident,$value:ident) => {
        #[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "camelCase")]
        pub enum $name {
            $value,
        }
    };
}
law!(RefineryScope, FluidTankConversion);
law!(
    RefineryAdmission,
    CaseSensitiveOilPrefixCanonicalKeepsTagsAlternateDiscardsTags
);
law!(RefineryWork, PoweredEnabledCountdownResetOnBlocked);
law!(RefineryOutputPolicy, DrainBeforeTryFillIgnoringRejectedFuel);
law!(RefineryContainers, TankOperationHasNoConsumedItemContainers);
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RefineryProcess {
    pub native_tile: String,
    pub version: String,
    pub scope: RefineryScope,
    pub input: String,
    pub normalized_oil: String,
    pub output: String,
    pub legacy_fuel: bool,
    pub hard: bool,
    pub max_extract_gj: u8,
    pub tank_capacity: u32,
    pub countdown_ticks: u8,
    pub fresh_startup_ticks: u8,
    pub transfer_amount: String,
    pub admission: RefineryAdmission,
    pub work: RefineryWork,
    pub output_policy: RefineryOutputPolicy,
    pub container_policy: RefineryContainers,
}
pub(super) fn validate(
    p: &RefineryProcess,
    r: &Recipe,
    fluids: &BTreeMap<&str, &Fluid>,
) -> Result<()> {
    ensure!(
        r.source.handler == "micdoodle8.mods.galacticraft.core.nei.RefineryRecipeHandler"
            && p.native_tile == "micdoodle8.mods.galacticraft.core.tile.TileEntityRefinery"
            && p.version == "3.3.13-GTNH"
            && p.max_extract_gj == if p.hard { 90 } else { 60 }
            && p.tank_capacity == 24000
            && p.countdown_ticks == 2
            && p.fresh_startup_ticks == 1
            && p.transfer_amount == "1",
        "Refinery native definition/configuration changed"
    );
    let input = fluids
        .get(p.input.as_str())
        .context("Refinery input missing")?;
    let oil = fluids
        .get(p.normalized_oil.as_str())
        .context("Refinery canonical oil missing")?;
    let fuel = fluids
        .get(p.output.as_str())
        .context("Refinery output missing")?;
    ensure!(
        input.registry.starts_with("oil")
            && oil.registry.starts_with("oil")
            && fuel.registry == if p.legacy_fuel { "fuelgc" } else { "fuel" }
            && [input, oil, fuel].iter().all(|f| f.nbt.is_none()),
        "Refinery lost native oil prefix/normalization/fuel role"
    );
    ensure!(
        r.properties.is_empty()
            && r.duration.is_none()
            && r.energy.is_none()
            && r.grid.is_none()
            && r.magic.is_none()
            && r.inputs.len() == 1
            && r.outputs.len() == 1
            && r.notes.len() == 1
            && r.notes[0].key == "galacticraft:refinery"
            && r.notes[0].source == format!("{}.updateEntity", p.native_tile),
        "Refinery cannot imply fixed time, energy or containers"
    );
    let i = &r.inputs[0];
    ensure!(
        i.kind == Kind::Fluid && i.slot == 0 && i.choices.len() == 1,
        "Refinery must consume a tank fluid"
    );
    let c = &i.choices[0];
    ensure!(
        c.id == p.input
            && c.amount == "1"
            && c.returns.is_empty()
            && matches!(
                c.rule,
                Match::Wildcard {
                    meta: false,
                    nbt: true
                }
            )
            && matches!(c.consume, Consumption::Consume),
        "Refinery oil debit/admission changed"
    );
    let o = &r.outputs[0];
    ensure!(
        o.kind == Kind::Fluid
            && o.slot == 0
            && o.id == p.output
            && matches!(o.role, OutputRole::Result)
            && o.amount.is_none()
            && o.change.is_none()
            && o.chance.numerator == "1"
            && o.chance.denominator == "1"
            && matches!(&o.quantity,Some(Quantity::Potential{stat,nominal,condition,sample,parameters})
            if stat=="galacticraft:refineryOutputState"&&nominal=="1"&&sample.as_deref()==Some("1")
                &&condition.as_deref()==Some("nativeRefineryTankAccepts")&&parameters.is_none()),
        "Refinery fuel must retain drain-before-fill/output-tank uncertainty"
    );
    Ok(())
}
