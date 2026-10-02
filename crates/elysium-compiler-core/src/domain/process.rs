use super::{Consumption, Fluid, Kind, Match, OutputRole, Quantity, Recipe};
use crate::identity::integer;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HarmonyMode {
    Single,
    Parallel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HarmonyOutcome {
    Success,
    Failure,
}

/// Native shared process, not independent probabilities on individual outputs.
/// See docs/harmony.md for state, energy, rounding and correlated yield rules.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Process {
    Harmony {
        mode: HarmonyMode,
        hydrogen: String,
        helium: String,
        ticks: String,
        #[serde(rename = "startEu")]
        start_eu: String,
        #[serde(rename = "outputEu")]
        output_eu: String,
        /// Round-trip decimal representation of the native binary64 base chance.
        chance: String,
        #[serde(rename = "rocketTier")]
        rocket_tier: u8,
        #[serde(rename = "compressionTier")]
        compression_tier: u8,
    },
}

impl Process {
    pub fn max_parallel(&self) -> i64 {
        match self {
            Self::Harmony {
                mode: HarmonyMode::Single,
                ..
            } => 1,
            _ => 1_048_576,
        }
    }

    pub(super) fn validate_parameters(&self) -> Result<()> {
        let Self::Harmony {
            hydrogen,
            helium,
            ticks,
            start_eu,
            output_eu,
            chance,
            rocket_tier,
            compression_tier,
            ..
        } = self;
        for value in [hydrogen, helium, ticks, start_eu, output_eu] {
            integer(value, 1, i64::MAX)?;
        }
        let rate: f64 = chance.parse().context("invalid harmony chance")?;
        ensure!(
            rate.is_finite() && (0.0..=1.0).contains(&rate),
            "invalid harmony chance"
        );
        ensure!(
            *rocket_tier <= 9 && *compression_tier == (*rocket_tier).min(8),
            "invalid harmony tiers"
        );
        Ok(())
    }
}

pub(super) fn validate(recipe: &Recipe, fluids: &BTreeMap<&str, &Fluid>) -> Result<()> {
    let Some(process) = &recipe.process else {
        ensure!(
            !recipe.inputs.iter().any(|i| i
                .choices
                .iter()
                .any(|c| matches!(c.consume, Consumption::Buffer))),
            "buffer consumption requires a native process"
        );
        return Ok(());
    };
    process.validate_parameters()?;
    let Process::Harmony {
        mode,
        hydrogen,
        helium,
        rocket_tier,
        ..
    } = process;
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none(),
        "harmony process cannot have fixed timing, energy or crafting semantics"
    );
    ensure!(
        recipe.inputs.len() == if *mode == HarmonyMode::Single { 3 } else { 2 },
        "invalid harmony inputs"
    );
    let trigger = recipe
        .inputs
        .iter()
        .find(|i| i.kind == Kind::Item && i.slot == 0)
        .context("harmony trigger missing")?;
    ensure!(
        !trigger.choices.is_empty(),
        "harmony requires a native block trigger"
    );
    for c in &trigger.choices {
        ensure!(
            c.amount == "1"
                && matches!(c.consume, Consumption::Keep)
                && c.returns.is_empty()
                && matches!(
                    c.rule,
                    Match::Wildcard {
                        meta: true,
                        nbt: true
                    }
                ),
            "invalid harmony trigger semantics"
        );
    }
    let h = integer(hydrogen, 1, i64::MAX)?;
    let he = integer(helium, 1, i64::MAX)?;
    let required = if *mode == HarmonyMode::Single {
        vec![(0, "hydrogen", h), (1, "helium", he)]
    } else {
        vec![(
            2,
            "rawstarmatter",
            ((he as f64 * 1.24e-5 * 8.0).ceil() as i64).max(1),
        )]
    };
    for (slot, registry, amount) in required {
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.kind == Kind::Fluid && i.slot == slot)
            .context("harmony buffer missing")?;
        ensure!(input.choices.len() == 1, "harmony buffer has alternatives");
        let c = &input.choices[0];
        let fluid = fluids
            .get(c.id.as_str())
            .context("harmony buffer fluid missing")?;
        ensure!(
            fluid.registry == registry
                && fluid.nbt.is_none()
                && c.amount == amount.to_string()
                && matches!(c.consume, Consumption::Buffer)
                && c.returns.is_empty()
                && matches!(
                    c.rule,
                    Match::Wildcard {
                        meta: false,
                        nbt: true
                    }
                ),
            "invalid harmony buffer semantics"
        );
    }
    let mut failures = 0;
    let mut successes = 0;
    for output in &recipe.outputs {
        ensure!(
            matches!(output.role, OutputRole::Result),
            "harmony output is not a result"
        );
        let Some(Quantity::Harmony { outcome, nominal }) = &output.quantity else {
            anyhow::bail!("harmony output lacks shared quantity");
        };
        super::quantity_bounds(recipe, output)?;
        match outcome {
            HarmonyOutcome::Success => successes += 1,
            HarmonyOutcome::Failure => {
                failures += 1;
                ensure!(
                    output.kind == Kind::Fluid
                        && nominal == &(14400_i64 * (1_i64 << (rocket_tier + 1))).to_string(),
                    "invalid harmony failure quantity"
                );
                let fluid = fluids
                    .get(output.id.as_str())
                    .context("harmony failure fluid missing")?;
                ensure!(
                    fluid.registry == "molten.spacetime" && fluid.nbt.is_none(),
                    "invalid harmony failure fluid"
                );
            }
        }
    }
    ensure!(
        failures == 1 && successes > 0,
        "harmony requires normal outputs and exactly one failure output"
    );
    Ok(())
}
