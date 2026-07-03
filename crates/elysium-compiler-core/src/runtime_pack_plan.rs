use crate::cli::CompileScope;
use crate::pack_abi::runtime_pack_artifact_paths_for_producer;
use crate::packs::browser::compile_browser_pack;
use crate::packs::recipe::compile_recipe_pack;
use crate::packs::search::compile_search_pack;
use crate::packs::texture::compile_texture_pack;
use crate::packs::ui::compile_ui_pack;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;

type RuntimePackCompilerFn = fn(&Path, &Path, bool, bool) -> Result<()>;

const SCOPE_ALL_NATIVE_BROWSER: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Browser,
];
const SCOPE_ALL_NATIVE_RECIPES: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Recipes,
];
const SCOPE_ALL_NATIVE_UI: &[CompileScope] =
    &[CompileScope::All, CompileScope::NativeUi, CompileScope::Ui];
const SCOPE_ALL_TEXTURES: &[CompileScope] = &[CompileScope::All, CompileScope::Textures];
const SCOPE_SEARCH_ONLY: &[CompileScope] = &[CompileScope::Search];

#[derive(Clone, Copy)]
pub(crate) struct RuntimePackCompilerDescriptor {
    id: &'static str,
    inputs: &'static [&'static str],
    capabilities: &'static [&'static str],
    scopes: &'static [CompileScope],
    run: RuntimePackCompilerFn,
}

impl RuntimePackCompilerDescriptor {
    const fn new(
        id: &'static str,
        inputs: &'static [&'static str],
        capabilities: &'static [&'static str],
        scopes: &'static [CompileScope],
        run: RuntimePackCompilerFn,
    ) -> Self {
        Self {
            id,
            inputs,
            capabilities,
            scopes,
            run,
        }
    }

    pub(crate) fn id(self) -> &'static str {
        self.id
    }

    pub(crate) fn outputs(self) -> Vec<&'static str> {
        runtime_pack_artifact_paths_for_producer(self.id, false)
    }

    pub(crate) fn debug_outputs(self) -> Vec<&'static str> {
        runtime_pack_artifact_paths_for_producer(self.id, true)
            .into_iter()
            .filter(|path| !self.outputs().contains(path))
            .collect()
    }

    pub(crate) fn capabilities(self) -> &'static [&'static str] {
        self.capabilities
    }

    pub(crate) fn applies_to(self, scope: CompileScope) -> bool {
        self.scopes.contains(&scope)
    }

    fn compile(self, input: &Path, output: &Path, strict: bool, debug_json: bool) -> Result<()> {
        (self.run)(input, output, strict, debug_json)
    }

    fn catalog_entry(self) -> Value {
        json!({
            "id": self.id(),
            "inputs": self.inputs,
            "outputs": self.outputs(),
            "debugOnlyOutputs": self.debug_outputs(),
            "capabilities": self.capabilities(),
            "scopes": self.scopes.iter().map(|scope| scope.as_str()).collect::<Vec<_>>(),
        })
    }
}

const RUNTIME_PACK_COMPILERS: &[RuntimePackCompilerDescriptor] = &[
    RuntimePackCompilerDescriptor::new(
        "browser",
        &["raw-export-manifest", "browser-layout", "browser-atlas"],
        &["compiler.browser_pack", "compiler.search_pack"],
        SCOPE_ALL_NATIVE_BROWSER,
        compile_browser_pack,
    ),
    RuntimePackCompilerDescriptor::new(
        "recipes",
        &[
            "raw-export-manifest",
            "recipe-shards",
            "native-ui-capture-facts",
        ],
        &["compiler.recipe_pack"],
        SCOPE_ALL_NATIVE_RECIPES,
        compile_recipe_pack,
    ),
    RuntimePackCompilerDescriptor::new(
        "ui",
        &["native-ui-capture-facts", "ui-template-catalog"],
        &["compiler.native_ui_pack"],
        SCOPE_ALL_NATIVE_UI,
        compile_ui_pack,
    ),
    RuntimePackCompilerDescriptor::new(
        "texture",
        &["texture-atlas", "texture-animation-facts"],
        &["compiler.texture_pack"],
        SCOPE_ALL_TEXTURES,
        compile_texture_pack,
    ),
    RuntimePackCompilerDescriptor::new(
        "search",
        &["raw-export-manifest", "browser-layout"],
        &["compiler.search_pack"],
        SCOPE_SEARCH_ONLY,
        compile_search_pack,
    ),
];

pub(crate) fn runtime_pack_compilers(
    scope: CompileScope,
) -> impl Iterator<Item = &'static RuntimePackCompilerDescriptor> {
    RUNTIME_PACK_COMPILERS
        .iter()
        .filter(move |compiler| compiler.applies_to(scope))
}

pub(crate) fn compile_runtime_packs(
    input: &Path,
    output: &Path,
    scope: CompileScope,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    for compiler in runtime_pack_compilers(scope) {
        compiler.compile(input, output, strict, debug_json)?;
    }
    Ok(())
}

pub(crate) fn runtime_pack_compiler_catalog() -> Vec<Value> {
    RUNTIME_PACK_COMPILERS
        .iter()
        .map(|compiler| compiler.catalog_entry())
        .collect()
}
