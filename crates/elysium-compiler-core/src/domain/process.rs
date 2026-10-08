use super::{Consumption, Fluid, Kind, Match, OutputRole, Quantity, Recipe};
use crate::identity::integer;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests {
    #[test]
    fn blast_requires_staged_count_gates_and_conditional_slag() {
        use crate::domain::Recipe;
        use serde_json::json;
        let choice = |id, amount, rule, returns| json!({"id":id,"amount":amount,"rule":rule,"consume":{"kind":"staged"},"returns":returns});
        let mut value = json!({"id":"recipe_test","source":{"owner":"IC2","handler":"native","key":"blast"},"category":"category_test","order":0,
            "duration":null,"energy":null,"grid":null,"magic":null,"view":null,"properties":{},"process":{"kind":"ic2Blast","heat":50000,"containers":[[null],[null]]},
            "inputs":[{"kind":"item","slot":0,"choices":[choice("item_iron","4",json!({"kind":"wildcard","meta":false,"nbt":true}),json!([]))]},
                {"kind":"item","slot":1,"choices":[choice("item_air","1",json!({"kind":"untagged","meta":true}),json!([{"kind":"item","id":"item_cell","amount":"1"}]))]}],
            "outputs":[{"kind":"item","slot":0,"id":"item_steel","amount":"2","quantity":null,"change":null,"role":"result","chance":{"numerator":"1","denominator":"1"}},
                {"kind":"item","slot":1,"id":"item_slag","amount":null,"quantity":{"kind":"potential","stat":"ic2:slagSpace","nominal":"3"},"change":null,"role":"result","chance":{"numerator":"1","denominator":"1"}}]});
        let check = |v: serde_json::Value| {
            let recipe: Recipe = serde_json::from_value(v)?;
            super::validate(&recipe, &Default::default(), &Default::default(), &[])
        };
        check(value.clone()).unwrap();
        for (path, bad) in [
            ("/process", serde_json::Value::Null),
            ("/duration", json!("6000")),
            ("/energy", json!("40")),
            ("/inputs/0/choices/0/consume", json!({"kind":"consume"})),
            ("/inputs/1/choices/0/amount", json!("6")),
            (
                "/inputs/1/choices/0/rule",
                json!({"kind":"wildcard","meta":false,"nbt":true}),
            ),
            ("/inputs/1/choices/0/returns", json!([])),
            ("/outputs/1/amount", json!("3")),
            ("/outputs/1/quantity/stat", json!("random_chance")),
            ("/process/containers", json!([[], [null]])),
            ("/process/containers", json!([[null], [null, null]])),
            (
                "/process/containers/0/0",
                json!({"id":"item_missing","amount":"1"}),
            ),
        ] {
            let mut bad_value = value.clone();
            *bad_value.pointer_mut(path).unwrap() = bad;
            assert!(check(bad_value).is_err(), "{path}");
        }
        value["process"]["heat"] = json!(-1);
        value["outputs"].as_array_mut().unwrap().pop();
        check(value).unwrap();
    }
    #[test]
    fn unstable_casting_preserves_runtime_output_semantics() {
        use crate::domain::Recipe;
        use serde_json::json;
        let recipe = json!({"id":"recipe_test","source":{"owner":"TConstruct","handler":"tconstruct.plugins.nei.RecipeHandlerCastingTable","key":"casting"},
            "category":"category_test","order":0,"duration":"80","energy":null,"grid":null,"magic":null,"view":null,"properties":{},
            "process":{"kind":"unstableCasting"},"inputs":[{"kind":"fluid","slot":0,"choices":[{"id":"fluid_metal","amount":"144","rule":{"kind":"exact"},"consume":{"kind":"consume"},"returns":[]}]}],
            "outputs":[{"kind":"item","slot":0,"id":"item_part","amount":"1","quantity":null,"change":null,"role":"result","chance":{"numerator":"1","denominator":"1"}}]});
        let check = |value| -> anyhow::Result<()> {
            let recipe: Recipe = serde_json::from_value(value)?;
            super::validate(&recipe, &Default::default(), &Default::default(), &[])
        };
        check(recipe.clone()).unwrap();
        for (path, bad) in [
            ("/source/handler", json!("other")),
            ("/duration", json!(null)),
            ("/energy", json!("32")),
            ("/outputs/0/chance/numerator", json!("0")),
        ] {
            let mut value = recipe.clone();
            *value.pointer_mut(path).unwrap() = bad;
            assert!(check(value).is_err(), "{path}");
        }
    }
    #[test]
    fn refinery_preserves_preflight_requirements_and_stale_fill_permissions() {
        use crate::{
            domain::{Domain, Recipe},
            source::Source,
        };
        use serde_json::json;
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = Domain::load(&source).unwrap();
        let fluids: std::collections::BTreeMap<_, _> =
            domain.fluids.iter().map(|f| (f.id.as_str(), f)).collect();
        let water = domain
            .fluids
            .iter()
            .find(|f| f.registry == "water" && f.nbt.is_none())
            .unwrap();
        let honey = domain
            .fluids
            .iter()
            .find(|f| f.registry == "honey" && f.nbt.is_none())
            .unwrap();
        let mut value = serde_json::to_value(&domain.recipes[0]).unwrap();
        for k in ["duration", "energy", "grid", "magic"] {
            value[k] = serde_json::Value::Null;
        }
        let requirement = json!({"kind":"fluid","slot":0,"choices":[{"id":water.id,"amount":"700","rule":{"kind":"exact"},"consume":{"kind":"allocated"},"returns":[]}]});
        value["inputs"] = json!([requirement.clone(), requirement]);
        value["inputs"][1]["slot"] = json!(1);
        value["outputs"] = json!([{"kind":"fluid","slot":0,"id":honey.id,"amount":"50","quantity":null,"change":null,"role":"result","chance":{"numerator":"1","denominator":"1"}}]);
        value["process"] = json!({"kind":"buildcraftRefinery","energy":30,"delay":"5","capacity":4000,"earlier":[[{"id":water.id,"amount":"200"}]],"filling":[[honey.id,water.id],[water.id]]});
        let check = |v: serde_json::Value| {
            let recipe: Recipe = serde_json::from_value(v)?;
            super::validate(&recipe, &fluids, &Default::default(), &[])
        };
        check(value.clone()).unwrap();
        for (path, bad) in [
            ("/process/energy", json!(0)),
            ("/process/delay", json!("9223372036854775808")),
            ("/process/capacity", json!(0)),
            ("/process/filling/0/0", json!("fluid_missing")),
            ("/inputs/1/choices/0/consume", json!({"kind":"consume"})),
            ("/outputs/0/amount", json!("0")),
            ("/duration", json!("5")),
        ] {
            let mut bad_value = value.clone();
            *bad_value.pointer_mut(path).unwrap() = bad;
            assert!(check(bad_value).is_err(), "{path}");
        }
        value["process"]["delay"] = json!("-2147483648");
        check(value).unwrap();
    }
    #[test]
    fn buildcraft_requires_ordered_allocation_and_fixed_result() {
        use crate::{
            domain::{Domain, Recipe},
            source::Source,
        };
        use serde_json::json;
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = Domain::load(&source).unwrap();
        let mut value = serde_json::to_value(&domain.recipes[0]).unwrap();
        for key in ["duration", "energy", "grid", "magic"] {
            value[key] = serde_json::Value::Null;
        }
        value["process"] = json!({"kind":"buildcraftAssembly","energy":-7});
        let c = json!({"id":domain.items[0].id,"amount":"3","rule":{"kind":"buildcraft","wildcard":false,"subtypes":true},"consume":{"kind":"allocated"},"returns":[]});
        value["inputs"] = json!([{"kind":"item","slot":0,"choices":[c.clone(),c]}]);
        value["outputs"] = json!([{"kind":"item","slot":0,"id":domain.items[1].id,"amount":"2","quantity":null,"change":null,"role":"result","chance":{"numerator":"1","denominator":"1"}}]);
        let check = |v: serde_json::Value| {
            let recipe: Recipe = serde_json::from_value(v)?;
            super::validate(&recipe, &Default::default(), &Default::default(), &[])
        };
        check(value.clone()).unwrap();
        for (key, bad) in [
            ("duration", json!("1")),
            ("energy", json!("7")),
            ("process", serde_json::Value::Null),
        ] {
            let mut altered = value.clone();
            altered[key] = bad;
            assert!(check(altered).is_err());
        }
        let mut bad = value.clone();
        bad["inputs"][0]["choices"][1]["amount"] = json!("1");
        assert!(check(bad).is_err());
        let mut bad = value.clone();
        bad["inputs"][0]["choices"][0]["consume"] = json!({"kind":"consume"});
        assert!(check(bad).is_err());
        let mut bad = value.clone();
        bad["outputs"][0]["chance"]["denominator"] = json!("2");
        assert!(check(bad).is_err());
        value["inputs"] = json!([]);
        check(value).unwrap();
    }
    #[test]
    fn sag_preserves_optional_stock_and_shared_grinding() {
        let value = serde_json::json!({"kind":"sag","energy":1000,"slot":-1,"bonus":true,"earlier":[],"balls":[{"choices":[],"grinding":"2.5","chance":"2.0","power":"0.5","duration":10000}],"blocked":[],"oreBlocked":[]});
        let process: super::Process = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(process).unwrap(), value);
        assert!(serde_json::from_value::<super::Consumption>(
            serde_json::json!({"kind":"reserve"})
        )
        .is_ok());
        assert!(serde_json::from_value::<super::Quantity>(
            serde_json::json!({"kind":"grinding","nominal":"2","threshold":"0.0"})
        )
        .is_ok());
    }

    #[test]
    fn splice_retains_tool_wear_separately_from_allocated_ingredients() {
        let value = serde_json::json!({"kind":"splice","energy":2000,"slots":[5,4,3,2,1,0]});
        let process: super::Process = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(process).unwrap(), value);
        assert!(
            serde_json::from_value::<super::Consumption>(serde_json::json!({"kind":"wear"}))
                .is_ok()
        );
    }

    #[test]
    fn alloy_retains_ordered_allocation_and_shared_roll() {
        let value = serde_json::json!({"kind":"alloy","energy":1200,"slots":[1,0]});
        let process: super::Process = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(process).unwrap(), value);
        assert!(serde_json::from_value::<super::Consumption>(
            serde_json::json!({"kind":"allocated"})
        )
        .is_ok());
        assert!(serde_json::from_value::<super::Quantity>(
            serde_json::json!({"kind":"sharedRoll","nominal":"2","threshold":"0.0"})
        )
        .is_ok());
    }

    #[test]
    fn vat_retains_native_understock_and_zero_yield() {
        let value = serde_json::json!({"kind":"vat","energy":-7,"extra":[{"id":"item_example","rule":{"kind":"wildcard","meta":false,"nbt":true},"amount":0}],"zeroOutput":"fluid_example"});
        let process: super::Process = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(process).unwrap(), value);
        let consume: super::Consumption =
            serde_json::from_value(serde_json::json!({"kind":"upto"})).unwrap();
        assert_eq!(
            serde_json::to_value(consume).unwrap(),
            serde_json::json!({"kind":"upto"})
        );
    }

    #[test]
    fn enchanter_preserves_level_gate_and_signed_xp() {
        let value = serde_json::json!({"kind":"enchanter","level":2,"maxLevel":5,"itemsPerLevel":3,"cost":-7});
        let process: super::Process = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(process).unwrap(), value);
    }

    #[test]
    fn map_scaling_requires_an_explicit_world_process() {
        let value = serde_json::json!({"kind":"mapScaling"});
        assert!(serde_json::from_value::<super::Process>(value.clone()).is_ok());
        assert!(serde_json::from_value::<super::super::Edit>(value).is_ok());
        assert!(serde_json::from_value::<super::Process>(
            serde_json::json!({"kind":"mapScaling","finalMapId":37})
        )
        .is_err());
    }

    #[test]
    fn runic_costs_preserve_native_rounding_and_large_repetition() {
        for (charge, expected) in [
            (-8, (1, 0, 1)),
            (-5, (1, 1, 3)),
            (0, (1, 32, 5)),
            (3, (4, 256, 6)),
            (30, (31, i32::MAX, 20)),
            (i32::MAX, (2147483648, i32::MAX, 1073741828)),
        ] {
            assert_eq!(super::runic_costs(charge), expected);
        }
        assert!(serde_json::from_value::<super::Process>(
            serde_json::json!({"kind":"runic","charge":3})
        )
        .is_ok());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HarmonyMode {
    Single,
    Parallel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HarmonyOutcome {
    Success,
    Failure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InscriberMode {
    Inscribe,
    Press,
}

/// Native shared process, not independent probabilities on individual outputs.
/// See docs/harmony.md for state, energy, rounding and correlated yield rules.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Process {
    /// Native shared stock/selection, conditional output and powered work steps.
    #[serde(rename = "forestrySqueezer")]
    ForestrySqueezer {
        program: String,
        selector: super::forestry::SqueezerSelector,
        time: i32,
        chance: super::Chance,
    },
    /// Retained native recipe selection, periodic preflight and ordered NBT transformation.
    #[serde(rename = "buildcraftIntegration")]
    BuildcraftIntegration {
        rule: super::integration::IntegrationRule,
    },
    /// IC2 heat/air checkpoints, persistent progress and native output-space behavior.
    #[serde(rename = "ic2Blast")]
    Ic2Blast {
        heat: i32,
        /// Per input slot and choice: container left in that input slot when
        /// consuming a single item. Stacks >1 with a container are not consumed.
        /// Air choice returns still describe the separate air-output slot.
        containers: [Vec<Option<super::Stack>>; 2],
    },
    /// Output prototype receives ExtraUtilities 1.2.12's conditional world-clock
    /// tags at completion, not at export time. See docs/casting.md.
    #[serde(rename = "unstableCasting")]
    UnstableCasting {},
    /// BuildCraft's cached refinery selection, retry timer and sequential tank drain.
    #[serde(rename = "buildcraftRefinery")]
    BuildcraftRefinery {
        energy: i32,
        delay: String,
        capacity: i32,
        earlier: Vec<Vec<super::Stack>>,
        filling: [Vec<String>; 2],
    },
    /// Selected BuildCraft assembly plan; ordered greedy allocation and signed RF budget.
    #[serde(rename = "buildcraftAssembly")]
    BuildcraftAssembly { energy: i32 },
    /// Railcraft shared ordered crafting registry and native reserve/power gates.
    Rolling {
        powered: bool,
        earlier: Vec<super::CraftingSelector>,
    },
    /// Extra Utilities QED ordered crafting registry. A completed craft drains
    /// the native default Ender Flux buffer and one item from every occupied cell.
    Qed {
        #[serde(rename = "enderFlux")]
        ender_flux: String,
        earlier: Vec<super::CraftingSelector>,
    },
    /// Ordinary GalaxySpace machine crafting after native repair priority has
    /// been ruled out. Timing remains dependent on the external power tier.
    #[serde(rename = "galaxyspace-assembly")]
    GalaxyspaceAssembly {
        earlier: Vec<super::CraftingSelector>,
    },
    /// Botania's ordinary floating-flower crafting within a 2x2 or 3x3 grid.
    /// Special inputs bind in increasing physical cell order. Only a missing
    /// or string `type` on the last special input is in the admitted domain.
    #[serde(rename = "floatingFlowers")]
    FloatingFlowers { special: Vec<u32> },
    /// Native two-slot Soul Binder selection, XP debit and last-vessel completion.
    Soul {
        energy: i32,
        levels: i32,
        experience: i32,
        capacity: i32,
        drains: bool,
        spawner: bool,
        earlier: Vec<super::SoulSelector>,
    },
    /// Native SAG task and grinding-ball lifecycle; see docs/grinding.md.
    Sag {
        energy: i32,
        slot: i32,
        bonus: bool,
        earlier: Vec<super::GrindingRequirement>,
        balls: Vec<super::GrindingBall>,
        blocked: Vec<super::MatchCase>,
        #[serde(rename = "oreBlocked")]
        ore_blocked: Vec<super::MatchCase>,
    },
    /// Registered EnderIO alloy matching and separate consumption traversals.
    Alloy { energy: i32, slots: Vec<i32> },
    /// Six ingredients plus axe/shears slots; native conditional tool wear at completion.
    Splice { energy: i32, slots: Vec<i32> },
    /// EnderIO precomputed fluid pair and ordered item consumption; see docs/vat.md.
    Vat {
        energy: i32,
        extra: Vec<VatConsumption>,
        #[serde(rename = "zeroOutput")]
        zero_output: Option<String>,
    },
    /// EnderIO manual enchanter. The offered material count determines the level;
    /// cost is signed native XP levels, not energy. See docs/enchanter.md.
    Enchanter {
        level: u16,
        #[serde(rename = "maxLevel")]
        max_level: u16,
        #[serde(rename = "itemsPerLevel")]
        items_per_level: u32,
        cost: i32,
    },
    /// AE2 registered recipe, after the name-press guard and earlier registry
    /// rows. top absent has a native empty-slot predicate; see docs/inscriber.md.
    Inscriber {
        mode: InscriberMode,
        top: Option<String>,
        bottom: Option<String>,
        #[serde(rename = "namePress")]
        name_press: Option<String>,
    },
    /// Vanilla map extension. Input eligibility depends on world MapData (scale < 4).
    /// The output is a pre-onCreated sample; the world allocates the final map ID.
    /// Missing data may be initialized server-side. See docs/maps.md.
    #[serde(rename = "mapScaling")]
    MapScaling {},
    /// Dynamic TC runic augmentation. Charge, component multiplicity and magic
    /// costs are an observed sample. Actual costs use the offered IRunicArmor's
    /// native final charge, including nested upgrades; see docs/runic.md.
    Runic { charge: i32 },
    Harmony {
        mode: HarmonyMode,
        hydrogen: String,
        helium: String,
        ticks: String,
        #[serde(rename = "startEu")]
        start_eu: String,
        #[serde(rename = "outputEu")]
        output_eu: String,
        /// Round-trip decimal representation of the native binary64 base chance.
        chance: String,
        #[serde(rename = "rocketTier")]
        rocket_tier: u8,
        #[serde(rename = "compressionTier")]
        compression_tier: u8,
    },
}

#[cfg(test)]
mod soul_contract_test {
    #[test]
    fn rolling_retains_cross_category_selectors() {
        let value = serde_json::json!({"kind":"rolling","powered":false,"earlier":[{"grid":{"width":2,"height":1,"cells":[0,null],"mirror":true},"inputs":[[{"id":"template","rule":{"kind":"wildcard","meta":true,"nbt":true}}]]}]});
        let process: super::Process = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(process).unwrap(), value);
    }
    #[test]
    fn soul_preserves_raw_experience_and_native_predicate() {
        let rule: super::Match = serde_json::from_value(serde_json::json!({"kind":"soul","filter":{"vessel":"vial","names":[null,"Forbidden"],"exclude":true}})).unwrap();
        assert_eq!(
            serde_json::to_value(rule).unwrap()["filter"]["names"][0],
            serde_json::Value::Null
        );
        let process: super::Process = serde_json::from_value(serde_json::json!({"kind":"soul","energy":1000,"levels":16,"experience":272,"capacity":825,"drains":false,"spawner":false,"earlier":[]})).unwrap();
        assert_eq!(serde_json::to_value(process).unwrap()["experience"], 272);
    }
}

#[cfg(test)]
mod qed_contract_test {
    use super::*;
    use crate::{domain::Domain, source::Source};
    use serde_json::{json, Value};

    fn fixture() -> (Domain, Value) {
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = Domain::load(&source).unwrap();
        let recipe = domain
            .recipes
            .iter()
            .find(|r| matches!(r.process, Some(Process::Rolling { .. })))
            .unwrap();
        let mut value = serde_json::to_value(recipe).unwrap();
        let earlier = value["process"]["earlier"].clone();
        value["process"] = json!({"kind":"qed","enderFlux":"20000","earlier":earlier});
        value["source"]["handler"] = json!("com.rwtema.extrautils.nei.EnderConstructorHandler");
        (domain, value)
    }

    #[test]
    fn galaxyspace_preserves_priority_without_inventing_fixed_timing() {
        let (domain, mut value) = fixture();
        let earlier = value["process"]["earlier"].clone();
        value["process"] = json!({"kind":"galaxyspace-assembly","earlier":earlier});
        value["source"]["handler"] = json!("galaxyspace.core.nei.AssemblyMachineRecipeHandler");
        let items = domain
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        for shaped in [true, false] {
            let mut current = value.clone();
            if !shaped {
                current["grid"] = Value::Null;
            }
            let recipe: Recipe = serde_json::from_value(current.clone()).unwrap();
            validate(&recipe, &BTreeMap::new(), &items, &[]).unwrap();
            assert_eq!(
                serde_json::to_value(recipe.process).unwrap(),
                current["process"]
            );
            for (pointer, replacement) in [
                ("/duration", json!("200")),
                ("/energy", json!("75")),
                ("/inputs/0/choices/0/amount", json!("2")),
                ("/inputs/0/choices/0/consume", json!({"kind":"keep"})),
                ("/process/earlier/0/inputs/0/0/id", json!("missing")),
                ("/process/earlier/0/grid/cells", json!([0, 0])),
            ] {
                let mut wrong = current.clone();
                *wrong.pointer_mut(pointer).unwrap() = replacement;
                let recipe: Recipe = serde_json::from_value(wrong).unwrap();
                assert!(
                    validate(&recipe, &BTreeMap::new(), &items, &[]).is_err(),
                    "accepted {pointer}"
                );
            }
        }
    }

    #[test]
    fn qed_retains_ordered_shaped_and_shapeless_selectors() {
        let (domain, mut value) = fixture();
        let mut shapeless = value["process"]["earlier"][0].clone();
        shapeless["grid"] = Value::Null;
        value["process"]["earlier"]
            .as_array_mut()
            .unwrap()
            .push(shapeless);
        let recipe: Recipe = serde_json::from_value(value.clone()).unwrap();
        let items = domain
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        validate(&recipe, &BTreeMap::new(), &items, &[]).unwrap();
        assert_eq!(
            serde_json::to_value(recipe.process).unwrap(),
            value["process"]
        );
        assert_eq!(
            value["process"]["earlier"][0]["grid"]["cells"],
            json!([0, null])
        );
        assert_eq!(value["process"]["earlier"][0]["grid"]["mirror"], true);
    }

    #[test]
    fn qed_rejects_invented_power_consumption_returns_and_invalid_priority_rules() {
        let (domain, value) = fixture();
        let items = domain
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        let base: Recipe = serde_json::from_value(value.clone()).unwrap();
        validate(&base, &BTreeMap::new(), &items, &[]).unwrap();
        for (pointer, replacement) in [
            ("/process/enderFlux", json!("1")),
            ("/duration", json!("100")),
            ("/energy", json!("200")),
            ("/inputs/0/choices/0/amount", json!("2")),
            ("/inputs/0/choices/0/consume", json!({"kind":"keep"})),
            ("/inputs/0/choices/0/rule/nbt", json!(false)),
            ("/process/earlier/0/inputs/0/0/id", json!("missing")),
            ("/process/earlier/0/grid/cells", json!([0, 0])),
            ("/grid", Value::Null),
        ] {
            let mut wrong = value.clone();
            *wrong.pointer_mut(pointer).unwrap() = replacement;
            let recipe: Recipe = serde_json::from_value(wrong).unwrap();
            assert!(
                validate(&recipe, &BTreeMap::new(), &items, &[]).is_err(),
                "accepted {pointer}"
            );
        }
        let mut wrong = base;
        wrong.inputs[0].choices[0]
            .returns
            .push(super::super::Remainder {
                kind: Kind::Item,
                id: wrong.outputs[0].id.clone(),
                amount: "1".into(),
            });
        assert!(validate(&wrong, &BTreeMap::new(), &items, &[]).is_err());
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VatConsumption {
    pub id: String,
    pub rule: Match,
    pub amount: i32,
}

impl Process {
    pub fn max_parallel(&self) -> i64 {
        match self {
            Self::Harmony {
                mode: HarmonyMode::Parallel,
                ..
            } => 1_048_576,
            _ => 1,
        }
    }

    pub(super) fn validate_parameters(&self) -> Result<()> {
        if let Self::Qed { ender_flux, .. } = self {
            ensure!(
                ender_flux == "20000",
                "QED requires the native default Ender Flux buffer"
            );
            return Ok(());
        }
        if let Self::Enchanter {
            level,
            max_level,
            items_per_level,
            ..
        } = self
        {
            ensure!(
                *level > 0
                    && *level <= *max_level
                    && *max_level <= 32767
                    && *items_per_level > 0
                    && u64::from(*level) * u64::from(*items_per_level) <= 64,
                "invalid enchanter level or input count"
            );
            return Ok(());
        }
        let Self::Harmony {
            hydrogen,
            helium,
            ticks,
            start_eu,
            output_eu,
            chance,
            rocket_tier,
            compression_tier,
            ..
        } = self
        else {
            return Ok(());
        };
        for value in [hydrogen, helium, ticks, start_eu, output_eu] {
            integer(value, 1, i64::MAX)?;
        }
        let rate: f64 = chance.parse().context("invalid harmony chance")?;
        ensure!(
            rate.is_finite() && (0.0..=1.0).contains(&rate),
            "invalid harmony chance"
        );
        ensure!(
            *rocket_tier <= 9 && *compression_tier == (*rocket_tier).min(8),
            "invalid harmony tiers"
        );
        Ok(())
    }
}

pub(super) fn validate(
    recipe: &Recipe,
    fluids: &BTreeMap<&str, &Fluid>,
    items: &BTreeMap<&str, &super::Item>,
    aspects: &[super::Aspect],
) -> Result<()> {
    ensure!(
        matches!(recipe.process, Some(Process::ForestrySqueezer { .. }))
            || !recipe
                .inputs
                .iter()
                .flat_map(|i| &i.choices)
                .any(|c| matches!(c.rule, Match::Forestry)),
        "Forestry requirements need their shared machine process"
    );
    ensure!(
        matches!(recipe.process, Some(Process::BuildcraftIntegration { .. }))
            || !recipe
                .inputs
                .iter()
                .flat_map(|i| &i.choices)
                .any(|c| matches!(c.rule, Match::Integration)),
        "integration predicate requires its machine process"
    );
    ensure!(
        matches!(recipe.process, Some(Process::Ic2Blast { .. }))
            || !recipe
                .inputs
                .iter()
                .flat_map(|i| &i.choices)
                .any(|c| matches!(c.consume, Consumption::Staged)),
        "staged consumption requires the IC2 blast process"
    );
    ensure!(
        matches!(recipe.process, Some(Process::Soul { .. }))
            || !recipe
                .inputs
                .iter()
                .flat_map(|i| &i.choices)
                .any(|c| matches!(c.rule, Match::Soul { .. })),
        "soul predicate requires a Soul Binder process"
    );
    ensure!(
        matches!(recipe.process, Some(Process::BuildcraftAssembly { .. }))
            || !recipe
                .inputs
                .iter()
                .flat_map(|i| &i.choices)
                .any(|c| matches!(c.rule, Match::Buildcraft { .. })),
        "BuildCraft predicate requires its native machine process"
    );
    ensure!(
        matches!(
            recipe.process,
            Some(
                Process::Alloy { .. }
                    | Process::Splice { .. }
                    | Process::Sag { .. }
                    | Process::BuildcraftAssembly { .. }
                    | Process::BuildcraftRefinery { .. }
                    | Process::BuildcraftIntegration { .. }
                    | Process::ForestrySqueezer { .. }
            )
        ) || !recipe
            .inputs
            .iter()
            .flat_map(|i| &i.choices)
            .any(|c| matches!(c.consume, Consumption::Allocated)),
        "allocated consumption requires a native assembly process"
    );
    ensure!(
        matches!(recipe.process, Some(Process::Sag { .. }))
            || !recipe
                .inputs
                .iter()
                .flat_map(|i| &i.choices)
                .any(|c| matches!(c.consume, Consumption::Reserve)),
        "optional ball stock requires a SAG process"
    );
    ensure!(
        matches!(recipe.process, Some(Process::Splice { .. }))
            || !recipe
                .inputs
                .iter()
                .flat_map(|i| &i.choices)
                .any(|c| matches!(c.consume, Consumption::Wear)),
        "conditional tool wear requires a native splice process"
    );
    ensure!(
        matches!(recipe.process, Some(Process::Vat { .. }))
            || !recipe
                .inputs
                .iter()
                .flat_map(|i| &i.choices)
                .any(|c| matches!(c.consume, Consumption::Upto)),
        "up-to consumption requires a Vat process"
    );
    ensure!(
        !recipe
            .inputs
            .iter()
            .flat_map(|i| &i.choices)
            .any(|c| matches!(c.rule, Match::Ae))
            || matches!(recipe.process, Some(Process::Inscriber { .. })),
        "AE precise inputs require an inscriber process"
    );
    let Some(process) = &recipe.process else {
        ensure!(
            !recipe.inputs.iter().any(|i| i
                .choices
                .iter()
                .any(|c| matches!(c.consume, Consumption::Buffer | Consumption::Pedestals))),
            "buffer consumption requires a native process"
        );
        return Ok(());
    };
    process.validate_parameters()?;
    if matches!(process, Process::ForestrySqueezer { .. }) {
        ensure!(
            recipe.duration.is_none()
                && recipe.energy.is_none()
                && recipe.grid.is_none()
                && recipe.magic.is_none(),
            "Squeezer cannot declare fixed duration, EU/t or crafting semantics"
        );
        // Cross-check against the already-reconstructed program in Domain::validate.
        return Ok(());
    }
    if let Process::BuildcraftIntegration { rule } = process {
        return super::integration::validate_recipe(recipe, rule);
    }
    if let Process::Ic2Blast { containers, .. } = process {
        return validate_blast(recipe, containers, items);
    }
    if matches!(process, Process::UnstableCasting {}) {
        ensure!(
            recipe.source.handler == "tconstruct.plugins.nei.RecipeHandlerCastingTable"
                && recipe.energy.is_none()
                && recipe.grid.is_none()
                && recipe.magic.is_none()
                && recipe.outputs.len() == 1
                && (1..=2).contains(&recipe.inputs.len()),
            "invalid unstable casting shape"
        );
        integer(
            recipe
                .duration
                .as_deref()
                .context("missing casting cooling time")?,
            1,
            i32::MAX.into(),
        )?;
        ensure!(
            recipe
                .inputs
                .iter()
                .filter(|i| i.kind == Kind::Fluid)
                .count()
                == 1,
            "unstable casting requires one fluid input"
        );
        let output = &recipe.outputs[0];
        ensure!(
            output.kind == Kind::Item
                && output.slot == 0
                && output.change.is_none()
                && output.quantity.is_none()
                && matches!(output.role, OutputRole::Result)
                && output.chance.numerator == "1"
                && output.chance.denominator == "1",
            "invalid unstable casting output prototype"
        );
        integer(
            output.amount.as_deref().context("missing casting amount")?,
            1,
            i32::MAX.into(),
        )?;
        return Ok(());
    }
    if let Process::BuildcraftRefinery {
        energy,
        delay,
        capacity,
        earlier,
        filling,
    } = process
    {
        return validate_refining(recipe, *energy, delay, *capacity, earlier, filling, fluids);
    }
    if matches!(process, Process::BuildcraftAssembly { .. }) {
        return validate_buildcraft(recipe);
    }
    if let Process::Rolling { earlier, .. } = process {
        return super::rolling::validate(recipe, earlier, items);
    }
    if let Process::GalaxyspaceAssembly { earlier } = process {
        return super::rolling::validate(recipe, earlier, items);
    }
    if let Process::FloatingFlowers { special } = process {
        return super::floating::validate(recipe, special, items);
    }
    if let Process::Qed { earlier, .. } = process {
        ensure!(
            recipe.grid.is_some(),
            "QED handler only publishes shaped recipes"
        );
        return super::rolling::validate(recipe, earlier, items);
    }
    if matches!(process, Process::Sag { .. }) {
        return super::grinding::validate(recipe, items);
    }
    if matches!(process, Process::Soul { .. }) {
        return super::soul::validate(recipe, items);
    }
    if let Process::Alloy { slots, .. } = process {
        return validate_assembly(recipe, slots, false);
    }
    if let Process::Splice { slots, .. } = process {
        return validate_assembly(recipe, slots, true);
    }
    if let Process::Vat {
        extra, zero_output, ..
    } = process
    {
        return validate_vat(recipe, extra, zero_output.as_deref(), fluids, items);
    }
    if let Process::Enchanter {
        level,
        items_per_level,
        ..
    } = process
    {
        return validate_enchanter(recipe, *level, *items_per_level, items);
    }
    if let Process::Inscriber {
        mode,
        top,
        bottom,
        name_press,
    } = process
    {
        return validate_inscriber(
            recipe,
            *mode,
            top.as_deref(),
            bottom.as_deref(),
            name_press.as_deref(),
            items,
        );
    }
    if matches!(process, Process::MapScaling {}) {
        return validate_map(recipe, items);
    }
    if let Process::Runic { charge } = process {
        return validate_runic(recipe, *charge, items, aspects);
    }
    let Process::Harmony {
        mode,
        hydrogen,
        helium,
        rocket_tier,
        ..
    } = process
    else {
        unreachable!()
    };
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none(),
        "harmony process cannot have fixed timing, energy or crafting semantics"
    );
    ensure!(
        recipe.inputs.len() == if *mode == HarmonyMode::Single { 3 } else { 2 },
        "invalid harmony inputs"
    );
    let trigger = recipe
        .inputs
        .iter()
        .find(|i| i.kind == Kind::Item && i.slot == 0)
        .context("harmony trigger missing")?;
    ensure!(
        !trigger.choices.is_empty(),
        "harmony requires a native block trigger"
    );
    for c in &trigger.choices {
        ensure!(
            c.amount == "1"
                && matches!(c.consume, Consumption::Keep)
                && c.returns.is_empty()
                && matches!(
                    c.rule,
                    Match::Wildcard {
                        meta: true,
                        nbt: true
                    }
                ),
            "invalid harmony trigger semantics"
        );
    }
    let h = integer(hydrogen, 1, i64::MAX)?;
    let he = integer(helium, 1, i64::MAX)?;
    let required = if *mode == HarmonyMode::Single {
        vec![(0, "hydrogen", h), (1, "helium", he)]
    } else {
        vec![(
            2,
            "rawstarmatter",
            ((he as f64 * 1.24e-5 * 8.0).ceil() as i64).max(1),
        )]
    };
    for (slot, registry, amount) in required {
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.kind == Kind::Fluid && i.slot == slot)
            .context("harmony buffer missing")?;
        ensure!(input.choices.len() == 1, "harmony buffer has alternatives");
        let c = &input.choices[0];
        let fluid = fluids
            .get(c.id.as_str())
            .context("harmony buffer fluid missing")?;
        ensure!(
            fluid.registry == registry
                && fluid.nbt.is_none()
                && c.amount == amount.to_string()
                && matches!(c.consume, Consumption::Buffer)
                && c.returns.is_empty()
                && matches!(
                    c.rule,
                    Match::Wildcard {
                        meta: false,
                        nbt: true
                    }
                ),
            "invalid harmony buffer semantics"
        );
    }
    let mut failures = 0;
    let mut successes = 0;
    for output in &recipe.outputs {
        ensure!(
            matches!(output.role, OutputRole::Result),
            "harmony output is not a result"
        );
        let Some(Quantity::Harmony { outcome, nominal }) = &output.quantity else {
            anyhow::bail!("harmony output lacks shared quantity");
        };
        super::quantity_bounds(recipe, output)?;
        match outcome {
            HarmonyOutcome::Success => successes += 1,
            HarmonyOutcome::Failure => {
                failures += 1;
                ensure!(
                    output.kind == Kind::Fluid
                        && nominal == &(14400_i64 * (1_i64 << (rocket_tier + 1))).to_string(),
                    "invalid harmony failure quantity"
                );
                let fluid = fluids
                    .get(output.id.as_str())
                    .context("harmony failure fluid missing")?;
                ensure!(
                    fluid.registry == "molten.spacetime" && fluid.nbt.is_none(),
                    "invalid harmony failure fluid"
                );
            }
        }
    }
    ensure!(
        failures == 1 && successes > 0,
        "harmony requires normal outputs and exactly one failure output"
    );
    Ok(())
}

fn validate_assembly(recipe: &Recipe, slots: &[i32], splice: bool) -> Result<()> {
    let max = if splice { 6 } else { 3 };
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none()
            && !slots.is_empty()
            && slots.len() <= max
            && slots.len() + if splice { 2 } else { 0 } == recipe.inputs.len()
            && slots.iter().all(|s| (-1..max as i32).contains(s))
            && !recipe.outputs.is_empty()
            && recipe.outputs.len() <= 128,
        "invalid native assembly shape"
    );
    for (index, input) in recipe.inputs.iter().enumerate() {
        if splice && index >= slots.len() {
            ensure!(
                input.kind == Kind::Item
                    && input.slot == (6 + index - slots.len()) as u32
                    && !input.choices.is_empty()
                    && input.choices.iter().all(|c| c.amount == "1"
                        && c.returns.is_empty()
                        && matches!(c.consume, Consumption::Wear)
                        && matches!(
                            c.rule,
                            Match::Wildcard {
                                meta: true,
                                nbt: true
                            }
                        )),
                "invalid splice axe/shears requirement"
            );
            continue;
        }
        ensure!(
            input.kind == Kind::Item && input.slot == index as u32 && !input.choices.is_empty(),
            "invalid alloy requirement order"
        );
        let amount = &input.choices[0].amount;
        integer(amount, 1, i64::from(i32::MAX))?;
        for c in &input.choices {
            ensure!(
                &c.amount == amount
                    && matches!(c.consume, Consumption::Allocated)
                    && matches!(c.rule, Match::Wildcard { nbt: true, .. })
                    && c.returns.is_empty(),
                "invalid alloy requirement semantics"
            );
        }
    }
    for (index, output) in recipe.outputs.iter().enumerate() {
        ensure!(
            output.kind == Kind::Item
                && output.slot == index as u32
                && matches!(output.role, OutputRole::Result)
                && matches!(output.quantity, Some(Quantity::SharedRoll { .. })),
            "alloy result lacks its shared roll or ordered slot"
        );
        super::quantity_bounds(recipe, output)?;
    }
    Ok(())
}

fn validate_vat(
    recipe: &Recipe,
    extra: &[VatConsumption],
    zero_output: Option<&str>,
    fluids: &BTreeMap<&str, &Fluid>,
    items: &BTreeMap<&str, &super::Item>,
) -> Result<()> {
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none()
            && (2..=3).contains(&recipe.inputs.len()),
        "invalid Vat process shape"
    );
    let count = recipe.inputs.len() - 1;
    for slot in 0..count {
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.kind == Kind::Item && i.slot == slot as u32)
            .context("missing Vat reagent slot")?;
        ensure!(!input.choices.is_empty(), "empty Vat reagent");
        for c in &input.choices {
            integer(&c.amount, 1, i64::from(i32::MAX))?;
            ensure!(
                c.returns.is_empty()
                    && (matches!(c.consume, Consumption::Upto)
                        || matches!(c.consume, Consumption::Keep) && c.amount == "1"),
                "invalid Vat reagent consumption"
            );
            let base = match &c.rule {
                Match::Except { base, exclude } => {
                    ensure!(
                        exclude
                            .iter()
                            .all(|c| matches!(c.rule, Match::Wildcard { nbt: true, .. })),
                        "invalid Vat priority predicate"
                    );
                    base.as_ref()
                }
                other => other,
            };
            ensure!(
                matches!(base, Match::Wildcard { nbt: true, .. }),
                "invalid Vat reagent predicate"
            );
        }
    }
    let input = recipe
        .inputs
        .iter()
        .find(|i| i.kind == Kind::Fluid && i.slot == 0)
        .context("missing Vat input fluid")?;
    ensure!(input.choices.len() == 1, "Vat requires one table fluid");
    let c = &input.choices[0];
    integer(&c.amount, 1, 8000)?;
    ensure!(
        matches!(c.rule, Match::Exact)
            && c.returns.is_empty()
            && (matches!(c.consume, Consumption::Consume)
                || matches!(c.consume, Consumption::Keep) && c.amount == "1"),
        "invalid Vat fluid consumption"
    );
    ensure!(
        extra.len() <= 4096
            && if count == 1 {
                !extra.is_empty()
            } else {
                extra.is_empty()
            },
        "invalid Vat optional slot"
    );
    for c in extra {
        let item = items
            .get(c.id.as_str())
            .context("missing Vat optional reagent")?;
        ensure!(
            matches!(c.rule, Match::Wildcard { nbt: true, .. }),
            "invalid Vat optional predicate"
        );
        c.rule.validate_item(item, items, false, false)?;
    }
    if let Some(id) = zero_output {
        ensure!(
            fluids.contains_key(id) && recipe.outputs.is_empty(),
            "invalid Vat zero output reference"
        );
    } else {
        ensure!(recipe.outputs.len() == 1, "Vat requires its table output");
        let output = &recipe.outputs[0];
        ensure!(
            output.kind == Kind::Fluid
                && output.slot == 0
                && output.quantity.is_none()
                && output.change.is_none()
                && matches!(output.role, OutputRole::Result)
                && output.chance.numerator == "1"
                && output.chance.denominator == "1",
            "invalid Vat fluid result"
        );
        integer(
            output
                .amount
                .as_deref()
                .context("missing Vat output amount")?,
            1,
            8000,
        )?;
    }
    Ok(())
}

fn validate_enchanter(
    recipe: &Recipe,
    level: u16,
    items_per_level: u32,
    items: &BTreeMap<&str, &super::Item>,
) -> Result<()> {
    use crate::identity::Nbt;
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none()
            && recipe.inputs.len() == 2
            && recipe.outputs.len() == 1,
        "invalid enchanter process shape"
    );
    for slot in 0..2 {
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.slot == slot && i.kind == Kind::Item)
            .context("missing enchanter input")?;
        let amount = if slot == 0 {
            1
        } else {
            u32::from(level) * items_per_level
        };
        ensure!(!input.choices.is_empty(), "empty enchanter input");
        for choice in &input.choices {
            ensure!(
                choice.amount == amount.to_string()
                    && matches!(choice.consume, Consumption::Consume)
                    && choice.returns.is_empty(),
                "invalid enchanter material consumption"
            );
            if slot == 0 {
                let book = items
                    .get(choice.id.as_str())
                    .context("missing writable book")?;
                ensure!(
                    book.registry == "minecraft:writable_book"
                        && matches!(
                            choice.rule,
                            Match::Wildcard {
                                meta: true,
                                nbt: true
                            }
                        ),
                    "invalid enchanter book predicate"
                );
            } else {
                let base = match &choice.rule {
                    Match::Except { base, .. } => base.as_ref(),
                    other => other,
                };
                ensure!(
                    matches!(base, Match::Wildcard { nbt: true, .. }),
                    "invalid enchanter material predicate"
                );
                if let Match::Except { exclude, .. } = &choice.rule {
                    ensure!(
                        exclude
                            .iter()
                            .all(|c| matches!(c.rule, Match::Wildcard { nbt: true, .. })),
                        "invalid enchanter priority predicate"
                    );
                }
            }
        }
    }
    let output = &recipe.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && output.amount.as_deref() == Some("1")
            && output.quantity.is_none()
            && output.change.is_none()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "invalid enchanter result"
    );
    let book = items
        .get(output.id.as_str())
        .context("missing enchanted book")?;
    ensure!(
        book.registry == "minecraft:enchanted_book" && book.meta == 0,
        "invalid enchanted book identity"
    );
    let Some(Nbt::Compound { value: root }) = &book.nbt else {
        anyhow::bail!("missing enchantment NBT")
    };
    let Some(Nbt::List {
        element,
        value: list,
    }) = root.get("StoredEnchantments")
    else {
        anyhow::bail!("missing stored enchantment")
    };
    ensure!(
        root.len() == 1 && element == "compound" && list.len() == 1,
        "invalid stored enchantment list"
    );
    let Nbt::Compound { value: enchantment } = &list[0] else {
        anyhow::bail!("invalid enchantment tag")
    };
    ensure!(
        enchantment.len() == 2
            && matches!(enchantment.get("id"), Some(Nbt::Short { .. }))
            && matches!(enchantment.get("lvl"), Some(Nbt::Short {value}) if value == &level.to_string()),
        "enchanted book level differs from process"
    );
    Ok(())
}

fn validate_blast(
    recipe: &Recipe,
    containers: &[Vec<Option<super::Stack>>; 2],
    items: &BTreeMap<&str, &super::Item>,
) -> Result<()> {
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none()
            && recipe.inputs.len() == 2
            && (1..=2).contains(&recipe.outputs.len()),
        "invalid IC2 blast shape or invented fixed time/energy"
    );
    for (slot, input) in recipe.inputs.iter().enumerate() {
        ensure!(
            containers[slot].len() == input.choices.len(),
            "blast container choices are misaligned"
        );
        for container in containers[slot].iter().flatten() {
            ensure!(
                items.contains_key(container.id.as_str()),
                "blast container references an unknown item"
            );
            integer(&container.amount, 1, i32::MAX.into())?;
        }
        ensure!(
            input.kind == Kind::Item && input.slot == slot as u32 && !input.choices.is_empty(),
            "invalid blast input slot"
        );
        for c in &input.choices {
            integer(&c.amount, 1, i32::MAX.into())?;
            ensure!(
                matches!(c.consume, Consumption::Staged),
                "blast input must retain stage consumption"
            );
            if slot == 0 {
                ensure!(
                    matches!(c.rule, Match::Wildcard { nbt: true, .. })
                        && c.returns.is_empty()
                        && c.amount == input.choices[0].amount,
                    "invalid blast main-input gate"
                );
            } else {
                ensure!(
                    input.choices.len() == 1
                        && c.amount == "1"
                        && matches!(c.rule, Match::Untagged { .. })
                        && c.returns.len() == 1
                        && c.returns[0].kind == Kind::Item,
                    "invalid blast air checkpoint"
                );
                integer(&c.returns[0].amount, 1, i32::MAX.into())?;
            }
        }
    }
    for (slot, output) in recipe.outputs.iter().enumerate() {
        ensure!(
            output.kind == Kind::Item
                && output.slot == slot as u32
                && output.change.is_none()
                && matches!(output.role, OutputRole::Result)
                && output.chance.numerator == "1"
                && output.chance.denominator == "1",
            "invalid blast product"
        );
        if slot == 0 {
            ensure!(
                output.quantity.is_none(),
                "blast main result is fixed at completion"
            );
            integer(
                output
                    .amount
                    .as_deref()
                    .context("missing blast main result")?,
                1,
                i32::MAX.into(),
            )?;
        } else {
            let Some(Quantity::Potential {
                stat,
                nominal,
                sample,
                condition,
                parameters,
            }) = &output.quantity
            else {
                anyhow::bail!("slag requires native storage-dependent quantity")
            };
            ensure!(
                output.amount.is_none()
                    && stat == "ic2:slagSpace"
                    && sample.is_none()
                    && condition.is_none()
                    && parameters.is_none(),
                "invalid blast slag-space rule"
            );
            integer(nominal, 1, i32::MAX.into())?;
        }
    }
    Ok(())
}

fn validate_refining(
    recipe: &Recipe,
    energy: i32,
    delay: &str,
    capacity: i32,
    earlier: &[Vec<super::Stack>],
    filling: &[Vec<String>; 2],
    fluids: &BTreeMap<&str, &Fluid>,
) -> Result<()> {
    ensure!(
        energy > 0 && capacity > 0,
        "refinery cannot execute a nonpositive energy gate"
    );
    integer(delay, i64::MIN, i64::MAX)?;
    ensure!(
        recipe.magic.is_none()
            && recipe.grid.is_none()
            && recipe.duration.is_none()
            && recipe.energy.is_none()
            && !recipe.inputs.is_empty()
            && recipe.inputs.len() <= 4096
            && recipe.outputs.len() == 1
            && earlier.len() <= 4096,
        "invalid BuildCraft refinery shape"
    );
    let fluid = |id: &str| -> Result<()> {
        ensure!(
            fluids
                .get(id)
                .context("missing refinery fluid")?
                .nbt
                .is_none(),
            "tagged refinery predicates require native NBT comparison"
        );
        Ok(())
    };
    for (slot, input) in recipe.inputs.iter().enumerate() {
        ensure!(
            input.kind == Kind::Fluid && input.slot == slot as u32 && input.choices.len() == 1,
            "invalid refinery ordered requirement"
        );
        let c = &input.choices[0];
        fluid(&c.id)?;
        integer(&c.amount, 1, i32::MAX.into())?;
        ensure!(
            matches!(c.rule, Match::Exact)
                && matches!(c.consume, Consumption::Allocated)
                && c.returns.is_empty(),
            "refinery requires native sequential fluid withdrawal"
        );
    }
    let output = &recipe.outputs[0];
    fluid(&output.id)?;
    ensure!(
        output.kind == Kind::Fluid
            && output.slot == 0
            && output.quantity.is_none()
            && output.change.is_none()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "invalid refinery output"
    );
    integer(
        output
            .amount
            .as_deref()
            .context("missing refinery result amount")?,
        1,
        i32::MAX.into(),
    )?;
    let mut count = 0;
    for prior in earlier {
        ensure!(
            !prior.is_empty() && prior.len() <= 4096,
            "invalid refinery preview selector"
        );
        count += prior.len();
        ensure!(count <= 65536, "refinery priority budget exceeded");
        for required in prior {
            fluid(&required.id)?;
            integer(&required.amount, 1, i32::MAX.into())?;
        }
    }
    for list in filling {
        ensure!(list.len() <= 65536, "refinery filling budget exceeded");
        let mut seen = std::collections::BTreeSet::new();
        for id in list {
            fluid(id)?;
            ensure!(seen.insert(id), "duplicate refinery filling predicate");
        }
    }
    Ok(())
}

fn validate_buildcraft(recipe: &Recipe) -> Result<()> {
    ensure!(
        recipe.magic.is_none()
            && recipe.grid.is_none()
            && recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.inputs.len() <= 4096
            && recipe.outputs.len() == 1,
        "invalid BuildCraft assembly shape"
    );
    let mut count = 0;
    for (slot, input) in recipe.inputs.iter().enumerate() {
        ensure!(
            input.kind == Kind::Item && input.slot == slot as u32 && !input.choices.is_empty(),
            "invalid BuildCraft ordered requirement"
        );
        let amount = &input.choices[0].amount;
        integer(amount, 1, i32::MAX.into())?;
        count += input.choices.len();
        ensure!(count <= 65536, "BuildCraft alternative budget exceeded");
        for choice in &input.choices {
            ensure!(
                &choice.amount == amount
                    && matches!(choice.consume, Consumption::Allocated)
                    && choice.returns.is_empty()
                    && matches!(choice.rule, Match::Buildcraft { .. }),
                "BuildCraft group shares one allocated amount and native predicate without returns"
            );
        }
    }
    let output = &recipe.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && output.amount.is_some()
            && output.quantity.is_none()
            && output.change.is_none()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "BuildCraft result must be fixed"
    );
    integer(output.amount.as_deref().unwrap(), 1, i32::MAX.into())?;
    Ok(())
}

fn validate_inscriber(
    recipe: &Recipe,
    mode: InscriberMode,
    top: Option<&str>,
    bottom: Option<&str>,
    name_press: Option<&str>,
    items: &BTreeMap<&str, &super::Item>,
) -> Result<()> {
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none(),
        "inscriber cannot have fixed timing/energy or crafting semantics"
    );
    for id in [top, bottom, name_press].into_iter().flatten() {
        let item = items.get(id).context("missing inscriber process item")?;
        Match::Ae.validate_item(item, items, false, true)?;
    }
    if let (Some(top), Some(name)) = (top, name_press) {
        let same = |id: &str| {
            items[id].registry == items[name].registry && items[id].meta == items[name].meta
        };
        ensure!(
            !same(top) || bottom.is_some_and(|id| !same(id)),
            "inscriber row is preempted by name presses"
        );
    }
    ensure!(
        recipe.inputs.len()
            == 1 + usize::from(top.is_some()) + usize::from(top.is_some() && bottom.is_some()),
        "invalid inscriber input count"
    );
    for (slot, anchor) in [(0, top), (1, bottom.filter(|_| top.is_some())), (2, None)] {
        if slot != 2 && anchor.is_none() {
            continue;
        }
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.kind == Kind::Item && i.slot == slot)
            .context("inscriber input missing")?;
        ensure!(
            !input.choices.is_empty() && (slot == 2 || input.choices.len() == 1),
            "invalid inscriber alternatives"
        );
        for c in &input.choices {
            ensure!(
                c.amount == "1"
                    && c.returns.is_empty()
                    && matches!(c.rule, Match::Ae)
                    && if slot == 2 || mode == InscriberMode::Press {
                        matches!(c.consume, Consumption::Consume)
                    } else {
                        matches!(c.consume, Consumption::Keep)
                    },
                "invalid inscriber consumption or matching"
            );
            ensure!(
                anchor.is_none_or(|id| id == c.id),
                "inscriber plate does not match process"
            );
        }
    }
    ensure!(recipe.outputs.len() == 1, "inscriber requires one output");
    let output = &recipe.outputs[0];
    ensure!(
        output.slot == 0
            && output.kind == Kind::Item
            && output.quantity.is_none()
            && output.change.is_none()
            && output.amount.is_some()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "invalid inscriber result"
    );
    Ok(())
}

fn validate_map(recipe: &Recipe, items: &BTreeMap<&str, &super::Item>) -> Result<()> {
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.magic.is_none()
            && recipe.inputs.len() == 9
            && recipe.outputs.len() == 1,
        "invalid map scaling process"
    );
    let grid = recipe
        .grid
        .as_ref()
        .context("map scaling requires its native grid")?;
    ensure!(
        grid.width == 3
            && grid.height == 3
            && grid.mirror
            && grid.cells == (0..9).map(Some).collect::<Vec<_>>(),
        "invalid map scaling grid"
    );
    for slot in 0..9 {
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.kind == Kind::Item && i.slot == slot)
            .context("map scaling input missing")?;
        ensure!(
            !input.choices.is_empty(),
            "map scaling input has no samples"
        );
        for c in &input.choices {
            let item = items
                .get(c.id.as_str())
                .context("map scaling sample missing")?;
            ensure!(
                item.registry
                    == if slot == 4 {
                        "minecraft:filled_map"
                    } else {
                        "minecraft:paper"
                    }
                    && (slot == 4 || item.meta == 0)
                    && item.meta != 32767
                    && c.amount == "1"
                    && matches!(c.consume, Consumption::Consume)
                    && c.returns.is_empty()
                    && matches!(c.rule, Match::Wildcard { meta, nbt: true } if meta == (slot == 4)),
                "invalid map scaling ingredient"
            );
        }
    }
    let output = &recipe.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && output.quantity.is_none()
            && output.amount.as_deref() == Some("1")
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1"
            && output
                .change
                .as_ref()
                .is_some_and(|c| c.input == 4 && matches!(c.action, super::Edit::MapScaling {})),
        "map scaling requires a pending native result"
    );
    Ok(())
}

fn runic_costs(charge: i32) -> (i64, i32, i32) {
    (
        1 + i64::from(charge.max(0)),
        (32.0 * 2.0_f64.powi(charge)) as i32,
        5 + charge / 2,
    )
}

fn validate_runic(
    recipe: &Recipe,
    charge: i32,
    items: &BTreeMap<&str, &super::Item>,
    aspects: &[super::Aspect],
) -> Result<()> {
    let (pedestals, vis, instability) = runic_costs(charge);
    let magic = recipe
        .magic
        .as_ref()
        .context("runic magic sample missing")?;
    ensure!(
        magic.kind == super::MagicKind::Infusion
            && magic.central == Some(0)
            && magic.instability == Some(instability)
            && magic.payment.is_none()
            && !magic.creative
            && recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none(),
        "invalid runic process sample"
    );
    ensure!(
        magic.research.len() == 1 && magic.research[0].key == "RUNICAUGMENTATION",
        "invalid runic research"
    );
    let actual: BTreeMap<_, _> = magic
        .aspects
        .iter()
        .map(|cost| {
            let aspect = aspects
                .iter()
                .find(|aspect| aspect.id == cost.aspect)
                .context("runic aspect missing")?;
            ensure!(
                aspect.source.owner == "Thaumcraft"
                    && aspect.source.handler == "thaumcraft.api.aspects.Aspect",
                "invalid runic aspect origin"
            );
            Ok((aspect.source.key.as_str(), cost.amount.clone()))
        })
        .collect::<Result<_>>()?;
    let expected = if vis > 0 {
        BTreeMap::from([
            ("tutamen", (vis / 2).to_string()),
            ("praecantatio", (vis / 2).to_string()),
            ("potentia", vis.to_string()),
        ])
    } else {
        BTreeMap::new()
    };
    ensure!(
        actual == expected,
        "runic aspect sample differs from native charge formula"
    );
    ensure!(
        recipe.inputs.len() == 3 && recipe.outputs.len() == 1,
        "runic requires central and two component groups"
    );
    for slot in 0..3 {
        let input = recipe
            .inputs
            .iter()
            .find(|input| input.kind == Kind::Item && input.slot == slot)
            .context("runic input missing")?;
        ensure!(
            !input.choices.is_empty() && (slot != 0 || input.choices.len() == 1),
            "runic samples require separate central branches"
        );
        let mut component_rule = None;
        for c in &input.choices {
            ensure!(
                (slot != 0 || c.returns.is_empty())
                    && c.amount
                        == if slot == 2 {
                            pedestals.to_string()
                        } else {
                            "1".into()
                        },
                "invalid runic component count"
            );
            if slot == 0 {
                ensure!(
                    matches!(c.consume, Consumption::Consume)
                        && matches!(
                            c.rule,
                            Match::Wildcard {
                                meta: true,
                                nbt: true
                            }
                        ),
                    "invalid runic central predicate"
                );
            } else {
                let Match::Infusion { template, .. } = &c.rule else {
                    anyhow::bail!("invalid runic component predicate");
                };
                let rule = serde_json::to_string(&c.rule)?;
                ensure!(
                    component_rule
                        .as_ref()
                        .is_none_or(|expected| expected == &rule),
                    "mixed runic component predicates"
                );
                component_rule = Some(rule);
                let item = items
                    .get(template.as_str())
                    .context("runic component missing")?;
                ensure!(
                    item.registry
                        == if slot == 1 {
                            "minecraft:diamond"
                        } else {
                            "Thaumcraft:ItemResource"
                        }
                        && item.meta == if slot == 1 { 0 } else { 14 }
                        && item.nbt.is_none(),
                    "invalid runic component identity"
                );
                ensure!(
                    if slot == 2 {
                        matches!(c.consume, Consumption::Pedestals)
                    } else {
                        matches!(c.consume, Consumption::Consume)
                    },
                    "invalid runic pedestal consumption"
                );
            }
        }
    }
    let output = &recipe.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && output.quantity.is_none()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1"
            && output.change.as_ref().is_some_and(
                |change| change.input == 0 && matches!(change.action, super::Edit::Runic)
            ),
        "invalid runic transformation"
    );
    Ok(())
}
