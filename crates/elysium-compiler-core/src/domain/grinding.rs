use super::{Consumption, Item, Kind, Match, MatchCase, OutputRole, Process, Quantity, Recipe};
use crate::identity::integer;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GrindingRequirement {
    pub amount: String,
    pub choices: Vec<MatchCase>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GrindingBall {
    pub choices: Vec<MatchCase>,
    pub grinding: String,
    pub chance: String,
    pub power: String,
    pub duration: i32,
}

pub(super) fn validate(recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    let Some(Process::Sag {
        slot,
        earlier,
        balls,
        blocked,
        ore_blocked,
        ..
    }) = &recipe.process
    else {
        unreachable!()
    };
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none()
            && (-1..=1).contains(slot)
            && (1..=2).contains(&recipe.inputs.len())
            && (1..=128).contains(&recipe.outputs.len())
            && earlier.len() <= 262144
            && balls.len() <= 4096,
        "invalid SAG shape"
    );
    for (index, input) in recipe.inputs.iter().enumerate() {
        ensure!(
            input.kind == Kind::Item && input.slot == index as u32 && !input.choices.is_empty(),
            "invalid SAG input order"
        );
        let amount = &input.choices[0].amount;
        integer(amount, 1, i64::from(i32::MAX))?;
        for c in &input.choices {
            ensure!(
                &c.amount == amount
                    && c.returns.is_empty()
                    && matches!(c.rule, Match::Wildcard { nbt: true, .. })
                    && if index == 0 {
                        matches!(c.consume, Consumption::Allocated)
                    } else {
                        c.amount == "1" && matches!(c.consume, Consumption::Reserve)
                    },
                "invalid SAG material or reserve consumption"
            );
        }
    }
    let cases = |values: &[MatchCase]| -> Result<()> {
        ensure!(values.len() <= 65536, "oversized SAG predicates");
        let mut seen = BTreeSet::new();
        for c in values {
            let item = items
                .get(c.id.as_str())
                .context("missing SAG predicate item")?;
            ensure!(
                matches!(c.rule, Match::Wildcard { nbt: true, .. })
                    && seen.insert(serde_json::to_string(c)?),
                "invalid or repeated SAG predicate"
            );
            c.rule.validate_item(item, items, false, false)?;
        }
        Ok(())
    };
    for p in earlier {
        integer(&p.amount, 1, i64::from(i32::MAX))?;
        ensure!(!p.choices.is_empty(), "empty SAG priority predicate");
        cases(&p.choices)?;
    }
    let mut ball_choices = BTreeSet::new();
    for b in balls {
        cases(&b.choices)?;
        for c in &b.choices {
            ball_choices.insert(serde_json::to_string(c)?);
        }
        let grinding: f32 = b.grinding.parse().context("invalid grinding multiplier")?;
        let chance: f32 = b.chance.parse().context("invalid ball chance multiplier")?;
        let power: f32 = b.power.parse().context("invalid ball power multiplier")?;
        ensure!(
            grinding.is_finite()
                && (0.0..=16_777_218.0).contains(&grinding)
                && chance.is_finite()
                && chance > 0.0
                && power.is_finite()
                && power >= 0.0,
            "unsupported ball parameters"
        );
    }
    let stock = recipe
        .inputs
        .get(1)
        .map(|i| {
            i.choices
                .iter()
                .map(|c| {
                    serde_json::to_string(&MatchCase {
                        id: c.id.clone(),
                        rule: c.rule.clone(),
                    })
                })
                .collect::<Result<BTreeSet<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    ensure!(
        stock == ball_choices,
        "SAG reserve differs from registered ball predicates"
    );
    cases(blocked)?;
    cases(ore_blocked)?;
    for (index, output) in recipe.outputs.iter().enumerate() {
        ensure!(
            output.kind == Kind::Item
                && output.slot == index as u32
                && matches!(output.role, OutputRole::Result)
                && matches!(output.quantity, Some(Quantity::Grinding { .. })),
            "invalid SAG shared output"
        );
        super::quantity_bounds(recipe, output)?;
    }
    Ok(())
}
