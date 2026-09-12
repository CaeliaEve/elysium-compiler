use super::*;
use crate::identity::{fluid_id, integer, item_id, resource_name};
use anyhow::{bail, ensure, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

impl Domain {
    pub fn validate(&self, source: &Source) -> Result<()> {
        let items: BTreeMap<_, _> = self
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        let fluids: BTreeSet<_> = self.fluids.iter().map(|fluid| fluid.id.as_str()).collect();
        let texts: BTreeSet<_> = self.strings.iter().map(|text| text.id.as_str()).collect();
        let views: BTreeMap<_, _> = self
            .views
            .iter()
            .map(|view| (view.id.as_str(), view))
            .collect();
        let assets: BTreeSet<_> = self.assets.iter().map(|asset| asset.id.as_str()).collect();
        let categories: BTreeMap<_, _> = self
            .categories
            .iter()
            .map(|category| (category.id.as_str(), category))
            .collect();
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
        magic::validate(self, &text, &reference, &asset)?;
        model::validate(self, &asset)?;
        structure::validate(
            self,
            source.manifest.scope.mode == "complete",
            &source.environment()?.probes,
            &text,
            &reference,
        )?;
        for record in &self.strings {
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
                fluid.temperature >= 0 && fluid.viscosity >= 0 && fluid.luminosity <= 15,
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
            if let Some(view) = &category.view {
                ensure!(
                    views.contains_key(view.as_str()),
                    "missing category view: {view}"
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
        let tracks: BTreeSet<_> = self.tracks.iter().map(|track| track.id.as_str()).collect();
        ensure!(tracks.len() == self.tracks.len(), "duplicate UI track");
        let mut used_tracks = BTreeSet::new();
        for track in &self.tracks {
            ensure!(
                track.id == content_id("track", track)?,
                "UI track identity mismatch"
            );
            motion(&track.frames)?;
        }
        for view in &self.views {
            ensure!(
                view.id == content_id("view", view)?,
                "view identity mismatch: {}",
                view.id
            );
            dimensions(view.width, view.height, 4096)?;
            ensure!(
                view.elements.len() <= 8192,
                "view has too many elements: {}",
                view.id
            );
            for element in &view.elements {
                match element {
                    Element::Clip {
                        asset: id,
                        x,
                        y,
                        width,
                        height,
                        z,
                        track,
                    } => {
                        asset(id)?;
                        rectangle(*x, *y, *width, *height, *z)?;
                        ensure!(tracks.contains(track.as_str()), "missing UI track");
                        used_tracks.insert(track.as_str());
                    }
                    Element::Sprite {
                        asset: id,
                        x,
                        y,
                        width,
                        height,
                        z,
                    } => {
                        asset(id)?;
                        rectangle(*x, *y, *width, *height, *z)?;
                    }
                    Element::Slot {
                        x,
                        y,
                        width,
                        height,
                        z,
                        slot,
                        ..
                    } => {
                        ensure!(*slot <= 65535, "invalid view slot");
                        rectangle(*x, *y, *width, *height, *z)?;
                    }
                    Element::Cost {
                        index,
                        x,
                        y,
                        width,
                        height,
                        z,
                    } => {
                        ensure!(*index < 4096, "invalid view cost index");
                        rectangle(*x, *y, *width, *height, *z)?;
                    }
                    Element::Rectangle {
                        x,
                        y,
                        width,
                        height,
                        z,
                        ..
                    } => rectangle(*x, *y, *width, *height, *z)?,
                    Element::Tooltip {
                        x,
                        y,
                        width,
                        height,
                        z,
                        lines,
                    } => {
                        rectangle(*x, *y, *width, *height, *z)?;
                        for line in lines {
                            text(line)?;
                        }
                    }
                    Element::Text {
                        text: id, x, y, z, ..
                    } => {
                        text(id)?;
                        position(*x, *y, *z)?;
                    }
                }
            }
        }
        ensure!(used_tracks == tracks, "unreferenced UI track");
        let mut recipe_orders = BTreeSet::new();
        for recipe in &self.recipes {
            origin(&recipe.source)?;
            ensure!(
                recipe.id == recipe_id(recipe)?,
                "recipe identity mismatch: {}",
                recipe.id
            );
            let category = categories
                .get(recipe.category.as_str())
                .with_context(|| format!("missing category: {}", recipe.category))?;
            ensure!(
                recipe_orders.insert((&recipe.category, recipe.order)),
                "duplicate recipe order in {}",
                recipe.category
            );
            ensure!(
                recipe.inputs.len() <= 4096 && recipe.outputs.len() <= 4096,
                "recipe has too many slots"
            );
            if let Some(duration) = &recipe.duration {
                integer(duration, 0, i64::MAX)?;
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
                    !slots.is_empty() && slots.len() == recipe.inputs.len(),
                    "crafting grid omits recipe inputs"
                );
            }
            let mut inputs = BTreeSet::new();
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
                            matches!(choice.rule, Match::Exact),
                            "fluid ingredients require exact registry/NBT alternatives"
                        );
                    }
                    reference(input.kind, &choice.id)?;
                    integer(&choice.amount, 1, i64::MAX)?;
                    ensure!(
                        choices.insert((&choice.id, serde_json::to_string(&choice.rule)?)),
                        "duplicate input alternative: {}",
                        choice.id
                    );
                    if let Consumption::Damage { points } = choice.consume {
                        ensure!(
                            input.kind == Kind::Item && points > 0,
                            "invalid durability consumption"
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
                        match &choice.rule {
                            Match::Exact => {}
                            Match::Ore { name, exclusive } => ensure!(
                                item.tags.contains(name) && (!exclusive || item.tags.len() == 1),
                                "ore alternative lacks membership: {} in {name}",
                                choice.id
                            ),
                            Match::Wildcard { meta, nbt } => {
                                ensure!(*meta || *nbt, "wildcard must ignore metadata or NBT")
                            }
                            Match::Tags {
                                keys,
                                present,
                                absent,
                                ..
                            } => {
                                let tags = match &item.nbt {
                                    Some(crate::identity::Nbt::Compound { value }) => Some(value),
                                    _ => None,
                                };
                                let mut used = BTreeSet::new();
                                for list in [keys, present, absent] {
                                    ensure!(
                                        list.len() <= 4096
                                            && list.windows(2).all(|pair| pair[0] < pair[1]),
                                        "tag constraints must be sorted and unique"
                                    );
                                    for key in list {
                                        ensure!(
                                            key.len() <= 65535 && used.insert(key),
                                            "overlapping tag constraints"
                                        );
                                    }
                                }
                                ensure!(
                                    keys.iter()
                                        .chain(present)
                                        .all(|key| tags.is_some_and(|tags| tags.contains_key(key)))
                                        && absent
                                            .iter()
                                            .all(|key| tags
                                                .is_none_or(|tags| !tags.contains_key(key))),
                                    "tag constraint rejects its representative item"
                                );
                            }
                        }
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
                integer(&output.amount, 1, i64::MAX)?;
                chance(&output.chance)?;
                change::validate(recipe, output, &items)?;
            }
            for (key, property) in &recipe.properties {
                resource_name(key)?;
                text(&property.name)?;
                property_value(&property.value, &text, &reference, 0)?;
            }
            if let Some(id) = recipe.view.as_ref().or(category.view.as_ref()) {
                let view = views
                    .get(id.as_str())
                    .with_context(|| format!("missing recipe view: {id}"))?;
                let mut bound_inputs = BTreeSet::new();
                let mut bound_outputs = BTreeSet::new();
                for element in &view.elements {
                    if let Element::Cost { index, .. } = element {
                        ensure!(
                            recipe
                                .magic
                                .as_ref()
                                .is_some_and(|magic| (*index as usize) < magic.aspects.len()),
                            "view refers to a missing magic cost"
                        );
                    }
                    if let Element::Slot {
                        direction,
                        substance,
                        slot,
                        ..
                    } = element
                    {
                        match direction {
                            Direction::Input => {
                                ensure!(
                                    inputs.contains(&(*substance, *slot)),
                                    "view refers to a missing input slot"
                                );
                                bound_inputs.insert((*substance, *slot));
                            }
                            Direction::Output => {
                                ensure!(
                                    outputs.contains(&(*substance, *slot)),
                                    "view refers to a missing output slot"
                                );
                                bound_outputs.insert((*substance, *slot));
                            }
                        }
                    }
                }
                ensure!(
                    bound_inputs == inputs && bound_outputs == outputs,
                    "view omits recipe slots: {}",
                    recipe.id
                );
            }
        }
        let files: BTreeMap<_, _> = source
            .files("asset")
            .map(|file| (file.path.as_str(), file))
            .collect();
        let mut used = BTreeSet::new();
        for asset in &self.assets {
            ensure!(
                asset.id == content_id("asset", asset)?,
                "asset identity mismatch: {}",
                asset.id
            );
            let file = files
                .get(asset.path.as_str())
                .with_context(|| format!("undeclared asset file: {}", asset.path))?;
            used.insert(asset.path.as_str());
            dimensions(asset.width, asset.height, 16384)?;
            ensure!(
                asset.width as u64 * asset.height as u64 <= 16 * 1024 * 1024,
                "asset exceeds 16 million pixels: {}",
                asset.id
            );
            ensure!(
                asset.frames.len() <= 4096 && !asset.source.location.is_empty(),
                "invalid asset source or animation"
            );
            ensure!(
                !asset.interpolate || !asset.frames.is_empty(),
                "still asset cannot interpolate"
            );
            let mut frame_size = None;
            for frame in &asset.frames {
                dimensions(frame.width, frame.height, 4096)?;
                ensure!(
                    frame.x as u64 + frame.width as u64 <= asset.width as u64
                        && frame.y as u64 + frame.height as u64 <= asset.height as u64,
                    "animation crop exceeds asset bounds: {}",
                    asset.id
                );
                ensure!(
                    frame.ticks > 0 && frame.ticks <= 72000,
                    "invalid animation duration"
                );
                let size = (frame.width, frame.height);
                if let Some(previous) = frame_size {
                    ensure!(
                        size == previous,
                        "animation frames must have equal dimensions"
                    );
                }
                frame_size = Some(size);
            }
            let bytes = source.read_file(file)?;
            let format = match file.encoding.as_str() {
                "png" => image::ImageFormat::Png,
                "webp" => image::ImageFormat::WebP,
                _ => bail!("unsupported image encoding"),
            };
            let mut reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
            let mut limits = image::Limits::default();
            limits.max_image_width = Some(asset.width);
            limits.max_image_height = Some(asset.height);
            limits.max_alloc = Some(128 * 1024 * 1024);
            reader.limits(limits);
            let decoded = reader
                .decode()
                .with_context(|| format!("decode asset {}", asset.id))?;
            ensure!(
                decoded.width() == asset.width && decoded.height() == asset.height,
                "asset dimensions differ from pixels: {}",
                asset.id
            );
        }
        ensure!(
            used.len() == files.len(),
            "source declares orphaned image files"
        );
        Ok(())
    }
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

fn position(x: i32, y: i32, z: i32) -> Result<()> {
    ensure!(
        (-65536..=65536).contains(&x)
            && (-65536..=65536).contains(&y)
            && (-65536..=65536).contains(&z),
        "view coordinate exceeds range"
    );
    Ok(())
}

fn rectangle(x: i32, y: i32, width: u32, height: u32, z: i32) -> Result<()> {
    position(x, y, z)?;
    dimensions(width, height, 16384)
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

fn motion(frames: &[Motion]) -> Result<()> {
    ensure!(
        !frames.is_empty() && frames.len() <= 4096,
        "invalid UI motion frame budget"
    );
    let mut total = 0u64;
    for (index, frame) in frames.iter().enumerate() {
        ensure!(
            frame.ticks > 0 && frame.ticks <= 72000 && frame.areas.len() <= 16,
            "invalid UI motion state"
        );
        total += u64::from(frame.ticks);
        ensure!(
            total <= 72000,
            "UI motion cycle exceeds its duration budget"
        );
        ensure!(
            index == 0 || frames[index - 1].areas != frame.areas,
            "UI motion repeats consecutive states"
        );
        let mut areas: Vec<[f32; 4]> = Vec::new();
        for area in &frame.areas {
            let rect = [
                scalar(&area[0], 0.0, 1.0)?,
                scalar(&area[1], 0.0, 1.0)?,
                scalar(&area[2], 0.0, 1.0)?,
                scalar(&area[3], 0.0, 1.0)?,
            ];
            ensure!(
                rect[0] < rect[2] && rect[1] < rect[3],
                "empty or reversed UI clipping area"
            );
            ensure!(
                areas.iter().all(|other| rect[2] <= other[0]
                    || rect[0] >= other[2]
                    || rect[3] <= other[1]
                    || rect[1] >= other[3]),
                "overlapping UI clipping areas"
            );
            areas.push(rect);
        }
    }
    Ok(())
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
