use super::{Consumption, Kind, Output, Recipe};
use crate::identity::integer;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Correlated amounts, evaluated without arbitrary sampling.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Quantity {
    Draw {
        input: u32,
        after: Vec<u32>,
        limit: String,
    },
    Remainder {
        input: u32,
        after: Vec<u32>,
    },
    Branch {
        group: String,
        branch: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        condition: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        threshold: Option<String>,
        nominal: String,
    },
    Potential {
        stat: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        condition: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sample: Option<String>,
        nominal: String,
    },
}

/// Exact attainable bounds; the rules, rather than these bounds, retain correlation.
pub fn quantity_bounds(recipe: &Recipe, output: &Output) -> Result<(i64, i64)> {
    let Some(rule) = &output.quantity else {
        let value = integer(
            output
                .amount
                .as_deref()
                .context("fixed output lacks amount")?,
            1,
            i64::MAX,
        )?;
        return Ok((value, value));
    };
    ensure!(
        output.amount.is_none() && output.change.is_none(),
        "computed output has a fixed amount or item change"
    );
    ensure!(
        output.chance.numerator == "1" && output.chance.denominator == "1",
        "computed output cannot carry an independent probability"
    );
    match rule {
        Quantity::Branch {
            group,
            nominal,
            ..
        } => {
            let max_val = integer(nominal, 1, i64::MAX)?;
            let mut seen_branches = BTreeSet::new();
            for candidate in &recipe.outputs {
                if let Some(Quantity::Branch {
                    group: g,
                    branch: b,
                    ..
                }) = &candidate.quantity
                {
                    if g == group {
                        ensure!(seen_branches.insert(b), "duplicate branch in quantity group");
                    }
                }
            }
            Ok((0, max_val))
        }
        Quantity::Potential { nominal, sample, .. } => {
            let max_val = integer(nominal, 1, i64::MAX)?;
            if let Some(s) = sample {
                let sample_val = integer(s, 0, i64::MAX)?;
                ensure!(
                    sample_val <= max_val,
                    "potential sample {} exceeds nominal upper bound {}",
                    sample_val,
                    max_val
                );
            }
            Ok((0, max_val))
        }
        Quantity::Draw { .. } | Quantity::Remainder { .. } => {
            ensure!(
                output.kind == Kind::Fluid,
                "computed quantities require fluid outputs"
            );
            let (input, after) = match rule {
                Quantity::Draw { input, after, .. } | Quantity::Remainder { input, after } => {
                    (*input, after)
                }
                _ => unreachable!(),
            };
            ensure!(
                after.len() <= 128,
                "quantity draw chain exceeds 128 entries"
            );
            let mut draws = BTreeSet::new();
            let mut remainder = None;
            for candidate in &recipe.outputs {
                match &candidate.quantity {
                    Some(Quantity::Draw { input: group, .. }) if *group == input => {
                        ensure!(
                            candidate.kind == Kind::Fluid && draws.insert(candidate.slot),
                            "quantity group repeats a draw slot"
                        );
                    }
                    Some(Quantity::Remainder {
                        input: group,
                        after: closing,
                    }) if *group == input => {
                        ensure!(
                            candidate.kind == Kind::Fluid && remainder.is_none(),
                            "quantity group has multiple remainders"
                        );
                        remainder = Some(closing);
                    }
                    _ => {}
                }
            }
            let closing = remainder.context("quantity group lacks its remainder")?;
            ensure!(
                closing.len() == draws.len()
                    && closing.iter().copied().collect::<BTreeSet<_>>() == draws,
                "quantity remainder does not include every draw"
            );
            let source = recipe
                .inputs
                .iter()
                .find(|row| row.kind == Kind::Fluid && row.slot == input)
                .context("quantity refers to a missing fluid input")?;
            ensure!(
                source.choices.len() == 1
                    && matches!(source.choices[0].consume, Consumption::Consume)
                    && source.choices[0].returns.is_empty(),
                "quantity requires one consumed fluid input alternative"
            );
            let initial = integer(&source.choices[0].amount, 1, i64::MAX)?;
            let (mut low, mut high) = (initial, initial);
            let mut used = BTreeSet::new();
            for (index, slot) in after.iter().enumerate() {
                ensure!(
                    *slot != output.slot && used.insert(*slot),
                    "quantity has a cyclic or repeated draw"
                );
                let prior = recipe
                    .outputs
                    .iter()
                    .find(|row| row.kind == Kind::Fluid && row.slot == *slot)
                    .context("quantity refers to a missing preceding output")?;
                let Some(Quantity::Draw {
                    input: prior_input,
                    after: prefix,
                    limit,
                }) = &prior.quantity
                else {
                    anyhow::bail!("quantity depends on an output that is not a draw");
                };
                ensure!(
                    *prior_input == input && prefix == &after[..index],
                    "quantity draw order or input is inconsistent"
                );
                ensure!(
                    prior.amount.is_none()
                        && prior.change.is_none()
                        && prior.chance.numerator == "1"
                        && prior.chance.denominator == "1",
                    "preceding draw has incompatible fixed fields"
                );
                let limit = integer(limit, 1, i64::MAX)?;
                ensure!(low > 1, "quantity draw can exhaust its input budget");
                low = (low - limit).max(1);
                high -= 1;
            }
            match rule {
                Quantity::Draw { limit, .. } => {
                    let limit = integer(limit, 1, i64::MAX)?;
                    ensure!(low > 1, "quantity draw can exhaust its input budget");
                    Ok((1, limit.min(high - 1)))
                }
                Quantity::Remainder { .. } => Ok((low, high)),
                _ => unreachable!(),
            }
        }
    }
}
