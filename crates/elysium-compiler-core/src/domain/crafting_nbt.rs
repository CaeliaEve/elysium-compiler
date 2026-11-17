//! Vanilla ShapedRecipes.field_92101_f: the LAST tagged offered row-major cell wins.
//! Examples are independent tag-transfer projections, not a Cartesian matching table.
use super::{change::Stack, Edit, Item, Kind, OutputRole, Process, Recipe};
use crate::identity::item_id;
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;

pub(super) fn validate(recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    match &recipe.process {
        Some(Process::CraftingNbt {}) => {
            ensure!(recipe.source.handler == "codechicken.nei.recipe.ShapedRecipeHandler"
                && recipe.source.key == recipe.source.handler && recipe.source.owner == "NEI",
                "tag transfer requires the native vanilla shaped process");
            super::native_crafting::validate(recipe)?;
        }
        Some(Process::Rolling { copy_nbt: true, earlier, .. }) => super::rolling::validate(recipe, earlier, items)?,
        Some(Process::Qed { copy_nbt: true, earlier, .. }) => super::rolling::validate(recipe, earlier, items)?,
        _ => anyhow::bail!("tag transfer requires an audited native shaped result process"),
    }
    let output = recipe.outputs.first().context("native tag-transfer result missing")?;
    let change = output.change.as_ref().context("native tag-transfer projections missing")?;
    let Edit::CraftingNbt { base } = &change.action else { anyhow::bail!("native tag-transfer action missing"); };
    ensure!(change.input == 0 && change.bindings.is_none(), "tag projections are not input tuples or a single-input transform");
    let prototype = items.get(base.id.as_str()).context("native tag-transfer base missing")?;
    ensure!(output.kind == Kind::Item && matches!(output.role, OutputRole::Result)
        && output.id == base.id && output.amount.as_deref() == Some(base.amount.as_str()), "tag-transfer declaration changed its base result");
    let count: usize = recipe.inputs.iter().map(|input| input.choices.len()).sum();
    ensure!(count <= 65536 && change.samples.len() == count + 1, "native tag projections omit or exceed candidates");
    ensure!(change.samples[0].id == base.id && change.samples[0].amount == base.amount, "untagged result must preserve the base");
    let mut index = 1;
    for input in &recipe.inputs { for choice in &input.choices {
        let specimen = items.get(choice.id.as_str()).context("tag projection input missing")?;
        let expected = Stack {
            id: item_id(&prototype.registry, prototype.meta,
                specimen.nbt.as_ref().or(prototype.nbt.as_ref()))?,
            amount: base.amount.clone(),
        };
        let sample = &change.samples[index];
        ensure!(items.contains_key(sample.id.as_str()) && sample.id == expected.id && sample.amount == expected.amount,
            "native tag projection changed registry, metadata, count or whole-compound replacement");
        index += 1;
    }}
    Ok(())
}
