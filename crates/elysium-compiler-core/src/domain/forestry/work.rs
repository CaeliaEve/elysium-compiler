//! Squeezer's output-space gate and native powered-machine arithmetic.
use super::{chance, contains_sets, observe_stock, ForestryFluid, ForestryStack, SqueezerRecipe};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkState {
    pub stock: Vec<Option<ForestryStack>>,
    pub tank: Option<ForestryFluid>,
    pub remnant: Option<ForestryStack>,
    pub slot_limit: i32,
    pub stackable: bool,
    /// The observed Java Random.nextFloat value, as raw float bits.
    pub draw: String,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkObservation {
    pub has_work: bool,
    pub completed: bool,
    pub rolled: bool,
    pub remaining: Vec<Option<ForestryStack>>,
    pub tank_after: Option<ForestryFluid>,
    pub remnant_after: Option<ForestryStack>,
}
/// Apply the selected recipe's preflight and completion. Selection/retention is
/// a separate step; this does not run the machine's power clock or package fluid.
pub fn observe_work(recipe: &SqueezerRecipe, state: &WorkState) -> Result<WorkObservation> {
    recipe.validate()?;
    ensure!(state.stock.len() <= 9, "Squeezer exceeds nine input slots");
    for stack in state.stock.iter().flatten().chain(state.remnant.iter()) {
        stack.validate()?;
    }
    if let Some(tank) = &state.tank {
        tank.validate()?;
    }
    let draw = chance(&state.draw)?;
    ensure!(
        draw.is_finite() && (0.0..1.0).contains(&draw) && (draw * 16_777_216.0).fract() == 0.0,
        "Invalid native nextFloat observation"
    );
    let has_recipe = contains_sets(&recipe.requirements, &state.stock, true) > 0;
    let mut observed = WorkObservation {
        has_work: false,
        completed: false,
        rolled: false,
        remaining: state.stock.clone(),
        tank_after: state.tank.clone(),
        remnant_after: state.remnant.clone(),
    };
    observed.remaining.resize(9, None);
    if !has_recipe {
        return Ok(observed);
    }
    let output = recipe
        .fluid
        .as_ref()
        .context("Native Squeezer canFill would dereference a null fluid output")?;
    let has_work = fill_amount(state.tank.as_ref(), output) == output.amount
        && recipe.remnant.as_ref().is_none_or(|remnant| {
            add_remnant(
                state.remnant.as_ref(),
                remnant,
                state.slot_limit,
                state.stackable,
            )
            .0
        });
    observed.has_work = has_work;
    if !has_work {
        return Ok(observed);
    }
    let allocation = observe_stock(&recipe.requirements, &state.stock)?;
    observed.remaining = allocation.remaining;
    observed.completed = allocation.removed;
    if observed.completed {
        if output.amount > 0 {
            if let Some(tank) = &mut observed.tank_after {
                tank.amount = tank.amount.wrapping_add(output.amount);
            } else {
                observed.tank_after = Some(output.clone());
            }
        }
        if let Some(remnant) = &recipe.remnant {
            observed.rolled = true;
            if draw < chance(&recipe.chance)? {
                observed.remnant_after = add_remnant(
                    state.remnant.as_ref(),
                    remnant,
                    state.slot_limit,
                    state.stackable,
                )
                .1;
            }
        }
    }
    Ok(observed)
}

pub(super) fn fill_amount(tank: Option<&ForestryFluid>, output: &ForestryFluid) -> i32 {
    if output.amount <= 0 {
        return 0;
    }
    match tank {
        None => 10000.min(output.amount),
        Some(tank) if tank.registry == output.registry && tags_equal(&tank.nbt, &output.nbt) => {
            10000_i32.wrapping_sub(tank.amount).min(output.amount)
        }
        _ => 0,
    }
}
pub(super) fn tags_equal(
    left: &Option<crate::identity::Nbt>,
    right: &Option<crate::identity::Nbt>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(a), Some(b)) => super::super::matching::native_tag(a, b, true),
        _ => false,
    }
}
fn add_remnant(
    current: Option<&ForestryStack>,
    output: &ForestryStack,
    limit: i32,
    stackable: bool,
) -> (bool, Option<ForestryStack>) {
    if let Some(current) = current {
        if stackable && current.same_type(output) && current.same_tags(output) {
            let space = limit.wrapping_sub(current.amount);
            if space > 0 && space >= output.amount {
                let mut updated = current.clone();
                updated.amount = updated.amount.wrapping_add(output.amount);
                return (true, Some(updated));
            }
        }
        // Zero remnants pass the native equality check without changing a slot.
        return (output.amount == 0, Some(current.clone()));
    }
    if output.amount <= 0 {
        return (output.amount == 0, None);
    }
    // InventoryUtil assigns the entire remainder to an empty slot, including
    // oversized stacks, after the inventory setter has run.
    (true, Some(output.clone()))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PowerState {
    pub time: i32,
    pub difficulty: String,
    pub speed: String,
    pub power: String,
    pub stored: i32,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PowerObservation {
    pub ticks: i32,
    pub energy: i32,
    pub capacity: i32,
    pub consumed: bool,
    pub after: i32,
}
pub fn observe_power(state: &PowerState) -> Result<PowerObservation> {
    let difficulty = chance(&state.difficulty)?;
    let speed = chance(&state.speed)?;
    let power = chance(&state.power)?;
    let ticks = java_round(state.time as f32 / speed);
    let scaled = java_round(state.time.wrapping_mul(200) as f32 * difficulty);
    let energy = java_round(scaled as f32 * power);
    let capacity = java_round(5000.0 * difficulty);
    let per_step = (energy as f32 / ticks as f32).ceil() as i32;
    let consumed = state.stored >= per_step;
    let after = if consumed {
        let value = state.stored.wrapping_sub(per_step);
        if value > capacity {
            capacity
        } else {
            value.max(0)
        }
    } else {
        state.stored
    };
    Ok(PowerObservation {
        ticks,
        energy,
        capacity,
        consumed,
        after,
    })
}

fn java_round(value: f32) -> i32 {
    (f64::from(value) + 0.5).floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    #[test]
    fn work_matches_native_tile_preflight_and_completion_helpers() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../../contracts/fixtures/forestry-work-observations.json"
        ))
        .unwrap();
        for value in fixture["work"].as_array().unwrap() {
            let mut input = value.clone();
            for key in [
                "name",
                "recipe",
                "hasWork",
                "completed",
                "rolled",
                "remaining",
                "tankAfter",
                "remnantAfter",
            ] {
                input.as_object_mut().unwrap().remove(key);
            }
            let state: WorkState = serde_json::from_value(input).unwrap();
            let recipe: SqueezerRecipe = serde_json::from_value(value["recipe"].clone()).unwrap();
            let expected: WorkObservation = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(
                observe_work(&recipe, &state).unwrap(),
                expected,
                "{}",
                value["name"]
            );
        }
    }
    #[test]
    fn power_matches_native_rounding_overflow_and_energy_storage() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../../contracts/fixtures/forestry-work-observations.json"
        ))
        .unwrap();
        for value in fixture["power"].as_array().unwrap() {
            let state: PowerState = serde_json::from_value(value.clone()).unwrap();
            let expected: PowerObservation = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(
                observe_power(&state).unwrap(),
                expected,
                "{}",
                value["name"]
            );
        }
    }
}
