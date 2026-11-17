//! Pinned Binnie lumbermill lookup, buffered work and conditional output placement.
use super::{Consumption, Fluid, Item, Kind, Match, OutputRole, Quantity, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LumbermillProcess {
    pub version: NativeVersion,
    pub work_ticks: u32,
    pub start_ticks: u32,
    pub queued_ticks: u32,
    pub rf_per_work_tick: u32,
    pub fluid_gate: u32,
    pub fluid_drain_limit: u32,
    pub native_index: u32,
    pub native_count: u32,
    pub first_ore_name: String,
    pub input_policy: InputPolicy,
    pub tank_policy: TankPolicy,
    pub output_policy: NativeOutputPolicy,
    pub work_policy: WorkPolicy,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum NativeVersion {
    #[serde(rename = "2.5.24")]
    Installed,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum InputPolicy {
    #[serde(rename = "normalFirstOreLogWoodRuntimeItemAndLiteralMetadata")]
    Native,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum TankPolicy {
    #[serde(rename = "normalWaterAdmissionRuntimeVolumeOnlyPartialDrain")]
    Native,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum NativeOutputPolicy {
    #[serde(rename = "emptyPrototypeMatchingMergeKeepsTagsBestEffortThenDebit")]
    Native,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum WorkPolicy {
    #[serde(rename = "cachedStateFloatProgressPauseResetAndReselectAtFinish")]
    Native,
}

pub(super) fn validate_native_recipe(
    p: &LumbermillProcess,
    r: &Recipe,
    items: &BTreeMap<&str, &Item>,
    fluids: &BTreeMap<&str, &Fluid>,
) -> Result<()> {
    ensure!(
        r.source.owner == "ExtraTrees"
            && r.source.handler == "binnie.extratrees.nei.NEIHandlerLumbermill"
            && p.work_ticks == 30
            && p.start_ticks == 1
            && p.queued_ticks == 30
            && p.rf_per_work_tick == 30
            && p.fluid_gate == 5
            && p.fluid_drain_limit == 10
            && p.native_count > 0
            && p.native_count <= 65536
            && p.native_index < p.native_count
            && p.first_ore_name.len() <= 65535,
        "Native lumbermill source, progress or work law differs"
    );
    ensure!(
        r.duration.is_none()
            && r.energy.is_none()
            && r.grid.is_none()
            && r.magic.is_none()
            && r.properties.is_empty()
            && r.inputs.len() == 2
            && r.outputs.len() == 3
            && r.notes.len() == 1
            && r.notes[0].key == "binnie:lumbermill"
            && r.notes[0].source
                == "binnie.extratrees.machines.lumbermill.LumbermillComponentLogic",
        "Native lumbermill GUI-free inventory semantics missing"
    );
    for (index, input) in r.inputs.iter().enumerate() {
        ensure!(
            input.slot == index as u32
                && input.kind == if index == 0 { Kind::Item } else { Kind::Fluid }
                && input.choices.len() == 1,
            "Native lumbermill physical input roles changed"
        );
        let c = &input.choices[0];
        ensure!(
            c.returns.is_empty()
                && matches!(c.consume, Consumption::Allocated)
                && c.amount == if index == 0 { "1" } else { "5" },
            "Lumbermill invented fixed per-cycle fluid or container consumption"
        );
        if index == 0 {
            ensure!(
                items.contains_key(c.id.as_str())
                    && matches!(
                        c.rule,
                        Match::Wildcard {
                            meta: false,
                            nbt: true
                        }
                    ),
                "Lumbermill runtime wood requires literal metadata and ignores tags"
            );
        } else {
            let f = fluids
                .get(c.id.as_str())
                .context("Missing water-role sample")?;
            ensure!(
                f.registry == "water" && f.nbt.is_none() && matches!(c.rule, Match::LumbermillTank),
                "Lumbermill normal water sample became runtime fluid identity filter"
            );
        }
    }
    for (index, o) in r.outputs.iter().enumerate() {
        ensure!(
            o.kind == Kind::Item
                && o.slot == index as u32
                && items.contains_key(o.id.as_str())
                && matches!(o.role, OutputRole::Result)
                && o.change.is_none()
                && o.amount.is_none()
                && o.chance.numerator == "1"
                && o.chance.denominator == "1"
                && matches!(&o.quantity,Some(Quantity::Potential{stat,condition,nominal,sample,parameters})if stat=="binnie:lumbermillOutputState"
                &&condition.as_deref()==Some("nativeLumbermillSlotAccepts")&&nominal==if index==0{"6"}else{"1"}&&sample.is_none()&&parameters.is_none()),
            "Lumbermill output must preserve conditional empty-slot inventory prototype"
        );
    }
    let bark=items[r.outputs[1].id.as_str()];let dust=items[r.outputs[2].id.as_str()];
    ensure!(bark.registry==dust.registry&&bark.meta==2&&dust.meta==1&&bark.nbt.is_none()&&dust.nbt.is_none(),
        "Lumbermill bark/sawdust metadata roles changed");
    Ok(())
}
