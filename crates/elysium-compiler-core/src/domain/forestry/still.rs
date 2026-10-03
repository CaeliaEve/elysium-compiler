//! Native Still selection, input reservation and lossy tank completion.
use super::ForestryFluid;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StillRecipe {
    pub cycles: i32,
    pub input: ForestryFluid,
    pub output: ForestryFluid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StillState {
    pub retained: Option<StillRecipe>,
    pub resource: Option<ForestryFluid>,
    pub product: Option<ForestryFluid>,
    pub buffer: Option<ForestryFluid>,
    /// The native tank's copied filter, which may predate recipe registration changes.
    pub filters: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StillObservation {
    pub selected: Option<StillRecipe>,
    pub has_work: bool,
    pub completed: bool,
    pub reserved: Option<ForestryFluid>,
    pub resource: Option<ForestryFluid>,
    pub product: Option<ForestryFluid>,
    pub buffer: Option<ForestryFluid>,
}

/// One hasWork observation followed by workCycle only when ready. The clock,
/// external filling/draining and container packaging are separate operations.
pub fn observe(recipes: &[StillRecipe], state: &StillState) -> Result<StillObservation> {
    ensure!(
        recipes.len() <= 262144 && state.filters.len() <= 65536,
        "Still rules exceed their budget"
    );
    for recipe in recipes.iter().chain(state.retained.iter()) {
        recipe.input.validate()?;
        recipe.output.validate()?;
    }
    for fluid in state
        .resource
        .iter()
        .chain(state.product.iter())
        .chain(state.buffer.iter())
    {
        fluid.validate()?;
    }
    for name in &state.filters {
        super::registry_name(name)?;
    }
    let liquid = state.buffer.as_ref().or(state.resource.as_ref());
    let matches = |recipe: &&StillRecipe| {
        liquid.is_some_and(|fluid| {
            fluid.registry == recipe.input.registry
                && super::work::tags_equal(&fluid.nbt, &recipe.input.nbt)
                && fluid.amount >= recipe.input.amount
        })
    };
    let selected = state
        .retained
        .as_ref()
        .filter(matches)
        .or_else(|| recipes.iter().find(matches));
    let mut result = StillObservation {
        selected: selected.cloned(),
        has_work: false,
        completed: false,
        reserved: state.buffer.clone(),
        resource: state.resource.clone(),
        product: state.product.clone(),
        buffer: state.buffer.clone(),
    };
    let Some(recipe) = selected else {
        return Ok(result);
    };
    let fill = |output: &ForestryFluid| {
        if state.filters.contains(&output.registry) {
            super::work::fill_amount(state.product.as_ref(), output)
        } else {
            0
        }
    };
    // Native preflight checks a unit, not the eventual batch. Reservation is
    // performed even when this space gate is false, and before power is checked.
    let space = fill(&recipe.output) == recipe.output.amount;
    let mut resource = true;
    if state.buffer.is_none() {
        let amount = recipe.cycles.wrapping_mul(recipe.input.amount);
        resource = amount > 0 && result.resource.as_ref().is_some_and(|f| f.amount >= amount);
        if resource {
            let mut reserved = recipe.input.clone();
            reserved.amount = amount;
            result.reserved = Some(reserved.clone());
            result.buffer = Some(reserved);
            let tank = result.resource.as_mut().unwrap();
            tank.amount -= amount;
            if tank.amount == 0 {
                result.resource = None;
            }
        }
    }
    result.has_work = resource && space;
    if result.has_work {
        let mut batch = recipe.output.clone();
        batch.amount = recipe.cycles.wrapping_mul(batch.amount);
        let accepted = fill(&batch);
        if accepted > 0 {
            if let Some(product) = &mut result.product {
                product.amount = product.amount.wrapping_add(accepted);
            } else {
                batch.amount = accepted;
                result.product = Some(batch);
            }
        }
        // workCycle ignores the amount accepted, clears the reservation and
        // reports success even if no output was stored.
        result.buffer = None;
        result.completed = true;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_still_selection_reserves_input_even_when_output_is_blocked() {
        #[derive(Deserialize)]
        struct Case {
            name: String,
            recipes: Vec<StillRecipe>,
            state: StillState,
            after: StillObservation,
        }
        #[derive(Deserialize)]
        struct Fixture {
            cases: Vec<Case>,
        }
        let fixture: Fixture = serde_json::from_str(include_str!(
            "../../../../../contracts/fixtures/forestry-still-observations.json"
        ))
        .unwrap();
        for case in fixture.cases {
            assert_eq!(
                observe(&case.recipes, &case.state).unwrap(),
                case.after,
                "{}",
                case.name
            );
        }
    }
}
