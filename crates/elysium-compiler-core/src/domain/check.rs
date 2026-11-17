use super::*;
use crate::identity::{fluid_id, integer, item_id, property_key};
use anyhow::{bail, ensure, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

impl Domain {
    pub fn validate(&self, source: &Source) -> Result<()> {
        self.validate_records(source)?;
        images::verify(source, &self.assets)
    }

    pub(crate) fn validate_records(&self, source: &Source) -> Result<()> {
        visual::validate(&self.visuals, &self.assets)?;
        let programs = programs::reconstruct(&self.programs)?;
        let mut referenced_programs = BTreeSet::new();
        for category in &self.categories {
            if let Some(id) = &category.program {
                ensure!(
                    programs.contains_key(id),
                    "Category refers to a missing shared program: {id}"
                );
                referenced_programs.insert(id);
            }
        }
        ensure!(
            referenced_programs.len() == programs.len(),
            "Unreferenced shared machine program"
        );
        let items: BTreeMap<_, _> = self
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        let fluids: BTreeSet<_> = self.fluids.iter().map(|fluid| fluid.id.as_str()).collect();
        for program in programs.values(){if let programs::SharedProgram::Fmp(rules)=program{
            for material in &rules.materials{let block=items.get(material.block.as_str()).context("FMP shared material block fact absent")?;
                ensure!(block.registry!=rules.settings().micro_registry,"FMP full block cannot be a micro part");}
        }}
        for program in programs.values(){if let programs::SharedProgram::Xu(rules)=program{
            for material in &rules.materials{let block=items.get(material.block.as_str()).context("XU shared material item fact absent")?;
                ensure!(block.nbt.is_none(),"XU audited native block item cannot invent a tag");}
        }}
        let fluid_records: BTreeMap<_, _> = self
            .fluids
            .iter()
            .map(|fluid| (fluid.id.as_str(), fluid))
            .collect();
        let texts: BTreeSet<_> = self.strings.iter().map(|text| text.id.as_str()).collect();
        let text_records:BTreeMap<_,_>=self.strings.iter().map(|text|(text.id.as_str(),text)).collect();
        let assets: BTreeSet<_> = self.assets.iter().map(|asset| asset.id.as_str()).collect();
        let categories: BTreeMap<_, _> = self
            .categories
            .iter()
            .map(|category| (category.id.as_str(), category))
            .collect();
        references::validate(&self.references,&items)?;
        forge_registry::validate_facts(&self.references,&fluid_records)?;
        usage_queries::validate_facts(&self.references,&fluid_records)?;
        structure_references::validate_facts(self)?;
        betterquesting::validate_facts(&self.references,&fluid_records)?;
        underground_fluids::validate_facts(&self.references,&fluid_records)?;
        aspect_combinations::validate_facts(&self.references, &self.aspects)?;
        nei_diagrams::validate_facts(&self.references, &fluid_records)?;
        ender_storage::validate_facts(&self.references, &fluid_records)?;
        let reference_domains: BTreeMap<_,_>=self.references.iter().filter_map(|row| {
            match &row.data {
                references::ReferenceData::CropsNhDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::UsageDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::ProjectBlueDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::LatheDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::FireworkDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::FurnaceFuelDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::Definition{source,..} => Some((row.domain.as_str(),source)),
                references::ReferenceData::AspectCombinationsDefinition{source,..} => Some((row.domain.as_str(),source)),
                references::ReferenceData::ItemAspectsDefinition{source,..} => Some((row.domain.as_str(),source)),
                references::ReferenceData::LexiconDefinition{source,..} => Some((row.domain.as_str(),source)),
                references::ReferenceData::MobDropsDefinition{source,..} => Some((row.domain.as_str(),source)),
                references::ReferenceData::InfernalDropsDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::DraconicReactorDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::BeeTreeDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::ForestryAddonsDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::UndergroundFluidsDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::GtMineralsDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::BartworksOresDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::QuestsDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::StructureCatalogDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::VillagerProfession{source,..} => Some((row.domain.as_str(),source)),
                references::ReferenceData::ToolMaterialsDefinition{source,..} => Some((row.domain.as_str(),source)),
                references::ReferenceData::ForgeRegistryDefinition(def) => Some((row.domain.as_str(),&def.source)),
                references::ReferenceData::NeiInformationDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::NeiDiagramDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::EnderStorageDefinition(definition) => Some((row.domain.as_str(),&definition.source)),
                references::ReferenceData::NeiInformationPage(page) => Some((row.domain.as_str(),&page.source)),
                references::ReferenceData::NeiInteractiveGroup{source,..} => Some((row.domain.as_str(),source)),
                _ => None,
            }
        }).collect();
        let mut used_reference_domains=BTreeSet::new();
        for category in &self.categories {
            if let Some(domain)=&category.reference {
                let native=reference_domains.get(domain.as_str()).context("Category refers to a missing native reference domain")?;
                ensure!(serde_json::to_value(&category.source)?==serde_json::to_value(native)?,
                    "Reference category changed native source identity");
                ensure!(used_reference_domains.insert(domain.as_str()) && category.program.is_none()
                    && !self.recipes.iter().any(|recipe|recipe.category==category.id),
                    "Native reference category became consuming recipes or shared machine program");
            }
        }
        ensure!(used_reference_domains.len()==reference_domains.len(),"Unreferenced native Database domain");
        let text = |id: &str| -> Result<()> {
            ensure!(texts.contains(id), "missing string: {id}");
            Ok(())
        };
        let asset = |id: &str| -> Result<()> {
            ensure!(assets.contains(id), "missing asset: {id}");
            Ok(())
        };
        let reference = |kind: Kind, id: &str| -> Result<()> {
            let exists = match kind {
                Kind::Item => items.contains_key(id),
                Kind::Fluid => fluids.contains(id),
            };
            ensure!(exists, "missing {kind:?}: {id}");
            Ok(())
        };
        industry::validate(self, &text, &reference)?;
        genetics::validate(self, &text, &reference)?;
        ores::validate(self)?;
        magic::validate(self, &text, &reference, &asset)?;
        model::validate(self, &asset)?;
        let environment = source.environment()?;
        structure::validate(
            self,
            source.manifest.scope.mode == "complete",
            &environment.probes,
            &text,
            &reference,
        )?;
        let locale = &environment.locale;
        usage_queries::validate_locale(&self.references,locale)?;
        for record in &self.strings {
            ensure!(record.locale == *locale, "string locale differs from source environment: {}", record.id);
            ensure!(
                record.id == content_id("text", record)?,
                "string identity mismatch: {}",
                record.id
            );
            ensure!(
                !record.locale.is_empty()
                    && record.locale.len() <= 32
                    && record
                        .locale
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-'),
                "invalid locale"
            );
            ensure!(
                record.text.len() <= 65536,
                "string exceeds 64 KiB: {}",
                record.id
            );
        }
        let mut orders = BTreeSet::new();
        for item in &self.items {
            ensure!(
                item.id == item_id(&item.registry, item.meta, item.nbt.as_ref())?,
                "item identity mismatch: {}",
                item.id
            );
            text(&item.name)?;
            for line in &item.tooltip {
                text(line)?;
            }
            if let Some(icon) = &item.icon {
                asset(icon)?;
            }
            ensure!(
                item.stack_limit > 0 && item.stack_limit <= i32::MAX as u32,
                "invalid stack limit: {}",
                item.id
            );
            ensure!(
                item.tags.windows(2).all(|pair| pair[0] < pair[1]),
                "item tags must be unique and sorted: {}",
                item.id
            );
            for tag in &item.tags {
                ensure!(!tag.is_empty() && tag.len() <= 256, "invalid ore name");
            }
            if let Some(order) = item.order {
                ensure!(orders.insert(order), "duplicate NEI order: {order}");
            }
        }
        for fluid in &self.fluids {
            ensure!(
                fluid.id == fluid_id(&fluid.registry, fluid.nbt.as_ref())?,
                "fluid identity mismatch: {}",
                fluid.id
            );
            text(&fluid.name)?;
            if let Some(icon) = &fluid.icon {
                asset(icon)?;
            }
            ensure!(
                fluid.temperature >= 0 && fluid.luminosity <= 15,
                "invalid fluid properties: {}",
                fluid.id
            );
        }
        for category in &self.categories {
            origin(&category.source)?;
            ensure!(
                category.id == origin_id("category", &category.source)?,
                "category identity mismatch: {}",
                category.id
            );
            text(&category.name)?;
            notes(&category.notes, &text)?;
            if let Some(icon) = &category.icon {
                reference(icon.kind, &icon.id)?;
            }
            let mut machines = BTreeSet::new();
            for machine in &category.machines {
                reference(machine.kind, &machine.id)?;
                ensure!(
                    machines.insert((machine.kind, &machine.id)),
                    "duplicate category machine"
                );
            }
        }
        let mut grouped = BTreeSet::new();
        for group in &self.groups {
            origin(&group.source)?;
            ensure!(
                group.id == origin_id("group", &group.source)?,
                "group identity mismatch: {}",
                group.id
            );
            if let Some(name) = &group.name {
                text(name)?;
            }
            ensure!(
                !group.members.is_empty() && group.members.contains(&group.representative),
                "invalid group representative: {}",
                group.id
            );
            let mut members = BTreeSet::new();
            for member in &group.members {
                reference(Kind::Item, member)?;
                ensure!(members.insert(member), "duplicate group member: {member}");
                ensure!(
                    grouped.insert(member),
                    "item belongs to multiple browser groups: {member}"
                );
            }
        }
        for recipe in &self.recipes {
            origin(&recipe.source)?;
            gt_probability::validate(recipe, &reference)?;
            ensure!(
                recipe.id == recipe_id(recipe)?,
                "recipe identity mismatch: {}",
                recipe.id
            );
            let category = categories
                .get(recipe.category.as_str())
                .with_context(|| format!("missing category: {}", recipe.category))?;
            ensure!(
                recipe.inputs.len() <= 4096 && recipe.outputs.len() <= 4096,
                "recipe has too many slots"
            );
            if let Some(duration) = &recipe.duration {
                integer(duration, i64::MIN, i64::MAX)?;
            }
            if let Some(energy) = &recipe.energy {
                integer(energy, i64::MIN, i64::MAX)?;
            }
            if let Some(grid) = &recipe.grid {
                dimensions(grid.width, grid.height, 32)?;
                ensure!(
                    grid.cells.len() == (grid.width * grid.height) as usize,
                    "crafting grid dimensions differ from its cells"
                );
                let mut slots = BTreeSet::new();
                for slot in grid.cells.iter().flatten() {
                    ensure!(slots.insert(*slot), "crafting grid repeats an input slot");
                    ensure!(
                        recipe
                            .inputs
                            .iter()
                            .any(|input| input.kind == Kind::Item && input.slot == *slot),
                        "crafting grid refers to a missing input"
                    );
                }
                ensure!(
                    !slots.is_empty() && (matches!(recipe.process,Some(Process::ForestryFabricator(_) | Process::ForestryProcessing(_)))
                        || slots.len() == recipe.inputs.len()),
                    "crafting grid omits recipe inputs"
                );
            }
            let mut inputs = BTreeSet::new();
            native_crafting::validate(recipe)?;
            process::validate(recipe, &fluid_records, &items, &self.aspects)?;
            if let Some(Process::NativeBotaniaPool(process))=&recipe.process{
                let programs::SharedProgram::BeePool(rules)=programs.get(&process.program).context("Missing native pool shared program")?
                    else{bail!("Native pool linked to another program family");};
                botania_pool::validate_program(process,recipe,rules,category,&items)?;
            }
            ensure!(!recipe.inputs.iter().flat_map(|i|&i.choices).any(|c|matches!(c.rule,Match::NativePoolInput))
                ||matches!(recipe.process,Some(Process::NativeBotaniaPool(_))),"Native pool display requires its paired process");
            if let Some(Process::FmpMicroblock(process))=&recipe.process{
                let programs::SharedProgram::Fmp(rules)=programs.get(&process.program).context("Missing FMP shared program")?
                    else{bail!("FMP linked to another native program family");};
                fmp::validate_program(process,recipe,rules,category,&items)?;
            }
            if let Some(Process::XuMicroblock(process))=&recipe.process{
                let programs::SharedProgram::Xu(rules)=programs.get(&process.program).context("Missing XU shared material program")?
                    else{bail!("XU linked to another native program family");};
                xu_microblock::validate_program(process,recipe,rules,category,&items)?;
            }
            forestry::recipes::validate(recipe, &programs, category, &items, &fluid_records)?;
            if let Some(Process::ForestryFabricator(process)) = &recipe.process {
                let programs::SharedProgram::Fabricator(program) = programs.get(&process.program).context("Missing Fabricator program")?
                    else {bail!("Fabricator recipe linked to a different machine program");};
                fabricator::validate_recipe(process,recipe,program,category,&items,&fluid_records)?;
            }
            if let Some(Process::ForestryProcessing(process)) = &recipe.process {
                let programs::SharedProgram::Processing(program) = programs.get(&process.program).context("Missing Forestry processing program")?
                    else {bail!("Forestry processing recipe linked to a different machine program");};
                forestry_processing::validate_recipe(process,recipe,program,category,&items,&fluid_records)?;
            }
            for input in &recipe.inputs {
                ensure!(
                    input.slot <= 65535 && inputs.insert((input.kind, input.slot)),
                    "duplicate or invalid input slot: {}",
                    recipe.id
                );
                ensure!(
                    !input.choices.is_empty() && input.choices.len() <= 65536,
                    "invalid alternatives: {}",
                    recipe.id
                );
                let mut choices = BTreeSet::new();
                for choice in &input.choices {
                    if input.kind == Kind::Fluid {
                        ensure!(
                            matches!(
                                choice.rule,
                                Match::Exact
                                    | Match::NativeZeroFluid { .. }
                                    | Match::LumbermillTank
                                    | Match::Wildcard {
                                        meta: false,
                                        nbt: true
                                    }
                            ),
                            "fluid ingredients require a fixed registry with exact or ignored NBT"
                        );
                    }
                    reference(input.kind, &choice.id)?;
                    if let Consumption::GtTool {cost,owner,sample} = &choice.consume {
                        ensure!(input.kind == Kind::Item, "GT crafting tool cannot be a fluid");
                        gt_tool::validate(recipe, choice, items.get(choice.id.as_str()).context("missing GT tool specimen")?, *cost, owner, sample)?;
                    }
                    if let Consumption::NativeContainer { callback } = choice.consume {
                        ensure!(input.kind == Kind::Item, "native crafting container cannot be a fluid");
                        super::native_container::validate(recipe, choice, items.get(choice.id.as_str()).context("missing native container specimen")?, callback)?;
                    }
                    if let Match::NativeZeroFluid { policy } = choice.rule {
                        native_zero_fluid(recipe, input, choice, policy)?;
                    } else if matches!(choice.rule, Match::Forestry) {
                        integer(&choice.amount, i64::from(i32::MIN), i64::from(i32::MAX))?;
                    } else {
                        integer(&choice.amount, 1, i64::MAX)?;
                    }
                    ensure!(
                        choices.insert(serde_json::to_string(choice)?),
                        "duplicate input alternative: {}",
                        choice.id
                    );
                    if let Consumption::Damage { points } = choice.consume {
                        ensure!(
                            input.kind == Kind::Item && points > 0,
                            "invalid durability consumption"
                        );
                    }
                    if let Consumption::Chance { chance: probability } = &choice.consume {
                        ensure!(recipe.gt_native.is_some() ||
                            input.kind == Kind::Item && matches!(recipe.process,Some(Process::GendustryGenetics(_))),
                            "conditional consumption requires a typed native RNG process");
                        chance(probability)?;
                    }
                    if matches!(choice.consume, Consumption::Stack) {
                        // A decay chest replaces the whole stored slot under its checked lifecycle.
                        let decay = matches!(recipe.process,Some(Process::DecayChest(_)));
                        ensure!(
                            input.kind == Kind::Item
                                && (decay || matches!(recipe.process,Some(Process::NativeInfusion(_)|Process::DisguiseInfusion(_)|Process::NativeRunic(_))) || matches!((&choice.rule, &recipe.process),
                                    (Match::BloodAltar { .. }, Some(Process::BloodAltar(p))) if !p.can_be_filled)
                                    || matches!((&choice.rule, &recipe.process),
                                        (Match::ChiselSource { .. }, Some(Process::Chisel(p))) if p.mode == super::chisel::ChiselMode::Stack)
                                    || matches!(choice.rule, Match::Member { .. })
                                && recipe.outputs.iter().any(|output| output
                                    .change
                                    .as_ref()
                                    .is_some_and(|change| change.input == input.slot
                                        && matches!(change.action, Edit::Analyze)))),
                            "whole-stack consumption requires native analysis"
                        );
                    }
                    ensure!(
                        choice.returns.len() <= 256,
                        "too many ingredient remainders"
                    );
                    for returned in &choice.returns {
                        reference(returned.kind, &returned.id)?;
                        integer(&returned.amount, 1, i64::MAX)?;
                    }
                    if input.kind == Kind::Item {
                        let item = items[choice.id.as_str()];
                        choice.rule.validate_item(
                            item,
                            &items,
                            matches!(choice.consume, Consumption::Stack),
                            true,
                        )?;
                    }
                }
            }
            let mut outputs = BTreeSet::new();
            for output in &recipe.outputs {
                ensure!(
                    output.slot <= 65535 && outputs.insert((output.kind, output.slot)),
                    "duplicate or invalid output slot: {}",
                    recipe.id
                );
                reference(output.kind, &output.id)?;
                quantity_bounds(recipe, output)?;
                chance(&output.chance)?;
                change::validate(recipe, output, &items)?;
            }
            for (key, property) in &recipe.properties {
                property_key(key)?;
                text(&property.name)?;
                property_value(&property.value, &text, &reference, 0)?;
            }
            notes(&recipe.notes, &text)?;
            botania::validate_observations(recipe,&text_records,&locale)?;
        }
        images::validate_metadata(source, &self.assets)
    }
}

fn native_zero_fluid(recipe: &Recipe, input: &Input, choice: &Choice, policy: NativeZeroFluidPolicy) -> Result<()> {
    ensure!(input.kind == Kind::Fluid && choice.amount == "0"
        && matches!(choice.consume, Consumption::Keep) && choice.returns.is_empty(),
        "native zero fluid must retain a zero amount without debit/returns");
    match policy {
        NativeZeroFluidPolicy::BinnieIncubator => ensure!(
            recipe.source.handler=="binnie.genetics.nei.IncubatorRecipeHandler" && input.slot==0
                && matches!(&recipe.process,Some(Process::BinnieGenetics(binnie_genetics::GeneticsProcess::Incubator(p))) if p.recipe.fluid_input.amount==0),
            "Incubator zero matching requires its native fluid-presence process"),
        NativeZeroFluidPolicy::VatMatch => ensure!(
            recipe.source.handler == "crazypants.enderio.nei.VatRecipeHandler"
                && matches!(recipe.process, Some(Process::Vat { .. })),
            "VAT zero matching requires its native table process"),
        NativeZeroFluidPolicy::GregtechCost | NativeZeroFluidPolicy::GregtechPresence => {
            ensure!(matches!(recipe.source.handler.as_str(),
                "gregtech.nei.GTNEIDefaultHandler" | "bartworks.neiHandler.BioLabNEIHandler" | "bartworks.neiHandler.BioVatNEIHandler")
                && recipe.process.is_none(), "GT zero-cost policy requires an ordinary native GT recipe");
            let rock_breaker = recipe.source.key.starts_with("gtpp.recipe.multiblockrockbreaker/");
            ensure!(rock_breaker == (policy == NativeZeroFluidPolicy::GregtechPresence),
                "GT zero-cost/presence policy differs from the native machine family");
            if rock_breaker {
                let fluids: Vec<_> = recipe.inputs.iter().filter(|i| i.kind == Kind::Fluid).collect();
                ensure!(fluids.len() == 2, "Rock breaker requires both native fluid presence conditions");
                for (slot, registry) in [(0, "water"), (1, "lava")] {
                    let requirement = fluids.iter().find(|i| i.slot == slot).context("Missing native rock breaker fluid slot")?;
                    ensure!(requirement.choices.len() == 1, "Rock breaker fluid presence is not an alternative");
                    let value = &requirement.choices[0];
                    ensure!(value.id == fluid_id(registry, None)? && value.amount == "0"
                        && matches!(value.rule, Match::NativeZeroFluid { policy: NativeZeroFluidPolicy::GregtechPresence })
                        && matches!(value.consume, Consumption::Keep) && value.returns.is_empty(),
                        "Rock breaker fluid presence differs from native water/lava identities");
                }
            }
        }
    }
    Ok(())
}

pub(super) fn origin(value: &Origin) -> Result<()> {
    for part in [&value.owner, &value.handler, &value.key] {
        ensure!(
            !part.is_empty() && part.len() <= 512 && !part.chars().any(char::is_control),
            "invalid record origin"
        );
    }
    Ok(())
}

fn dimensions(width: u32, height: u32, limit: u32) -> Result<()> {
    ensure!(
        width > 0 && height > 0 && width <= limit && height <= limit,
        "invalid dimensions {width}x{height}"
    );
    Ok(())
}

pub(super) fn chance(chance: &Chance) -> Result<()> {
    let denominator = integer(&chance.denominator, 1, i64::MAX)?;
    let numerator = integer(&chance.numerator, 0, denominator)?;
    ensure!(
        gcd(numerator as u64, denominator as u64) == 1,
        "chance must be a reduced fraction"
    );
    Ok(())
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

pub(super) fn scalar(value: &str, minimum: f32, maximum: f32) -> Result<f32> {
    ensure!(
        !value.is_empty()
            && value.len() <= 32
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || b"+-.eE".contains(&byte)),
        "invalid rendering coordinate"
    );
    let number: f32 = value.parse().context("invalid rendering coordinate")?;
    ensure!(
        number.is_finite() && number >= minimum && number <= maximum,
        "rendering coordinate outside bounds"
    );
    Ok(number)
}

fn decimal(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && value.len() <= 256 && value != "-0",
        "invalid decimal property"
    );
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let mut parts = unsigned.split('.');
    let whole = parts.next().unwrap_or_default();
    ensure!(
        !whole.is_empty()
            && whole.bytes().all(|byte| byte.is_ascii_digit())
            && (whole == "0" || !whole.starts_with('0')),
        "invalid decimal property"
    );
    if let Some(fraction) = parts.next() {
        ensure!(
            !fraction.is_empty()
                && fraction.bytes().all(|byte| byte.is_ascii_digit())
                && !fraction.ends_with('0'),
            "decimal must be canonical"
        );
    }
    ensure!(parts.next().is_none(), "invalid decimal property");
    Ok(())
}

pub(super) fn property_value(
    value: &PropertyValue,
    text: &dyn Fn(&str) -> Result<()>,
    reference: &dyn Fn(Kind, &str) -> Result<()>,
    depth: usize,
) -> Result<()> {
    ensure!(depth <= 16, "property nesting exceeds 16 levels");
    match value {
        PropertyValue::Integer { value } => {
            integer(value, i64::MIN, i64::MAX)?;
        }
        PropertyValue::Decimal { value } => decimal(value)?,
        PropertyValue::Quantity { amount, .. } => {
            integer(amount, i64::MIN, i64::MAX)?;
        }
        PropertyValue::Text { text: id } => text(id)?,
        PropertyValue::Reference { target } => reference(target.kind, &target.id)?,
        PropertyValue::Flag { .. } => {}
        PropertyValue::Symbol { namespace, value } => {
            ensure!(
                !namespace.is_empty()
                    && namespace.len() <= 256
                    && !value.is_empty()
                    && value.len() <= 256,
                "invalid property symbol"
            );
        }
        PropertyValue::List { values } => {
            ensure!(values.len() <= 4096, "property list exceeds 4096 values");
            for value in values {
                property_value(value, text, reference, depth + 1)?;
            }
        }
        PropertyValue::Map { values } => {
            ensure!(values.len() <= 4096, "property map exceeds 4096 values");
            for (key, value) in values {
                ensure!(
                    !key.is_empty() && key.len() <= 256,
                    "invalid property map key"
                );
                property_value(value, text, reference, depth + 1)?;
            }
        }
    }
    Ok(())
}


fn notes(notes: &[Note], text: &impl Fn(&str) -> Result<()>) -> Result<()> {
    ensure!(notes.len() <= 256, "too many native notes");
    let mut keys = BTreeSet::new();
    for note in notes {
        ensure!(note.key.len() <= 256, "native note key exceeds 256 bytes");
        property_key(&note.key)?;
        ensure!(keys.insert(&note.key), "duplicate native note key");
        ensure!(!note.source.is_empty() && note.source.len() <= 512
            && !note.source.chars().any(char::is_control), "invalid native note source");
        text(&note.text)?;
    }
    Ok(())
}
