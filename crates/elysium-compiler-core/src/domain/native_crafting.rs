//! The plain native crafting family. Dynamic subclasses require their own result process.
use super::{Consumption, Kind, Match, OutputRole, Process, Recipe};
use anyhow::{ensure, Result};

/// Only readers whose actual workbench lifecycle calls Item.getContainerItem.
/// Dedicated processes are independently checked against their native handler.
pub(super) fn container_runtime(recipe: &Recipe) -> bool {
    matches!(recipe.source.handler.as_str(),
        "codechicken.nei.recipe.ShapedRecipeHandler" | "codechicken.nei.recipe.ShapelessRecipeHandler")
        || super::magic::ari_source(&recipe.source, super::magic::ARI_SHAPED)
        || super::magic::ari_source(&recipe.source, super::magic::ARI_SHAPELESS)
        || matches!(recipe.process, Some(Process::ExtremeCrafting(_) | Process::AeCrafting(_)
            | Process::Ic2Advanced(_) | Process::ProjectRedCrafting(_) | Process::XuMicroblock(_)
            | Process::MatterManipulatorUpgrade(_) | Process::SalisReplacement(_) | Process::ArcaneSpecial(_)
            | Process::ExtraUtilsSoul(_) | Process::GendustryTemplate(_) | Process::BloodOrbCrafting(_)))
}

pub(super) fn validate(recipe: &Recipe) -> Result<()> {
    if matches!(recipe.process,Some(Process::WirelessTerminalUpgrade(_))) {
        // The dedicated process checks the fixed physical cells, including ignored extras.
        return Ok(());
    }
    if matches!(recipe.source.handler.as_str(),"ic2.neiIntegration.core.recipehandler.AdvRecipeHandler"|"ic2.neiIntegration.core.recipehandler.AdvShapelessRecipeHandler"){
        ensure!(matches!(recipe.process,Some(Process::Ic2Advanced(_))),"Native IC2 Advanced requires its result process");
    }
    let shaped = match recipe.source.handler.as_str() {
        "codechicken.nei.recipe.ShapedRecipeHandler" => true,
        "codechicken.nei.recipe.ShapelessRecipeHandler" => false,
        _ => return Ok(()),
    };
    ensure!(recipe.duration.is_none() && recipe.energy.is_none() && recipe.magic.is_none(),
        "manual native crafting cannot acquire machine costs or a magic process");
    ensure!(matches!(recipe.process, None | Some(Process::MapScaling {} | Process::CraftingNbt {} | Process::GtCrafting(_))),
        "native crafting requires a dedicated dynamic result process");
    if shaped {
        let grid = recipe.grid.as_ref().ok_or_else(|| anyhow::anyhow!("native shaped recipe requires its logical grid"))?;
        ensure!((1..=3).contains(&grid.width) && (1..=3).contains(&grid.height), "invalid native crafting grid size");
    } else {
        ensure!(recipe.grid.is_none(), "native shapeless multiplicity cannot become a shaped grid");
    }
    ensure!(!recipe.inputs.is_empty() && recipe.inputs.len() <= 9, "invalid native crafting input multiplicity");
    for (index, input) in recipe.inputs.iter().enumerate() {
        ensure!(input.kind == Kind::Item && input.slot == index as u32, "native crafting cells require ordered item input slots");
        for choice in &input.choices {
            ensure!(choice.amount == "1", "native crafting consumes one item per occupied cell");
            ensure!(matches!(choice.rule, Match::Wildcard { nbt: true, .. } | Match::NativeItemMetadata { nbt: true, .. }), "native crafting requires its metadata/ignored-NBT predicate");
            ensure!(matches!(choice.consume, Consumption::Consume | Consumption::Keep | Consumption::Damage { .. } | Consumption::GtTool { .. } | Consumption::NativeContainer { .. }),
                "native crafting container consumption needs a dedicated adapter");
        }
    }
    ensure!(recipe.outputs.len() == 1, "native crafting has one result stack; containers belong to ingredient returns");
    let output = &recipe.outputs[0];
    ensure!(output.kind == Kind::Item && output.slot == 0 && matches!(output.role, OutputRole::Result)
        && output.quantity.is_none() && output.amount.is_some()
        && output.chance.numerator == "1" && output.chance.denominator == "1", "invalid native crafting result");
    if recipe.process.is_none() {
        ensure!(output.change.is_none(), "native fixed result acquired an unsupported dynamic transform");
    }
    Ok(())
}
