//! Pinned Galacticraft five-slot matching and conditional native output state.
use super::{Consumption, Item, Kind, Match, OutputRole, Quantity, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! law {
    ($name:ident,$variant:ident) => {
        #[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "camelCase")]
        pub enum $name {
            $variant,
        }
    };
}
law!(CircuitPriority, FirstMapEncounter);
law!(CircuitMatcher, ItemLiteralMetadataIgnoreCountAndTags);
law!(CircuitDebit, OnCompletionOneWithoutContainers);
law!(CircuitCapacity, EmptyOrOriginalCountSignedIntWithinLimits);
law!(CircuitOutput, EmptyCopiesPrototypeMergeKeepsExistingTags);
law!(
    CircuitOverflow,
    MergeAbove64SpawnsUntaggedUnitsIgnoringSpawnResult
);
law!(CircuitDirectCall, DebitsEvenIfOutputRejected);
law!(CircuitWork, ConsecutiveEligibleTicksResetOnBlocked);

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CircuitProcess {
    pub native_registry: String,
    pub native_tile: String,
    pub version: String,
    pub native_index: u32,
    /// Native inventory slots 1..5. A null is required empty, not an unchecked cell.
    pub slots: [Option<String>; 5],
    pub output: String,
    pub raw_output_amount: String,
    pub output_amount: String,
    pub quick: bool,
    pub hard: bool,
    pub basic_item: Option<String>,
    pub processing_ticks: u16,
    pub max_extract_gj: u8,
    pub work: CircuitWork,
    pub priority: CircuitPriority,
    pub matcher: CircuitMatcher,
    pub input_debit: CircuitDebit,
    pub capacity: CircuitCapacity,
    pub output_policy: CircuitOutput,
    pub overflow: CircuitOverflow,
    pub direct_call: CircuitDirectCall,
}
pub fn is_handler(handler: &str) -> bool {
    matches!(
        handler,
        "micdoodle8.mods.galacticraft.core.nei.CircuitFabricatorRecipeHandler"
            | "de.katzenpapst.amunra.nei.recipehandler.ARCircuitFab"
    )
}
pub(super) fn validate(
    p: &CircuitProcess,
    r: &Recipe,
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    ensure!(
        is_handler(&r.source.handler)
            && p.native_registry
                == "micdoodle8.mods.galacticraft.api.recipe.CircuitFabricatorRecipes"
            && p.native_tile
                == "micdoodle8.mods.galacticraft.core.tile.TileEntityCircuitFabricator"
            && p.version == "3.3.13-GTNH"
            && p.native_index < 262144
            && p.processing_ticks == 300
            && p.max_extract_gj == if p.hard { 40 } else { 20 },
        "Circuit native definition, work ticks or extraction ceiling changed"
    );
    ensure!(
        r.properties.is_empty()
            && r.duration.is_none()
            && r.energy.is_none()
            && r.grid.is_none()
            && r.magic.is_none()
            && r.notes.len() == 2
            && r.notes[0].key == "galacticraft:layout"
            && r.notes[0].source == format!("{}.updateInput", p.native_tile)
            && r.notes[1].key == "galacticraft:outputState"
            && r.notes[1].source == format!("{}.compressItems", p.native_tile),
        "Circuit cannot become a generic GUI recipe or wall-clock duration"
    );
    ensure!(
        r.inputs.len() == p.slots.iter().flatten().count(),
        "Circuit lost fixed/empty slots"
    );
    for (slot, id) in p.slots.iter().enumerate() {
        let input = r.inputs.iter().find(|i| i.slot == slot as u32 + 1);
        let Some(id) = id else {
            ensure!(input.is_none(), "Circuit required empty slot occupied");
            continue;
        };
        let input = input.context("Circuit occupied slot missing")?;
        let fact = items
            .get(id.as_str())
            .context("Circuit input fact missing")?;
        ensure!(
            input.kind == Kind::Item
                && input.choices.len() == 1
                && fact.meta >= 0
                && fact.nbt.is_none(),
            "Circuit input is not an owned literal-metadata exemplar"
        );
        let c = &input.choices[0];
        ensure!(
            &c.id == id
                && c.amount == "1"
                && matches!(c.consume, Consumption::Consume)
                && c.returns.is_empty()
                && matches!(
                    c.rule,
                    Match::Wildcard {
                        meta: false,
                        nbt: true
                    }
                ),
            "Circuit input must ignore count/tags, consume one and return no container"
        );
    }
    let raw = count(&p.raw_output_amount)?;
    let adjusted = count(&p.output_amount)?;
    let fact = items
        .get(p.output.as_str())
        .context("Circuit output prototype missing")?;
    ensure!(
        p.basic_item
            .as_ref()
            .is_none_or(|s| !s.is_empty() && s.len() <= 512)
            && (!p.quick || p.basic_item.is_some()),
        "Circuit quick-mode native role missing"
    );
    let expected = if p.quick && p.basic_item.as_deref() == Some(&fact.registry) {
        match fact.meta {
            13 => 5,
            14 => 2,
            _ => raw,
        }
    } else {
        raw
    };
    ensure!(
        adjusted == expected && r.outputs.len() == 1,
        "Circuit quick output count changed"
    );
    let o = &r.outputs[0];
    ensure!(
        o.kind == Kind::Item
            && o.slot == 6
            && o.id == p.output
            && matches!(o.role, OutputRole::Result)
            && o.amount.is_none()
            && o.change.is_none()
            && o.chance.numerator == "1"
            && o.chance.denominator == "1"
            && matches!(&o.quantity,Some(Quantity::Potential{stat,nominal,condition,sample,parameters})
            if stat=="galacticraft:circuitOutputState" && nominal==&adjusted.to_string()
            && condition.as_deref()==Some("nativeCircuitOutputState") && sample.as_ref()==Some(nominal)
            && parameters.is_none()),
        "Circuit output must retain capacity/merge/drop conditions"
    );
    Ok(())
}
fn count(value: &str) -> Result<i32> {
    let n: i32 = value.parse()?;
    ensure!(
        n > 0 && value == n.to_string(),
        "Invalid circuit native count"
    );
    Ok(n)
}
