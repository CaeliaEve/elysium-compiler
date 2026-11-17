//! Variable-input vanilla firework rules. NEI's cycling examples are not fixed recipes.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Item, Origin,
};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ColorOrder {
    GridScan,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlightPolicy {
    OnlyWithChargeCell,
    AlwaysWithPowder,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FadePolicy {
    CopyTaggedStarModifyExistingCompound,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "FireworkDefinition")]
pub struct Definition {
    pub source: Origin,
    pub ready: bool,
    pub consuming: bool,
    pub recipe_class: String,
    pub grid_width: u32,
    pub grid_height: u32,
    pub units_per_cell: u32,
    pub output_amount: String,
    pub color_order: ColorOrder,
    pub flight_policy: FlightPolicy,
    pub fade_policy: FadePolicy,
    pub gunpowder: String,
    pub paper: String,
    pub star: String,
    pub rocket: String,
    pub dyes: Vec<Dye>,
    pub shapes: Vec<Shape>,
    pub effects: Vec<Effect>,
    pub examples: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "FireworkDye")]
pub struct Dye {
    pub item: String,
    pub meta: i32,
    pub rgb: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "FireworkShape")]
pub struct Shape {
    pub item: Option<String>,
    pub r#type: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "FireworkEffect")]
pub struct Effect {
    pub item: String,
    pub nbt_key: EffectKey,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum EffectKey {
    Trail,
    Flicker,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "FireworkOutput")]
pub struct Output {
    pub item: String,
    pub amount: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FailureCode {
    MalformedExplosion,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "FireworkFailure")]
pub struct Failure {
    pub code: FailureCode,
    pub exception: String,
    pub cause: String,
    pub inputs: Vec<FailureInput>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "FireworkFailureInput")]
pub struct FailureInput {pub slot: u32, pub template: super::ores::OreTemplate}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "FireworkExample")]
pub struct Example {
    pub label: String,
    pub inputs: [Option<String>; 9],
    pub output: Option<Output>,
    pub failure: Option<Failure>,
}

enum Outcome {
    Unmatched,
    Output(String, i32, Option<Nbt>),
    MalformedExplosion,
}

fn evaluate_example(def:&Definition,example:&Example,items:&BTreeMap<&str,&Item>)->Result<Outcome>{
    let Some(failure)=&example.failure else{return evaluate(def,&example.inputs,items);};
    ensure!(failure.inputs.len()==1,"Malformed firework probe requires one raw state cell");
    let raw=&failure.inputs[0];
    let slot=raw.slot as usize;
    ensure!(slot<9&&example.inputs[slot].is_none(),"Raw firework state overwrites a real cell");
    ensure!(raw.template.registry=="minecraft:firework_charge"&&raw.template.meta==0&&raw.template.amount=="1","Unverified raw firework state item");
    let Some(Nbt::Compound{value})=&raw.template.nbt else{anyhow::bail!("Raw firework state lacks compound NBT");};
    ensure!(value.get("Explosion").is_some_and(|value|!matches!(value,Nbt::Compound{..})),"Raw probe must retain its malformed Explosion");
    let mut item=(*items.get(def.star.as_str()).context("Safe native star material missing")?).clone();
    item.id="!native-firework-failure".into();item.nbt=raw.template.nbt.clone();
    let mut cells=example.inputs.clone();cells[slot]=Some(item.id.clone());
    let mut local=items.clone();local.insert(item.id.as_str(),&item);
    evaluate(def,&cells,&local)
}
fn compound(fields: impl IntoIterator<Item = (&'static str, Nbt)>) -> Nbt {
    Nbt::Compound {
        value: fields
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect(),
    }
}
fn byte(value: i32) -> Nbt {
    Nbt::Byte {
        value: value.to_string(),
    }
}
fn colors(values: &[i32]) -> Nbt {
    Nbt::IntArray {
        value: values.iter().map(ToString::to_string).collect(),
    }
}
fn evaluate(
    def: &Definition,
    cells: &[Option<String>; 9],
    items: &BTreeMap<&str, &Item>,
) -> Result<Outcome> {
    let mut powder = 0;
    let mut paper = 0;
    let mut shapes = 0;
    let mut other = 0;
    let mut shape = 0;
    let mut dyes = Vec::new();
    let mut stars = Vec::new();
    let mut trail = false;
    let mut flicker = false;
    for id in cells.iter().flatten() {
        let item = items
            .get(id.as_str())
            .context("Firework input fact absent")?;
        match item.registry.as_str() {
            "minecraft:gunpowder" => powder += 1,
            "minecraft:paper" => paper += 1,
            "minecraft:firework_charge" => stars.push(*item),
            "minecraft:dye" => {
                let dye = def
                    .dyes
                    .iter()
                    .find(|dye| dye.meta == item.meta)
                    .context("Unsafe native firework dye metadata")?;
                dyes.push(dye.rgb);
                other += 1;
            }
            "minecraft:diamond" => {
                trail = true;
                other += 1;
            }
            "minecraft:glowstone_dust" => {
                flicker = true;
                other += 1;
            }
            "minecraft:fire_charge"
            | "minecraft:gold_nugget"
            | "minecraft:skull"
            | "minecraft:feather" => {
                shapes += 1;
                other += 1;
                shape = def
                    .shapes
                    .iter()
                    .find(|candidate| {
                        candidate
                            .item
                            .as_deref()
                            .and_then(|id| items.get(id))
                            .is_some_and(|candidate| candidate.registry == item.registry)
                    })
                    .context("Native firework shape absent")?
                    .r#type;
            }
            _ => return Ok(Outcome::Unmatched),
        }
    }
    if powder > 3 || paper > 1 {
        return Ok(Outcome::Unmatched);
    }
    if powder >= 1 && paper == 1 && other == 0 {
        let mut tag = compound([]);
        let fixed = matches!(def.flight_policy, FlightPolicy::AlwaysWithPowder);
        if fixed || !stars.is_empty() {
            let explosions: Vec<_> = stars
                .iter()
                .filter_map(|item| {
                    let Some(Nbt::Compound { value }) = &item.nbt else {
                        return None;
                    };
                    value
                        .get("Explosion")
                        .filter(|tag| matches!(tag, Nbt::Compound { .. }))
                        .cloned()
                })
                .collect();
            let mut fields = BTreeMap::from([("Flight".into(), byte(powder))]);
            if !fixed || !explosions.is_empty() {
                fields.insert("Explosions".into(), Nbt::List {
                    element: if explosions.is_empty() { "end" } else { "compound" }.into(), value: explosions,
                });
            }
            tag = compound([("Fireworks", Nbt::Compound { value: fields })]);
        }
        return Ok(Outcome::Output("minecraft:fireworks".into(), 0, Some(tag)));
    }
    if powder == 1 && paper == 0 && stars.is_empty() && !dyes.is_empty() && shapes <= 1 {
        let mut fields = BTreeMap::from([
            ("Type".into(), byte(shape as i32)),
            ("Colors".into(), colors(&dyes)),
        ]);
        if trail {
            fields.insert("Trail".into(), byte(1));
        }
        if flicker {
            fields.insert("Flicker".into(), byte(1));
        }
        return Ok(Outcome::Output(
            "minecraft:firework_charge".into(),
            0,
            Some(compound([("Explosion", Nbt::Compound { value: fields })])),
        ));
    }
    if powder == 0 && paper == 0 && stars.len() == 1 && !dyes.is_empty() && dyes.len() == other {
        let star = stars[0];
        let Some(Nbt::Compound { mut value }) = star.nbt.clone() else {
            return Ok(Outcome::Unmatched);
        };
        if let Some(explosion) = value.get_mut("Explosion") {
            if let Nbt::Compound { value } = explosion {
                value.insert("FadeColors".into(), colors(&dyes));
            } else {
                return Ok(Outcome::MalformedExplosion);
            }
        }
        return Ok(Outcome::Output(
            star.registry.clone(),
            star.meta,
            Some(Nbt::Compound { value }),
        ));
    }
    Ok(Outcome::Unmatched)
}
pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::FireworkDefinition(def) =
        &rows.first().context("Missing firework definition")?.data
    else {
        anyhow::bail!("Firework definition must be first")
    };
    ensure!(
        def.ready
            && !def.consuming
            && def.source.handler == "codechicken.nei.recipe.FireworkRecipeHandler"
            && !def.source.owner.is_empty()
            && !def.source.key.is_empty()
            && matches!((def.recipe_class.as_str(), &def.flight_policy, def.output_amount.as_str()),
                ("net.minecraft.item.crafting.RecipeFireworks", FlightPolicy::OnlyWithChargeCell, "1")
                | ("ganymedes01.etfuturum.recipes.crafting.RecipeFixedFireworks", FlightPolicy::AlwaysWithPowder, "3"))
            && def.grid_width == 3
            && def.grid_height == 3
            && def.units_per_cell == 1,
        "Invalid native firework rule policy"
    );
    ensure!(
        def.examples > 0 && def.examples <= 256 && rows.len() == def.examples as usize + 1,
        "Incomplete firework callback examples"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"fireworkRules","records":data})
        )?,
        "Firework rule identity differs"
    );
    let material = |id: &str, registry: &str, meta: i32| -> Result<()> {
        let item = items.get(id).context("Native firework material absent")?;
        ensure!(
            item.registry == registry && item.meta == meta && item.nbt.is_none(),
            "Native firework material identity changed"
        );
        Ok(())
    };
    for (id, registry) in [
        (&def.gunpowder, "minecraft:gunpowder"),
        (&def.paper, "minecraft:paper"),
        (&def.star, "minecraft:firework_charge"),
        (&def.rocket, "minecraft:fireworks"),
    ] {
        material(id, registry, 0)?;
    }
    ensure!(
        def.dyes.len() == 16 && def.shapes.len() == 5 && def.effects.len() == 2,
        "Incomplete firework material palette"
    );
    for (i, dye) in def.dyes.iter().enumerate() {
        ensure!(
            dye.meta == i as i32 && (0..=0xffffff).contains(&dye.rgb),
            "Invalid native dye metadata/color"
        );
        material(&dye.item, "minecraft:dye", dye.meta)?;
    }
    let shape_registry = [
        None,
        Some("minecraft:fire_charge"),
        Some("minecraft:gold_nugget"),
        Some("minecraft:skull"),
        Some("minecraft:feather"),
    ];
    for (i, shape) in def.shapes.iter().enumerate() {
        ensure!(shape.r#type == i as u8, "Native shape Type differs");
        match (&shape.item, shape_registry[i]) {
            (None, None) => {}
            (Some(id), Some(registry)) => material(id, registry, 0)?,
            _ => anyhow::bail!("Invalid default firework shape"),
        }
    }
    for (i, effect) in def.effects.iter().enumerate() {
        material(
            &effect.item,
            if i == 0 {
                "minecraft:diamond"
            } else {
                "minecraft:glowstone_dust"
            },
            0,
        )?;
        ensure!(
            matches!(
                (&effect.nbt_key, i),
                (EffectKey::Trail, 0) | (EffectKey::Flicker, 1)
            ),
            "Native firework effect key differs"
        );
    }
    let mut labels = BTreeSet::new();
    for row in &rows[1..] {
        let ReferenceData::FireworkExample(example) = &row.data else {
            anyhow::bail!("Mixed firework domain")
        };
        ensure!(
            !example.label.is_empty()
                && example.label.len() <= 128
                && labels.insert(&example.label),
            "Invalid/duplicate native callback example label"
        );
        match (
            evaluate_example(def, example, items)?,
            &example.output,
            &example.failure,
        ) {
            (Outcome::Unmatched, None, None) => {}
            (Outcome::MalformedExplosion, None, Some(failure)) => ensure!(
                failure.exception == "net.minecraft.util.ReportedException"
                    && failure.cause == "java.lang.ClassCastException",
                "Native malformed Explosion failure changed"
            ),
            (Outcome::Output(registry, meta, nbt), Some(output), None) => {
                let item = items
                    .get(output.item.as_str())
                    .context("Native firework output fact absent")?;
                ensure!(
                    output.amount == if registry == "minecraft:fireworks" { def.output_amount.as_str() } else { "1" }
                        && item.registry == registry
                        && item.meta == meta
                        && item.nbt == nbt,
                    "Native firework callback output contradicts exported rule"
                );
            }
            _ => anyhow::bail!("Native firework callback classification contradicts exported rule"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_java_fireworks_preserve_dynamic_nbt_and_reject_semantic_drift() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let mut raw: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixtures.join("firework-reference-contract.json")).unwrap(),
        )
        .unwrap();
        // The historical input keeps its original bytes. GUI example cardinality
        // is retired from the current contract; migrate this test specimen only.
        raw["records"][0]["data"].as_object_mut().unwrap().remove("nativeTemplates");
        let mut rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(raw["records"].clone()).unwrap();
        let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
        let current_domain = super::super::value_id(
            "reference", &serde_json::json!({"kind":"fireworkRules","records":data}),
        ).unwrap();
        for row in &mut rows {
            row.domain = current_domain.clone(); row.id = row.content_id().unwrap();
        }
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        for item in raw["items"].as_array().unwrap() {
            if domain
                .items
                .iter()
                .any(|existing| existing.id == item["id"].as_str().unwrap())
            {
                continue;
            }
            let mut value = base.clone();
            for key in ["id", "registry", "meta", "nbt"] {
                value[key] = item[key].clone();
            }
            value["icon"] = serde_json::Value::Null;
            value["aspects"] = serde_json::Value::Null;
            value["order"] = serde_json::Value::Null;
            value["tags"] = serde_json::json!([]);
            domain.items.push(serde_json::from_value(value).unwrap());
        }
        let ReferenceData::FireworkDefinition(def) = &rows[0].data else {
            panic!()
        };
        let mut category = domain.categories[0].clone();
        category.source = def.source.clone();
        category.id = super::super::origin_id("category", &category.source).unwrap();
        category.reference = Some(rows[0].domain.clone());
        category.program = None;
        category.icon = None;
        category.machines.clear();
        domain.categories.push(category);
        domain.references = rows.clone();
        domain.validate(&source).unwrap();
        let items = domain
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        let check = |mut altered: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = altered.iter().map(|row| &row.data).collect();
            let id = super::super::value_id(
                "reference",
                &serde_json::json!({"kind":"fireworkRules","records":data}),
            )
            .unwrap();
            for row in &mut altered {
                row.domain = id.clone();
                row.id = row.content_id().unwrap();
            }
            super::super::references::validate(&altered, &items)
        };
        let mut changed = rows.clone();
        if let ReferenceData::FireworkExample(example) = &mut changed[1].data {
            example.output.as_mut().unwrap().amount = "3".into();
        }
        assert!(check(changed).is_err());
        let mut missing = rows.clone();
        missing.pop();
        assert!(check(missing).is_err());
        let mut color = rows.clone();
        if let ReferenceData::FireworkDefinition(def) = &mut color[0].data {
            def.dyes[1].rgb += 1;
        }
        assert!(check(color).is_err());
        let mut unknown = raw["records"][0].clone();
        unknown["data"]["fakeProgressTicks"] = serde_json::json!(40);
        assert!(serde_json::from_value::<GeneticReferenceRow>(unknown).is_err());
    }
}
