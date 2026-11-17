//! Native NEI and Extra Utilities documentation; no consuming recipe or GUI geometry.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Item, Origin,
};
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Backend {
    Nei,
    ExtraUtilities,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Representation {
    UnwrappedNativeText,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum LinePolicy {
    NeiEscapedNewlines,
    PlainNativeText,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeInformationCounts")]
pub struct Counts {
    pub native_entries: u32,
    pub scanned_items: u32,
    pub pages: u32,
    pub listed: u32,
    pub item_links: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeInformationDefinition")]
pub struct Definition {
    pub source: Origin,
    pub backend: Backend,
    pub ready: bool,
    pub consuming: bool,
    pub representation: Representation,
    pub line_policy: LinePolicy,
    pub counts: Counts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeInformationItemLink")]
pub struct Link {
    pub item: String,
    pub amount: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeInformationText")]
pub struct Text {
    pub key: String,
    pub translated: bool,
    pub text: String,
}
pub(crate) fn required_nullable<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}
/// Presence is mandatory even when the native value is null; keep schema and serde aligned.
pub(crate) struct RequiredNullable<T>(std::marker::PhantomData<T>);
impl<T: JsonSchema> JsonSchema for RequiredNullable<T> {
    fn schema_name() -> String {
        format!("RequiredNullable{}", T::schema_name())
    }
    fn is_referenceable() -> bool {
        false
    }
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <Option<T>>::json_schema(generator)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeInformationPage")]
pub struct Page {
    pub source: Origin,
    pub page: u32,
    pub native_index: u32,
    pub items: Vec<Link>,
    pub listed: bool,
    pub text_sources: Vec<Text>,
    pub lines: Vec<String>,
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "RequiredNullable<String>")]
    pub filter_owner: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "RequiredNullable<bool>")]
    pub precise: Option<bool>,
    pub consuming: bool,
}
pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::NeiInformationDefinition(def) = &rows[0].data else {
        anyhow::bail!("Native information definition missing")
    };
    let c = &def.counts;
    let (handler, policy) = match def.backend {
        Backend::Nei => (
            "codechicken.nei.recipe.InformationHandler",
            LinePolicy::NeiEscapedNewlines,
        ),
        Backend::ExtraUtilities => (
            "com.rwtema.extrautils.nei.InfoHandler",
            LinePolicy::PlainNativeText,
        ),
    };
    ensure!(
        def.ready
            && !def.consuming
            && def.source.handler == handler
            && !def.source.owner.is_empty()
            && !def.source.key.is_empty()
            && def.line_policy == policy,
        "Invalid native information source/policy"
    );
    ensure!(
        [
            c.native_entries,
            c.scanned_items,
            c.pages,
            c.listed,
            c.item_links
        ]
        .iter()
        .all(|n| *n <= 262144)
            && c.listed <= c.pages
            && rows.len() == 1 + c.pages as usize,
        "Native information counts differ"
    );
    ensure!(
        match def.backend {
            Backend::Nei => c.pages == c.native_entries,
            Backend::ExtraUtilities =>
                c.native_entries == c.scanned_items && c.pages <= c.native_entries,
        },
        "Native information enumeration differs"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"neiInformation","records":data})
        )? && serde_json::to_vec(&data)?.len() <= 32 * 1024 * 1024,
        "Native information identity/budget differs"
    );
    let mut links = 0u32;
    let mut listed = 0u32;
    let mut last = None;
    for (index, row) in rows.iter().enumerate().skip(1) {
        let ReferenceData::NeiInformationPage(p) = &row.data else {
            anyhow::bail!("Mixed native information domain")
        };
        ensure!(
            p.source.owner == def.source.owner
                && p.source.handler == def.source.handler
                && p.source.key == def.source.key
                && !p.consuming
                && p.page as usize == index - 1
                && p.native_index < c.native_entries
                && last.is_none_or(|previous| p.native_index > previous)
                && p.listed == !p.items.is_empty(),
            "Native information page/order differs"
        );
        last = Some(p.native_index);
        match def.backend {
            Backend::Nei => ensure!(
                p.native_index == p.page
                    && p.precise.is_none()
                    && p.filter_owner
                        .as_ref()
                        .is_some_and(|name| !name.is_empty() && name.len() <= 1024)
                    && p.text_sources.len() == 1,
                "Native NEI registration differs"
            ),
            Backend::ExtraUtilities => {
                ensure!(
                    p.text_sources.iter().all(|text| text.translated),
                    "Native Extra Utilities text key is not translated"
                );
                ensure!(
                    p.precise.is_some()
                        && p.filter_owner.is_none()
                        && p.items.len() == 1
                        && p.items[0].amount == "1"
                        && !p.text_sources.is_empty()
                        && p.text_sources.len() <= 4096,
                    "Native Extra Utilities documentation differs"
                );
                let first = &p.text_sources[0].key;
                if first.ends_with(".documentation") {
                    ensure!(
                        p.text_sources.len() == 1,
                        "Single documentation key has extra paragraphs"
                    );
                } else {
                    let base = first.strip_suffix(".documentation.0").ok_or_else(|| {
                        anyhow::anyhow!("Sequential documentation must start at zero")
                    })?;
                    for (i, t) in p.text_sources.iter().enumerate() {
                        ensure!(
                            t.key == format!("{base}.documentation.{i}") && t.translated,
                            "Native sequential documentation key differs"
                        );
                    }
                }
            }
        }
        let mut lines = Vec::new();
        for text in &p.text_sources {
            ensure!(
                !text.key.is_empty()
                    && text.key.len() <= 65536
                    && text.text.len() <= 65536
                    && (text.translated || text.text == text.key),
                "Native documentation translation differs"
            );
            let value = if def.line_policy == LinePolicy::NeiEscapedNewlines {
                text.text.replace("\\n", "\n")
            } else {
                text.text.clone()
            };
            lines.extend(value.replace("\r\n", "\n").split('\n').map(str::to_owned));
        }
        ensure!(
            lines == p.lines,
            "Native documentation paragraphs were altered or omitted"
        );
        for link in &p.items {
            let amount = link.amount.parse::<i32>()?;
            ensure!(
                amount > 0
                    && amount.to_string() == link.amount
                    && items.contains_key(link.item.as_str()),
                "Native documentation item/quantity absent"
            );
        }
        links += p.items.len() as u32;
        if p.listed {
            listed += 1;
        }
    }
    ensure!(
        links == c.item_links && listed == c.listed,
        "Native documentation link/visibility counts differ"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::super::references::GeneticReferenceRow;
    #[test]
    fn java_native_information_contract_is_accepted() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../contracts/fixtures/nei-information-contract.json");
        let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let domains: Vec<Vec<GeneticReferenceRow>> =
            serde_json::from_value(raw["domains"].clone()).unwrap();
        assert_eq!(domains.len(), 2);
        let empty: Vec<GeneticReferenceRow> = serde_json::from_value(raw["empty"].clone()).unwrap();
        assert_eq!(empty.len(), 1);
        let schema = schemars::schema_for!(super::Page);
        let fields = &schema.schema.object.as_ref().unwrap().required;
        assert!(fields.contains("filterOwner") && fields.contains("precise"));
    }
    #[test]
    fn native_information_rejects_omission_quantity_paragraph_and_readiness_drift() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixtures.join("nei-information-contract.json")).unwrap(),
        )
        .unwrap();
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        for item in raw["items"].as_array().unwrap() {
            let mut value = domain
                .items
                .iter()
                .find(|old| old.id == item["id"].as_str().unwrap())
                .map(|old| serde_json::to_value(old).unwrap())
                .unwrap_or_else(|| base.clone());
            for key in ["id", "registry", "meta", "nbt"] {
                value[key] = item[key].clone();
            }
            let fact: super::Item = serde_json::from_value(value).unwrap();
            domain.items.retain(|old| old.id != fact.id);
            domain.items.push(fact);
        }
        let groups: Vec<Vec<GeneticReferenceRow>> =
            serde_json::from_value(raw["domains"].clone()).unwrap();
        for rows in &groups {
            let super::ReferenceData::NeiInformationDefinition(def) = &rows[0].data else {
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
        }
        domain.references = groups.iter().flatten().cloned().collect();
        domain.validate(&source).unwrap();
        let items = domain
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        let empty: Vec<GeneticReferenceRow> = serde_json::from_value(raw["empty"].clone()).unwrap();
        super::super::references::validate(&empty, &items).unwrap();
        let reject = |mut rows: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
            let id = super::super::value_id(
                "reference",
                &serde_json::json!({"kind":"neiInformation","records":data}),
            )
            .unwrap();
            for row in &mut rows {
                row.domain = id.clone();
                row.id = row.content_id().unwrap();
            }
            assert!(super::super::references::validate(&rows, &items).is_err());
        };
        let mut missing = groups[0].clone();
        missing.pop();
        reject(missing);
        let mut quantity = groups[0].clone();
        if let super::ReferenceData::NeiInformationPage(p) = &mut quantity[1].data {
            p.items[0].amount = "0".into();
        }
        reject(quantity);
        let mut paragraph = groups[1].clone();
        if let super::ReferenceData::NeiInformationPage(p) = &mut paragraph[2].data {
            p.lines.pop();
        }
        reject(paragraph);
        let mut not_ready = empty;
        if let super::ReferenceData::NeiInformationDefinition(d) = &mut not_ready[0].data {
            d.ready = false;
        }
        reject(not_ready);
        let mut absent = raw["domains"][0][1].clone();
        absent["data"]
            .as_object_mut()
            .unwrap()
            .remove("filterOwner");
        assert!(serde_json::from_value::<GeneticReferenceRow>(absent).is_err());
    }
}
