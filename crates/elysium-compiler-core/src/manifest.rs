use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const RAW_EXPORT_MANIFEST_FILE: &str = "manifest.json";
pub const RAW_MANIFEST_PATH_POLICY: &str =
    "portable-relative-only; no drive letters, UNC paths, file URLs, '.', or '..' segments";
pub const JSONL_EXTENSIONS: &[&str] = &["jsonl", "gz"];
pub const MANIFEST_COLLECTION_CATALOG_SCHEMA_VERSION: &str =
    "elysium-compiler/manifest-collection-catalog/v1";
pub const SCHEMA_HASH_MANIFEST_COLLECTION_INPUT: &str =
    "manifest-collection-catalog=elysium-compiler/manifest-collection-catalog/v1;descriptor-table=v2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManifestCollectionDescriptor {
    pub id: &'static str,
    pub logical_names: &'static [&'static str],
    pub array_fields: &'static [&'static str],
}

impl ManifestCollectionDescriptor {
    pub const fn new(
        id: &'static str,
        logical_names: &'static [&'static str],
        array_fields: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            logical_names,
            array_fields,
        }
    }
}

pub const COLLECTION_BROWSER_ITEMS: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new("browser-items", &["items", "browserCatalog"], &["items"]);
pub const COLLECTION_BROWSER_ITEMS_PREFER_CATALOG: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new(
        "browser-items-catalog-first",
        &["browserCatalog", "items"],
        &["items"],
    );
pub const COLLECTION_SEARCH_ITEMS: ManifestCollectionDescriptor = ManifestCollectionDescriptor::new(
    "search-items",
    &["items", "browserCatalog", "searchAll"],
    &["items"],
);
pub const COLLECTION_SEARCH_ALL_PREFER_INDEX: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new(
        "search-all-index-first",
        &["searchAll", "browserCatalog", "items"],
        &["items"],
    );
pub const COLLECTION_NEI_ORDER: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new("nei-order", &["neiOrder"], &[]);
pub const COLLECTION_BROWSER_GROUPS: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new("browser-groups", &["groups", "browserGroups"], &["groups"]);
pub const COLLECTION_SEMANTIC_ITEMS: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new("semantic-items", &["semanticItems"], &["items"]);
pub const COLLECTION_ITEM_IDENTITY_MAP: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new("item-identity-map", &["itemIdentityMap"], &["items"]);
pub const COLLECTION_TEXTURE_ROWS: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new("texture-rows", &["textures"], &["textures"]);
pub const COLLECTION_TEXTURE_ROWS_WITH_MANIFEST: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new(
        "texture-rows-with-manifest",
        &["textures", "textureManifest"],
        &["textures"],
    );
pub const COLLECTION_HANDLER_LAYOUTS: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new(
        "handler-layouts",
        &["neiHandlerLayouts", "recipeLayouts"],
        &["handler-layouts"],
    );
pub const COLLECTION_NEI_HANDLERS: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new("nei-handlers", &["neiHandlers"], &[]);
pub const COLLECTION_ANIMATIONS: ManifestCollectionDescriptor = ManifestCollectionDescriptor::new(
    "animations",
    &["animations", "animationTable"],
    &["animations"],
);
pub const COLLECTION_NATIVE_SPRITES: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new(
        "native-sprites",
        &["nativeSprites", "nativeRenderIndex"],
        &["sprites"],
    );
pub const COLLECTION_FACADE_RESOLUTIONS: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new(
        "facade-resolutions",
        &["facadeResolutions"],
        &["facadeResolutions", "resolutions"],
    );
pub const COLLECTION_ANIMATION_FRAME_MATERIALIZATIONS: ManifestCollectionDescriptor =
    ManifestCollectionDescriptor::new(
        "animation-frame-materializations",
        &["animationFrameMaterializations"],
        &["animationFrameMaterializations", "materializations"],
    );

pub const MANIFEST_COLLECTION_DESCRIPTORS: &[ManifestCollectionDescriptor] = &[
    COLLECTION_BROWSER_ITEMS,
    COLLECTION_BROWSER_ITEMS_PREFER_CATALOG,
    COLLECTION_SEARCH_ITEMS,
    COLLECTION_SEARCH_ALL_PREFER_INDEX,
    COLLECTION_NEI_ORDER,
    COLLECTION_BROWSER_GROUPS,
    COLLECTION_SEMANTIC_ITEMS,
    COLLECTION_ITEM_IDENTITY_MAP,
    COLLECTION_TEXTURE_ROWS,
    COLLECTION_TEXTURE_ROWS_WITH_MANIFEST,
    COLLECTION_HANDLER_LAYOUTS,
    COLLECTION_NEI_HANDLERS,
    COLLECTION_ANIMATIONS,
    COLLECTION_NATIVE_SPRITES,
    COLLECTION_FACADE_RESOLUTIONS,
    COLLECTION_ANIMATION_FRAME_MATERIALIZATIONS,
];

#[derive(Debug, Deserialize)]
pub struct RawManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub files: BTreeMap<String, String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(rename = "generatedAt")]
    pub generated_at: Option<Value>,
    #[serde(rename = "repositoryName")]
    pub repository_name: Option<String>,
}

pub fn resolve_manifest_path(
    input: &Path,
    manifest: &RawManifest,
    logical_name: &str,
) -> Option<PathBuf> {
    let relative_path = manifest.files.get(logical_name)?;
    let portable = portable_relative_path(relative_path)?;
    Some(input.join(portable))
}

pub fn portable_relative_path(value: &str) -> Option<PathBuf> {
    let normalized = value.trim().replace('\\', "/");
    if normalized.is_empty()
        || normalized.contains("://")
        || normalized.contains(':')
        || normalized.starts_with('/')
        || normalized.starts_with("//")
        || normalized
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(PathBuf::from(normalized))
}

pub fn is_jsonl_path(path: &Path) -> bool {
    path_extension(path)
        .map(|extension| JSONL_EXTENSIONS.contains(&extension))
        .unwrap_or(false)
}

fn path_extension(path: &Path) -> Option<&str> {
    path.extension().and_then(|value| value.to_str())
}

pub fn validate_manifest_collection_descriptors() {
    if MANIFEST_COLLECTION_DESCRIPTORS.is_empty() {
        panic!("manifest collection descriptor catalog must not be empty");
    }
    let mut ids = std::collections::BTreeSet::new();
    for descriptor in MANIFEST_COLLECTION_DESCRIPTORS {
        require_non_empty("manifest collection id", descriptor.id);
        if !ids.insert(descriptor.id) {
            panic!(
                "duplicate manifest collection descriptor id: {}",
                descriptor.id
            );
        }
        if descriptor.logical_names.is_empty() {
            panic!(
                "manifest collection descriptor must declare logical names: {}",
                descriptor.id
            );
        }
        for logical_name in descriptor.logical_names {
            require_manifest_key("manifest collection logical name", logical_name);
        }
        for field in descriptor.array_fields {
            require_non_empty("manifest collection array field", field);
        }
    }
}

pub fn manifest_collection_catalog() -> Value {
    validate_manifest_collection_descriptors();
    json!({
        "schemaVersion": MANIFEST_COLLECTION_CATALOG_SCHEMA_VERSION,
        "schemaHashInput": SCHEMA_HASH_MANIFEST_COLLECTION_INPUT,
        "pathPolicy": RAW_MANIFEST_PATH_POLICY,
        "jsonlExtensions": JSONL_EXTENSIONS,
        "collections": MANIFEST_COLLECTION_DESCRIPTORS
            .iter()
            .map(|descriptor| {
                json!({
                    "id": descriptor.id,
                    "logicalNames": descriptor.logical_names,
                    "arrayFields": descriptor.array_fields,
                })
            })
            .collect::<Vec<_>>()
    })
}

fn require_manifest_key(label: &str, value: &str) {
    require_non_empty(label, value);
    if value.contains('/') || value.contains('\\') || value.contains(':') {
        panic!("{label} must be a manifest logical key");
    }
}

fn require_non_empty(label: &str, value: &str) {
    if value.trim().is_empty() {
        panic!("{label} must be non-empty");
    }
}
