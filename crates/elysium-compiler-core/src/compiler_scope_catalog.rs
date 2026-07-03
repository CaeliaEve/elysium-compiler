//! Descriptor-owned compile scope catalog.
//!
//! Compile scopes affect ABI output, runtime-pack producer selection, and
//! artifact validation. Keep scope names, active producers, and capability
//! projections in one table so pack emission and schema/handshake surfaces
//! cannot drift apart.

use crate::cli::CompileScope;
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const COMPILE_SCOPE_CATALOG_SCHEMA_VERSION: &str = "elysium-compiler/compile-scope-catalog/v1";
pub const COMPILE_SCOPES: &[&str] = &[
    "all",
    "native-ui",
    "search",
    "browser",
    "recipes",
    "ui",
    "textures",
];
pub const SCHEMA_HASH_SCOPES_INPUT: &str =
    "scopes=all,native-ui,search,browser,recipes,ui,textures";

pub const RUNTIME_PACK_PRODUCER_BROWSER: &str = "browser";
pub const RUNTIME_PACK_PRODUCER_RECIPES: &str = "recipes";
pub const RUNTIME_PACK_PRODUCER_UI: &str = "ui";
pub const RUNTIME_PACK_PRODUCER_TEXTURE: &str = "texture";
pub const RUNTIME_PACK_PRODUCER_SEARCH: &str = "search";

const PRODUCERS_ALL: &[&str] = &[
    RUNTIME_PACK_PRODUCER_BROWSER,
    RUNTIME_PACK_PRODUCER_RECIPES,
    RUNTIME_PACK_PRODUCER_UI,
    RUNTIME_PACK_PRODUCER_TEXTURE,
];
const PRODUCERS_NATIVE_UI: &[&str] = &[
    RUNTIME_PACK_PRODUCER_BROWSER,
    RUNTIME_PACK_PRODUCER_RECIPES,
    RUNTIME_PACK_PRODUCER_UI,
];
const PRODUCERS_SEARCH: &[&str] = &[RUNTIME_PACK_PRODUCER_SEARCH];
const PRODUCERS_BROWSER: &[&str] = &[RUNTIME_PACK_PRODUCER_BROWSER];
const PRODUCERS_RECIPES: &[&str] = &[RUNTIME_PACK_PRODUCER_RECIPES];
const PRODUCERS_UI: &[&str] = &[RUNTIME_PACK_PRODUCER_UI];
const PRODUCERS_TEXTURES: &[&str] = &[RUNTIME_PACK_PRODUCER_TEXTURE];

const CAPABILITIES_ALL: &[&str] = &[
    "compiler.browser_pack",
    "compiler.recipe_pack",
    "compiler.native_ui_pack",
    "compiler.texture_pack",
];
const CAPABILITIES_NATIVE_UI: &[&str] = &[
    "compiler.browser_pack",
    "compiler.recipe_pack",
    "compiler.native_ui_pack",
];
const CAPABILITIES_SEARCH: &[&str] = &["compiler.search_pack"];
const CAPABILITIES_BROWSER: &[&str] = &["compiler.browser_pack", "compiler.search_pack"];
const CAPABILITIES_RECIPES: &[&str] = &["compiler.recipe_pack"];
const CAPABILITIES_UI: &[&str] = &["compiler.native_ui_pack"];
const CAPABILITIES_TEXTURES: &[&str] = &["compiler.texture_pack"];

const RUNTIME_CAPABILITIES_ALL: &[&str] = &[
    "atlas.static",
    "atlas.animated",
    "atlas.meta",
    "groups.collapse",
    "groups.semantic-nbt",
    "recipes.lookup",
    "recipes.native-ui-layout",
    "search.zh-cn",
    "strings.zh-cn",
    "native-render.webgl2",
    "recipes.ui-pack",
];
const RUNTIME_CAPABILITIES_NATIVE_UI: &[&str] = &[
    "groups.collapse",
    "groups.semantic-nbt",
    "recipes.lookup",
    "recipes.native-ui-layout",
    "recipes.ui-pack",
    "search.zh-cn",
    "strings.zh-cn",
    "native-render.webgl2",
];
const RUNTIME_CAPABILITIES_SEARCH: &[&str] = &["search.zh-cn", "strings.zh-cn"];
const RUNTIME_CAPABILITIES_BROWSER: &[&str] = &[
    "groups.collapse",
    "groups.semantic-nbt",
    "search.zh-cn",
    "strings.zh-cn",
    "native-render.webgl2",
];
const RUNTIME_CAPABILITIES_RECIPES: &[&str] = &["recipes.lookup", "recipes.native-ui-layout"];
const RUNTIME_CAPABILITIES_UI: &[&str] = &["recipes.ui-pack", "native-render.webgl2"];
const RUNTIME_CAPABILITIES_TEXTURES: &[&str] = &["atlas.static", "atlas.animated", "atlas.meta"];

pub const SCOPE_EVERY: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Search,
    CompileScope::Browser,
    CompileScope::Recipes,
    CompileScope::Ui,
    CompileScope::Textures,
];
pub const SCOPE_ALL_NATIVE_BROWSER: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Browser,
];
pub const SCOPE_ALL_NATIVE_SEARCH_BROWSER: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Search,
    CompileScope::Browser,
];
pub const SCOPE_ALL_NATIVE_RECIPES: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Recipes,
];
pub const SCOPE_ALL_NATIVE_UI: &[CompileScope] =
    &[CompileScope::All, CompileScope::NativeUi, CompileScope::Ui];
pub const SCOPE_ALL_TEXTURES: &[CompileScope] = &[CompileScope::All, CompileScope::Textures];

#[derive(Clone, Copy, Debug)]
pub struct CompileScopeDescriptor {
    scope: CompileScope,
    name: &'static str,
    runtime_pack_producers: &'static [&'static str],
    capabilities: &'static [&'static str],
    runtime_capabilities: &'static [&'static str],
    policy: &'static str,
}

impl CompileScopeDescriptor {
    const fn new(
        scope: CompileScope,
        name: &'static str,
        runtime_pack_producers: &'static [&'static str],
        capabilities: &'static [&'static str],
        runtime_capabilities: &'static [&'static str],
        policy: &'static str,
    ) -> Self {
        Self {
            scope,
            name,
            runtime_pack_producers,
            capabilities,
            runtime_capabilities,
            policy,
        }
    }

    pub fn scope(&self) -> CompileScope {
        self.scope
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn runtime_pack_producers(&self) -> &'static [&'static str] {
        self.runtime_pack_producers
    }

    pub fn capabilities(&self) -> &'static [&'static str] {
        self.capabilities
    }

    pub fn runtime_capabilities(&self) -> &'static [&'static str] {
        self.runtime_capabilities
    }

    pub fn policy(&self) -> &'static str {
        self.policy
    }
}

const COMPILE_SCOPE_DESCRIPTORS: &[CompileScopeDescriptor] = &[
    CompileScopeDescriptor::new(
        CompileScope::All,
        "all",
        PRODUCERS_ALL,
        CAPABILITIES_ALL,
        RUNTIME_CAPABILITIES_ALL,
        "emit every production runtime pack",
    ),
    CompileScopeDescriptor::new(
        CompileScope::NativeUi,
        "native-ui",
        PRODUCERS_NATIVE_UI,
        CAPABILITIES_NATIVE_UI,
        RUNTIME_CAPABILITIES_NATIVE_UI,
        "emit the browser, recipe, and native UI packs required by native UI rendering",
    ),
    CompileScopeDescriptor::new(
        CompileScope::Search,
        "search",
        PRODUCERS_SEARCH,
        CAPABILITIES_SEARCH,
        RUNTIME_CAPABILITIES_SEARCH,
        "emit the standalone search hot path only",
    ),
    CompileScopeDescriptor::new(
        CompileScope::Browser,
        "browser",
        PRODUCERS_BROWSER,
        CAPABILITIES_BROWSER,
        RUNTIME_CAPABILITIES_BROWSER,
        "emit browser/search packs needed by item browsing",
    ),
    CompileScopeDescriptor::new(
        CompileScope::Recipes,
        "recipes",
        PRODUCERS_RECIPES,
        CAPABILITIES_RECIPES,
        RUNTIME_CAPABILITIES_RECIPES,
        "emit recipe graph/runtime packs only",
    ),
    CompileScopeDescriptor::new(
        CompileScope::Ui,
        "ui",
        PRODUCERS_UI,
        CAPABILITIES_UI,
        RUNTIME_CAPABILITIES_UI,
        "emit native UI binary and sidecar packs only",
    ),
    CompileScopeDescriptor::new(
        CompileScope::Textures,
        "textures",
        PRODUCERS_TEXTURES,
        CAPABILITIES_TEXTURES,
        RUNTIME_CAPABILITIES_TEXTURES,
        "emit texture atlas and animation packs only",
    ),
];

pub fn compile_scope_descriptors() -> &'static [CompileScopeDescriptor] {
    validate_compile_scope_descriptors();
    COMPILE_SCOPE_DESCRIPTORS
}

pub fn compile_scope_name(scope: CompileScope) -> &'static str {
    descriptor_for_scope(scope).name()
}

pub fn compile_scope_catalog() -> Value {
    json!({
        "schemaVersion": COMPILE_SCOPE_CATALOG_SCHEMA_VERSION,
        "selectionPolicy": "static-compile-scope-descriptor-table",
        "legacyFallback": "forbidden",
        "scopes": compile_scope_descriptors()
            .iter()
            .map(scope_descriptor_catalog)
            .collect::<Vec<_>>(),
        "schemaHashInput": SCHEMA_HASH_SCOPES_INPUT,
    })
}

pub(crate) fn compile_scope_enables_runtime_pack_producer(
    scope: CompileScope,
    producer_id: &str,
) -> bool {
    descriptor_for_scope(scope)
        .runtime_pack_producers()
        .contains(&producer_id)
}

pub(crate) fn compile_scope_runtime_capabilities(scope: CompileScope) -> &'static [&'static str] {
    descriptor_for_scope(scope).runtime_capabilities()
}

pub(crate) fn compile_scopes_for_runtime_pack_producer(producer_id: &str) -> Vec<&'static str> {
    compile_scope_descriptors()
        .iter()
        .filter(|descriptor| descriptor.runtime_pack_producers().contains(&producer_id))
        .map(CompileScopeDescriptor::name)
        .collect()
}

fn descriptor_for_scope(scope: CompileScope) -> &'static CompileScopeDescriptor {
    compile_scope_descriptors()
        .iter()
        .find(|descriptor| descriptor.scope() == scope)
        .unwrap_or_else(|| panic!("missing compile scope descriptor: {scope:?}"))
}

fn scope_descriptor_catalog(descriptor: &CompileScopeDescriptor) -> Value {
    json!({
        "name": descriptor.name(),
        "runtimePackProducers": descriptor.runtime_pack_producers(),
        "capabilities": descriptor.capabilities(),
        "runtimeCapabilities": descriptor.runtime_capabilities(),
        "policy": descriptor.policy(),
    })
}

fn validate_compile_scope_descriptors() {
    if COMPILE_SCOPE_DESCRIPTORS.is_empty() {
        panic!("compile scope descriptor catalog must not be empty");
    }
    let mut names = BTreeSet::new();
    let mut scopes = BTreeSet::new();
    for descriptor in COMPILE_SCOPE_DESCRIPTORS {
        require_non_empty("compile scope name", descriptor.name);
        if !names.insert(descriptor.name) {
            panic!(
                "duplicate compile scope descriptor name: {}",
                descriptor.name
            );
        }
        if !scopes.insert(descriptor.scope() as u8) {
            panic!(
                "duplicate compile scope descriptor enum: {:?}",
                descriptor.scope()
            );
        }
        require_non_empty_slice(
            "compile scope runtime pack producers",
            descriptor.name,
            descriptor.runtime_pack_producers,
        );
        require_non_empty_slice(
            "compile scope capabilities",
            descriptor.name,
            descriptor.capabilities,
        );
        require_non_empty_slice(
            "compile scope runtime capabilities",
            descriptor.name,
            descriptor.runtime_capabilities,
        );
        require_non_empty("compile scope policy", descriptor.policy);
    }
    require_exact_projection("compile scope", COMPILE_SCOPES, &names);
    if scopes.len() != SCOPE_EVERY.len() {
        panic!("compile scope descriptor catalog must cover every CompileScope variant");
    }
}

fn require_non_empty(label: &str, value: &str) {
    if value.trim().is_empty() {
        panic!("{label} must be non-empty");
    }
}

fn require_non_empty_slice(label: &str, scope_name: &str, values: &[&str]) {
    if values.is_empty() {
        panic!("{label} must not be empty: {scope_name}");
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_non_empty(label, value);
        if !seen.insert(*value) {
            panic!("duplicate {label}: {scope_name}:{value}");
        }
    }
}

fn require_exact_projection(label: &str, expected: &[&str], seen: &BTreeSet<&'static str>) {
    for name in expected {
        if !seen.contains(name) {
            panic!("missing {label} descriptor: {name}");
        }
    }
    for name in seen {
        if !expected.contains(name) {
            panic!("unknown {label} descriptor: {name}");
        }
    }
}
