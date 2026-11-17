//! Version-pinned, input-dependent workbench returns. Never predict a sampled remainder.
use super::{Choice, Item, NativeContainerCallback, Recipe};
use anyhow::{ensure, Result};

pub(super) fn validate(recipe: &Recipe, choice: &Choice, item: &Item, callback: NativeContainerCallback) -> Result<()> {
    ensure!(super::native_crafting::container_runtime(recipe), "native container is outside a workbench runtime");
    ensure!(choice.amount == "1" && choice.returns.is_empty(), "dynamic native container cannot have a fixed remainder or stack debit");
    let registries: &[&str] = match callback {
        NativeContainerCallback::Ic2Cutter => &["IC2:itemToolCutter"],
        NativeContainerCallback::BotaniaCosmetic => &["Botania:divaCharm", "Botania:tinyPlanet"],
        NativeContainerCallback::BotaniaBlackHole => &["Botania:blackHoleTalisman"],
        NativeContainerCallback::ThaumicMortar => &["thaumicbases:mortar"],
        NativeContainerCallback::RailFirestone => &["Railcraft:firestone.refined"],
    };
    ensure!(registries.contains(&item.registry.as_str()), "native container callback item identity differs");
    Ok(())
}
