//! GT 5.09.54.265 native evidence. Machine locks, output callbacks, matching cache
//! history, cleanroom filters, chance multipliers and batching remain unresolved.
use super::{Chance, Consumption, Edit, Kind, Match, Process, Recipe};
use crate::identity::integer;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const ARCHIVE_SHA256: &str = "b969e11625a0298416a75b6e39703da795233edb71cb66f25fb9bbf654d75528";

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GtNative {
    pub archive_sha256: String,
    pub native_recipe_class: String,
    pub nbt_sensitive: bool,
    pub fake: bool,
    pub enabled: bool,
    /// Array position is the native slot, including null holes.
    pub item_inputs: Vec<Option<GtStack>>,
    pub item_outputs: Vec<Option<GtStack>>,
    pub fluid_inputs: Vec<Option<GtStack>>,
    pub fluid_outputs: Vec<Option<GtStack>>,
    pub item_alternatives: Option<Vec<Option<Vec<Option<GtStack>>>>>,
    /// Null falls back to the representative; nonnull empty does not.
    pub fluid_alternatives: Option<Vec<Option<Vec<Option<GtStack>>>>>,
    /// Captured raw values, not reconstructed getter results. Null/empty/short differ.
    pub input_chances: Option<Vec<i32>>,
    pub output_chances: Option<Vec<i32>>,
    pub fluid_input_chances: Option<Vec<i32>>,
    pub fluid_output_chances: Option<Vec<i32>>,
    pub policy: GtPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GtStack {
    pub id: String,
    /// Exact signed native int; zero and invalid negative evidence stay distinct.
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GtPolicy {
    pub item_consumption: GtItemConsumption,
    pub fluid_consumption: GtFluidConsumption,
    /// The default helper's sampler, not a promise that every machine uses it.
    pub sampler: GtSampler,
    pub execution: GtExecution,
    pub projection: GtProjection,
}

macro_rules! policy_enum {
    ($name:ident { $($variant:ident),+ }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "camelCase")]
        pub enum $name { $($variant),+ }
    };
}
policy_enum!(GtItemConsumption { NativeMatchingGroups });
policy_enum!(GtFluidConsumption { NativeSlots });
policy_enum!(GtSampler { GtIntegralChancedMultiplier });
policy_enum!(GtExecution { UnresolvedMachineContext, ReferenceOnly });
policy_enum!(GtProjection { NativeSlots, SpecializedProcess });

pub fn effective_chance(raw: &Option<Vec<i32>>, slot: usize) -> i32 {
    raw.as_ref().and_then(|values| values.get(slot)).copied().unwrap_or(10000)
}

fn probability(chance: &Chance, native: i32) -> Result<()> {
    super::check::chance(chance)?;
    let numerator = integer(&chance.numerator, 0, i64::MAX)?;
    let denominator = integer(&chance.denominator, 1, i64::MAX)?;
    ensure!(i128::from(numerator) * 10000 == i128::from(denominator) * i128::from(native),
        "GT projected chance differs from its native slot");
    Ok(())
}

fn consumption(choice: &super::Choice, native: i32, amount: &str) -> Result<()> {
    ensure!(choice.amount == amount, "GT admission quantity was replaced by expected consumption");
    if amount == "0" {
        ensure!(matches!(choice.consume, Consumption::Keep), "GT zero fluid cost needs its native policy");
    } else {
        let Consumption::Chance { chance } = &choice.consume else {
            anyhow::bail!("GT positive input requires grouped/slot native chance consumption");
        };
        probability(chance, native)?;
    }
    Ok(())
}

pub(super) fn validate(recipe: &Recipe, reference: &impl Fn(Kind, &str) -> Result<()>) -> Result<()> {
    let Some(native) = &recipe.gt_native else { return Ok(()); };
    ensure!(native.archive_sha256 == ARCHIVE_SHA256, "GT native facts target another archive");
    ensure!(!native.native_recipe_class.is_empty() && native.native_recipe_class.len() <= 512,
        "GT native recipe class is missing or unbounded");
    ensure!(matches!(recipe.source.handler.as_str(), "gregtech.nei.GTNEIDefaultHandler"
        | "bartworks.neiHandler.BioLabNEIHandler" | "bartworks.neiHandler.BioVatNEIHandler"),
        "GT native facts need an admitted handler");
    let mut budget = 0usize;
    let mut stacks = |kind: Kind, values: &[Option<GtStack>]| -> Result<()> {
        budget += values.len();
        ensure!(budget <= 65536, "GT native slot/alternative budget exceeded");
        for stack in values.iter().flatten() {
            reference(kind, &stack.id)?;
            integer(&stack.amount, i64::from(i32::MIN), i64::from(i32::MAX))?;
        }
        Ok(())
    };
    stacks(Kind::Item, &native.item_inputs)?;
    stacks(Kind::Item, &native.item_outputs)?;
    stacks(Kind::Fluid, &native.fluid_inputs)?;
    stacks(Kind::Fluid, &native.fluid_outputs)?;
    for (kind, alternatives) in [(Kind::Item, &native.item_alternatives), (Kind::Fluid, &native.fluid_alternatives)] {
        if let Some(alternatives) = alternatives {
            ensure!(alternatives.len() <= 65536, "GT alternative slot budget exceeded");
            for values in alternatives.iter().flatten() { stacks(kind, values)?; }
        }
    }
    let reference_only = native.policy.execution == GtExecution::ReferenceOnly;
    for raw in [&native.input_chances, &native.output_chances, &native.fluid_input_chances, &native.fluid_output_chances] {
        if let Some(values) = raw {
            ensure!(values.len() <= 65536, "GT raw chance budget exceeded");
            ensure!(values.iter().all(|value| (0..=10000).contains(value)),
                "malformed raw GT chance cannot form a probability projection");
        }
    }
    if reference_only {
        ensure!(native.fake || !native.enabled, "GT reference-only policy needs a fake or disabled native recipe");
    } else {
        ensure!(native.enabled, "disabled GT recipe cannot form an executable projection");
    }
    if native.policy.projection == GtProjection::SpecializedProcess {
        let harmony = matches!(recipe.process, Some(Process::Harmony { .. }));
        let scan = recipe.source.key.starts_with("gt.recipe.scanner/")
            && recipe.inputs.iter().any(|input| input.kind == Kind::Item && input.choices.iter().all(|choice|
                matches!(choice.rule, Match::Member { .. }) && matches!(choice.consume, Consumption::Stack)))
            && recipe.outputs.iter().any(|output| output.change.as_ref().is_some_and(|change| matches!(change.action, Edit::Analyze)));
        ensure!(harmony || scan, "GT specialized projection requires the existing Harmony or scanner contract");
        return Ok(());
    }
    ensure!(!native.fake || reference_only, "fake GT display is not a native executable recipe");
    let input = |kind, slot| recipe.inputs.iter().find(|input| input.kind == kind && input.slot as usize == slot);
    let mut expected_inputs = 0usize;
    for (slot, stack) in native.item_inputs.iter().enumerate() {
        let Some(stack) = stack else { continue; };
        let projected = input(Kind::Item, slot).context("GT native item input slot missing")?;
        expected_inputs += 1;
        let alternatives = native.item_alternatives.as_ref().and_then(|values| values.get(slot))
            .and_then(Option::as_ref).filter(|values| !values.is_empty());
        let amounts: Vec<&str> = match alternatives {
            Some(values) => values.iter().map(|value| Ok(value.as_ref().context("GT null item alternative")?.amount.as_str()))
                .collect::<Result<_>>()?,
            None => vec![stack.amount.as_str()],
        };
        for choice in &projected.choices {
            let zero = amounts.iter().all(|amount| *amount == "0");
            if zero {
                ensure!(choice.amount == "1" && matches!(choice.consume, Consumption::Keep),
                    "GT zero-size item presence projection changed");
            } else {
                ensure!(amounts.contains(&choice.amount.as_str()), "GT native item admission amount changed");
                consumption(choice, effective_chance(&native.input_chances, slot), &choice.amount)?;
            }
        }
    }
    for (slot, stack) in native.fluid_inputs.iter().enumerate() {
        let stack = stack.as_ref().context("GT null native fluid representative cannot be executed")?;
        let alternatives = native.fluid_alternatives.as_ref().and_then(|values| values.get(slot)).and_then(Option::as_ref);
        if alternatives.is_some_and(Vec::is_empty) { continue; }
        let projected = input(Kind::Fluid, slot).context("GT native fluid input slot missing")?;
        expected_inputs += 1;
        let choices: Vec<&GtStack> = if stack.amount == "0" { vec![stack] } else {
            match alternatives {
                Some(values) => values.iter().map(|value| value.as_ref().context("GT null fluid alternative cannot be executed")).collect::<Result<_>>()?,
                None => vec![stack],
            }
        };
        ensure!(projected.choices.len() == choices.len(), "GT fluid alternatives lost native order or cardinality");
        for (choice, stack) in projected.choices.iter().zip(choices) {
            ensure!(choice.id == stack.id, "GT fluid alternative identity/order changed");
            integer(&stack.amount, 0, i64::from(i32::MAX))?;
            consumption(choice, effective_chance(&native.fluid_input_chances, slot), &stack.amount)?;
        }
    }
    let special = recipe.inputs.iter().filter(|input| input.kind == Kind::Item && input.slot == 65535).count();
    ensure!(recipe.inputs.len() == expected_inputs + special, "GT projection contains nonnative input slots");
    let mut expected_outputs = 0usize;
    for (kind, values, raw) in [(Kind::Item, &native.item_outputs, &native.output_chances),
            (Kind::Fluid, &native.fluid_outputs, &native.fluid_output_chances)] {
        for (slot, stack) in values.iter().enumerate() {
            let Some(stack) = stack else { continue; };
            expected_outputs += 1;
            let output = recipe.outputs.iter().find(|output| output.kind == kind && output.slot as usize == slot)
                .context("GT native output slot missing")?;
            ensure!(output.id == stack.id, "GT output identity is not bound to its native slot");
            if let Some(amount) = &output.amount {
                ensure!(amount == &stack.amount, "GT native whole-slot output amount changed");
            }
            probability(&output.chance, effective_chance(raw, slot))?;
        }
    }
    ensure!(recipe.outputs.len() == expected_outputs, "GT projection contains nonnative output slots");
    Ok(())
}
