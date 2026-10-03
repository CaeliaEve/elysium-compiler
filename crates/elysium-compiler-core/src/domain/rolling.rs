//! Native Railcraft grid selection and machine gates; see docs/rolling.md.
use super::{Consumption, Grid, Item, Kind, Match, MatchCase, OutputRole, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Ordered requirements. None grid means native greedy shapeless matching in
/// offered row-major cell order; a grid permits translations and its mirror flag.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CraftingSelector {
    pub grid: Option<Grid>,
    pub inputs: Vec<Vec<MatchCase>>,
}

fn grid(grid: Option<&Grid>, inputs: usize) -> Result<()> {
    if let Some(g) = grid {
        ensure!(
            (1..=3).contains(&g.width)
                && (1..=3).contains(&g.height)
                && g.cells.len() == (g.width * g.height) as usize,
            "invalid rolling grid dimensions"
        );
        let cells: Vec<_> = g.cells.iter().flatten().copied().collect();
        ensure!(
            cells.len() == inputs
                && cells.iter().copied().collect::<BTreeSet<_>>() == (0..inputs as u32).collect(),
            "rolling grid must use each ordered input once"
        );
    }
    Ok(())
}

fn rule(rule: &Match) -> Result<()> {
    ensure!(
        matches!(rule, Match::Wildcard { nbt: true, .. }),
        "rolling requires native item/meta matching with ignored NBT"
    );
    Ok(())
}

pub(super) fn validate(
    recipe: &Recipe,
    earlier: &[CraftingSelector],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    ensure!(
        recipe.magic.is_none()
            && recipe.duration.is_none()
            && recipe.energy.is_none()
            && (1..=9).contains(&recipe.inputs.len())
            && recipe.outputs.len() == 1
            && earlier.len() <= 262144,
        "invalid rolling recipe shape"
    );
    grid(recipe.grid.as_ref(), recipe.inputs.len())?;
    for (slot, input) in recipe.inputs.iter().enumerate() {
        ensure!(
            input.kind == Kind::Item && input.slot == slot as u32 && !input.choices.is_empty(),
            "invalid rolling input slot"
        );
        for choice in &input.choices {
            ensure!(
                choice.amount == "1"
                    && matches!(choice.consume, Consumption::Consume)
                    && choice.returns.is_empty(),
                "rolling consumes one per occupied cell and never returns crafting containers"
            );
            rule(&choice.rule)?;
        }
    }
    let output = &recipe.outputs[0];
    ensure!(
        output.slot == 0
            && output.kind == Kind::Item
            && output.amount.is_some()
            && output.quantity.is_none()
            && output.change.is_none()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "rolling result must be fixed and unconditional after selection"
    );
    let mut count = 0;
    for prior in earlier {
        ensure!(
            (1..=9).contains(&prior.inputs.len()),
            "invalid prior rolling input count"
        );
        grid(prior.grid.as_ref(), prior.inputs.len())?;
        for choices in &prior.inputs {
            ensure!(
                !choices.is_empty() && choices.len() <= 65536,
                "empty or oversized prior rolling choices"
            );
            count += choices.len();
            ensure!(count <= 1_048_576, "rolling selector budget exceeded");
            for choice in choices {
                rule(&choice.rule)?;
                items
                    .get(choice.id.as_str())
                    .context("missing earlier rolling template")?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::Process, source::Source};

    #[test]
    fn rolling_does_not_invent_returns_or_lose_prior_grid() {
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = super::super::Domain::load(&source).unwrap();
        let items = domain.items.iter().map(|i| (i.id.as_str(), i)).collect();
        let recipes: Vec<_> = domain
            .recipes
            .iter()
            .filter(|r| matches!(r.process, Some(Process::Rolling { .. })))
            .collect();
        assert_eq!(recipes.len(), 2);
        for recipe in recipes {
            let Some(Process::Rolling { earlier, .. }) = &recipe.process else {
                unreachable!()
            };
            validate(recipe, earlier, &items).unwrap();
            let mut wrong = recipe.clone();
            wrong.inputs[0].choices[0].amount = "2".into();
            assert!(validate(&wrong, earlier, &items).is_err());
            wrong = recipe.clone();
            wrong.inputs[0].choices[0].consume = Consumption::Keep;
            assert!(validate(&wrong, earlier, &items).is_err());
            let mut missing = earlier.clone();
            missing[0].inputs[0][0].id = "missing".into();
            assert!(validate(recipe, &missing, &items).is_err());
            let mut wrong_grid = earlier.clone();
            wrong_grid[0].grid.as_mut().unwrap().cells = vec![Some(0), Some(0)];
            assert!(validate(recipe, &wrong_grid, &items).is_err());
        }
    }
}
