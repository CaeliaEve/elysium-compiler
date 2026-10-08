use super::{Consumption, Edit, Item, Kind, Match, OutputRole, Recipe};
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;

pub(super) fn validate(
    recipe: &Recipe,
    special: &[u32],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let count = recipe.inputs.len();
    ensure!(
        (2..=9).contains(&count) && !special.is_empty() && special.len() < count,
        "floating flower craft needs both types and at most nine occupied cells"
    );
    let start = count - special.len();
    ensure!(
        special.iter().copied().eq(start as u32..count as u32),
        "special flower slots must be the ordered suffix"
    );
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none()
            && recipe.outputs.len() == 1,
        "invalid floating flower process shape"
    );
    let mut registries = Vec::new();
    for (index, input) in recipe.inputs.iter().enumerate() {
        ensure!(
            input.kind == Kind::Item && input.slot == index as u32 && input.choices.len() == 1,
            "floating flowers require one template for each occupied input cell"
        );
        let choice = &input.choices[0];
        ensure!(
            choice.amount == "1"
                && matches!(choice.consume, Consumption::Consume)
                && choice.returns.is_empty(),
            "floating flower settlement consumes one item per occupied cell"
        );
        if index + 1 == count {
            ensure!(
                matches!(&choice.rule, Match::StringTag { key } if key == "type"),
                "last special flower must declare string type domain"
            );
        } else {
            ensure!(
                matches!(
                    choice.rule,
                    Match::Wildcard {
                        meta: true,
                        nbt: true
                    }
                ),
                "other flower inputs ignore metadata and tags"
            );
        }
        registries.push(
            items
                .get(choice.id.as_str())
                .context("missing floating flower input")?
                .registry
                .as_str(),
        );
    }
    ensure!(
        registries[..start].iter().all(|r| *r == registries[0])
            && registries[start..].iter().all(|r| *r == registries[start])
            && registries[0] != registries[start],
        "floating and special flower registry roles differ"
    );
    let output = &recipe.outputs[0];
    let change = output
        .change
        .as_ref()
        .context("floating flower output must retain its type transformation")?;
    let Edit::FloatingFlower { base } = &change.action else {
        anyhow::bail!("invalid floating flower transformation")
    };
    let base = items
        .get(base.as_str())
        .context("missing floating flower base")?;
    ensure!(
        base.meta == 0 && !registries.contains(&base.registry.as_str()),
        "invalid floating special flower base registry"
    );
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && matches!(output.role, OutputRole::Result)
            && output.amount.as_deref() == Some("1")
            && output.quantity.is_none()
            && output.chance.numerator == "1"
            && output.chance.denominator == "1"
            && change.input == count as u32 - 1
            && change.bindings.is_none(),
        "invalid last-special flower result"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::domain::{change, process, Item, Recipe};
    use crate::identity::{item_id, Nbt};
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    fn item(registry: &str, nbt: Option<Nbt>) -> Item {
        serde_json::from_value(
            json!({"id":item_id(registry,0,nbt.as_ref()).unwrap(),"registry":registry,
            "meta":0,"nbt":nbt,"name":"text_flower","tooltip":[],"stackLimit":64,"durability":0,
            "tools":{},"armor":false,"tags":[],"icon":null,"order":null,"aspects":null}),
        )
        .unwrap()
    }

    #[test]
    fn floating_flower_preserves_last_slot_and_fresh_string_output() {
        let floating = item("Botania:floatingFlower", None);
        let base = item(
            "Botania:floatingSpecialFlower",
            Some(Nbt::Compound {
                value: BTreeMap::from([
                    (
                        "type".into(),
                        Nbt::String {
                            value: "baseExample".into(),
                        },
                    ),
                    ("discardBase".into(), Nbt::Int { value: "9".into() }),
                ]),
            }),
        );
        for tag in [
            None,
            Some(Nbt::String {
                value: "pureDaisy".into(),
            }),
            Some(Nbt::String {
                value: String::new(),
            }),
        ] {
            let mut tags = BTreeMap::from([("discard".into(), Nbt::Int { value: "7".into() })]);
            let value = if let Some(Nbt::String { value }) = &tag {
                value.clone()
            } else {
                String::new()
            };
            if let Some(tag) = tag {
                tags.insert("type".into(), tag);
            }
            let flower = item("Botania:specialFlower", Some(Nbt::Compound { value: tags }));
            let rule: crate::domain::Match =
                serde_json::from_value(json!({"kind":"string_tag","key":"type"})).unwrap();
            for bad in [
                Nbt::Int { value: "73".into() },
                Nbt::Compound {
                    value: BTreeMap::new(),
                },
                Nbt::List {
                    element: "string".into(),
                    value: vec![],
                },
            ] {
                let bad_flower = item(
                    "Botania:specialFlower",
                    Some(Nbt::Compound {
                        value: BTreeMap::from([("type".into(), bad)]),
                    }),
                );
                assert!(rule
                    .validate_item(&bad_flower, &BTreeMap::new(), false, true)
                    .is_err());
            }
            let product = item(
                "Botania:floatingSpecialFlower",
                Some(Nbt::Compound {
                    value: BTreeMap::from([("type".into(), Nbt::String { value })]),
                }),
            );
            let items = [&floating, &flower, &base, &product]
                .into_iter()
                .map(|i| (i.id.as_str(), i))
                .collect();
            let choice = |id: &str, rule: Value| json!({"id":id,"amount":"1","rule":rule,"consume":{"kind":"consume"},"returns":[]});
            let mut r = json!({"id":"recipe_flower","source":{"owner":"Botania","handler":"vazkii.botania.client.integration.nei.recipe.RecipeHandlerFloatingFlowers","key":"flowers"},
                "category":"category_flowers","order":0,"duration":null,"energy":null,"grid":null,"magic":null,"view":null,"properties":{},
                "process":{"kind":"floatingFlowers","special":[1,2]},
                "inputs":[{"kind":"item","slot":0,"choices":[choice(&floating.id,json!({"kind":"wildcard","meta":true,"nbt":true}))]},
                    {"kind":"item","slot":1,"choices":[choice(&flower.id,json!({"kind":"wildcard","meta":true,"nbt":true}))]},
                    {"kind":"item","slot":2,"choices":[choice(&flower.id,json!({"kind":"string_tag","key":"type"}))]}],
                "outputs":[{"kind":"item","slot":0,"id":product.id,"amount":"1","quantity":null,"role":"result","chance":{"numerator":"1","denominator":"1"},
                    "change":{"input":2,"action":{"kind":"floatingFlower","base":base.id},"samples":[{"id":product.id,"amount":"1"}]}}]});
            let check = |r: Value| -> anyhow::Result<()> {
                let r: Recipe = serde_json::from_value(r)?;
                process::validate(&r, &BTreeMap::new(), &items, &[])?;
                for i in &r.inputs {
                    for c in &i.choices {
                        c.rule
                            .validate_item(items[c.id.as_str()], &items, false, true)?;
                    }
                }
                change::validate(&r, &r.outputs[0], &items)
            };
            check(r.clone()).unwrap();
            for (path, bad) in [
                ("/process/special", json!([2, 1])),
                ("/process/special", json!([])),
                (
                    "/inputs/2/choices/0/rule",
                    json!({"kind":"wildcard","meta":true,"nbt":true}),
                ),
                ("/outputs/0/change/input", json!(1)),
                ("/outputs/0/change/action/base", json!(flower.id)),
                ("/outputs/0/change/samples/0/id", json!(flower.id)),
                ("/outputs/0/amount", json!("2")),
                ("/duration", json!("1")),
                ("/process", Value::Null),
            ] {
                let mut bad_r = r.clone();
                *bad_r.pointer_mut(path).unwrap() = bad;
                assert!(check(bad_r).is_err(), "{path}");
            }
            r["inputs"][0]["choices"][0]["id"] = json!(flower.id);
            assert!(
                check(r).is_err(),
                "floating input cannot be another special flower"
            );
        }
    }
}
