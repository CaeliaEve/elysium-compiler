//! Cross-check display rows against a single validated native rule context.
use super::SqueezerProgram;
use crate::domain::{
    Category, Consumption, Fluid, Item, Kind, Match, OutputRole, Process, Quantity, Recipe,
};
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;

pub fn validate(
    recipe: &Recipe,
    programs: &BTreeMap<String, SqueezerProgram>,
    category: &Category,
    items: &BTreeMap<&str, &Item>,
    fluids: &BTreeMap<&str, &Fluid>,
) -> Result<()> {
    let Some(Process::ForestrySqueezer {
        program,
        selector,
        time,
        chance,
    }) = &recipe.process
    else {
        return Ok(());
    };
    ensure!(
        category.program.as_ref() == Some(program),
        "Squeezer process and category use different programs"
    );
    let rules = programs.get(program).context("Missing Squeezer program")?;
    let native = rules.entry(selector)?;
    ensure!(
        *time == native.time,
        "Squeezer work steps differ from the native rule"
    );
    let threshold = super::chance(&native.chance)?;
    let mut n = if threshold.is_nan() || threshold <= 0.0 {
        0
    } else if threshold >= 1.0 {
        16777216
    } else {
        (f64::from(threshold) * 16777216.0).ceil() as u64
    };
    let mut d = 16777216;
    let (mut a, mut b) = (n, d);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    n /= a;
    d /= a;
    ensure!(
        chance.numerator == n.to_string() && chance.denominator == d.to_string(),
        "Squeezer remnant chance differs from native nextFloat"
    );
    ensure!(
        recipe.inputs.len() == native.requirements.iter().flatten().count(),
        "Squeezer requirement slots were omitted or added"
    );
    for (slot, requirement) in native.requirements.iter().enumerate() {
        let Some(requirement) = requirement else {
            continue;
        };
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.slot == slot as u32 && i.kind == Kind::Item)
            .context("Missing native squeezer requirement")?;
        ensure!(
            !input.choices.is_empty(),
            "Squeezer requirement has no display samples"
        );
        for choice in &input.choices {
            ensure!(
                choice.amount == requirement.amount.to_string()
                    && matches!(choice.rule, Match::Forestry)
                    && matches!(choice.consume, Consumption::Allocated)
                    && choice.returns.is_empty(),
                "Squeezer requirement changed native count or allocation"
            );
            let item = items
                .get(choice.id.as_str())
                .context("Missing Squeezer input sample")?;
            ensure!(
                item.registry == requirement.registry
                    && (requirement.meta == 32767 || item.meta == requirement.meta)
                    && item.nbt == requirement.nbt,
                "Squeezer input sample differs from native requirement"
            );
        }
    }
    ensure!(
        recipe.outputs.len()
            == usize::from(native.fluid.is_some()) + usize::from(native.remnant.is_some()),
        "Squeezer native output fields were omitted or added"
    );
    for output in &recipe.outputs {
        ensure!(
            output.slot == 0
                && matches!(output.role, OutputRole::Result)
                && output.amount.is_none()
                && output.change.is_none()
                && output.chance.numerator == "1"
                && output.chance.denominator == "1",
            "Squeezer output must retain shared completion semantics"
        );
        let amount = match output.kind {
            Kind::Item => {
                let expected = native.remnant.as_ref().context("Squeezer has no remnant")?;
                let item = items
                    .get(output.id.as_str())
                    .context("Missing Squeezer remnant")?;
                ensure!(
                    item.registry == expected.registry
                        && item.meta == expected.meta
                        && item.nbt == expected.nbt,
                    "Squeezer remnant identity changed"
                );
                expected.amount
            }
            Kind::Fluid => {
                let expected = native.fluid.as_ref().context("Squeezer has no fluid")?;
                let fluid = fluids
                    .get(output.id.as_str())
                    .context("Missing Squeezer output fluid")?;
                ensure!(
                    fluid.registry == expected.registry && fluid.nbt == expected.nbt,
                    "Squeezer output fluid identity changed"
                );
                expected.amount
            }
        };
        ensure!(
            matches!(&output.quantity,Some(Quantity::Squeezer {nominal}) if *nominal==amount.to_string()),
            "Squeezer output changed its signed native parameter"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    #[test]
    fn native_adapter_rows_validate_and_reject_divergent_summary_and_samples() {
        for data in [
            include_str!("../../../../../contracts/fixtures/squeezer-adapter-records.json"),
            include_str!("../../../../../contracts/fixtures/squeezer-signed-adapter-records.json"),
        ] {
            check_native_adapter_rows(serde_json::from_str(data).unwrap());
        }
    }
    fn check_native_adapter_rows(observed: Value) {
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = crate::domain::Domain::load(&source).unwrap();
        let items: Vec<Item> = observed["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|raw| {
                let mut value = serde_json::to_value(&domain.items[0]).unwrap();
                for key in ["id", "registry", "meta", "nbt"] {
                    value[key] = raw[key].clone();
                }
                value["tags"] = raw["ores"].clone();
                serde_json::from_value(value).unwrap()
            })
            .collect();
        let fluids: Vec<Fluid> = observed["fluids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|raw| {
                let mut value = serde_json::to_value(&domain.fluids[0]).unwrap();
                for key in ["id", "registry", "nbt"] {
                    value[key] = raw[key].clone();
                }
                serde_json::from_value(value).unwrap()
            })
            .collect();
        let item_refs = items.iter().map(|item| (item.id.as_str(), item)).collect();
        let fluid_refs = fluids
            .iter()
            .map(|fluid| (fluid.id.as_str(), fluid))
            .collect();
        let id = observed["programId"].as_str().unwrap().to_string();
        let programs = BTreeMap::from([(
            id.clone(),
            serde_json::from_value::<SqueezerProgram>(observed["program"].clone()).unwrap(),
        )]);
        let mut category = domain.categories[0].clone();
        category.program = Some(id);
        let check = |value: Value| -> Result<()> {
            let recipe: Recipe = serde_json::from_value(value)?;
            crate::domain::process::validate(&recipe, &fluid_refs, &item_refs, &[])?;
            validate(&recipe, &programs, &category, &item_refs, &fluid_refs)?;
            for output in &recipe.outputs {
                let bounds = crate::domain::quantity_bounds(&recipe, output)?;
                if output.kind == Kind::Item {
                    let mut never = recipe.clone();
                    if let Some(Process::ForestrySqueezer { chance, .. }) = &mut never.process {
                        chance.numerator = "0".into();
                        chance.denominator = "1".into();
                    }
                    assert_eq!(crate::domain::quantity_bounds(&never, output)?, (0, 0));
                }
                ensure!(bounds.0 <= bounds.1, "Reversed Squeezer bounds");
            }
            Ok(())
        };
        for row in observed["recipes"].as_array().unwrap() {
            check(row.clone()).unwrap();
        }
        let row = observed["recipes"][0].clone();
        for (path, bad) in [
            ("/process/program", json!("program_missing")),
            ("/process/time", json!(999)),
            ("/process/chance/numerator", json!("7")),
            ("/process/selector/index", json!(999)),
            ("/inputs/0/choices/0/amount", json!("200")),
            ("/inputs/0/choices/0/consume", json!({"kind":"consume"})),
            ("/inputs/0/choices/0/rule", json!({"kind":"exact"})),
            ("/outputs/0/quantity/nominal", json!("999")),
            ("/outputs/0/amount", json!("100")),
            ("/duration", json!("10")),
            ("/process", Value::Null),
        ] {
            let mut changed = row.clone();
            *changed.pointer_mut(path).unwrap() = bad;
            assert!(check(changed).is_err(), "{path}");
        }
    }
}
