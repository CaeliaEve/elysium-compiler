use crate::cli::CompileScope;
use crate::compiler_scope_catalog::{
    compile_scope_enables_runtime_pack_producer, compile_scopes_for_runtime_pack_producer,
};
use crate::pack_abi::runtime_pack_artifact_paths_for_producer;
use crate::packs::browser::compile_browser_pack;
use crate::packs::recipe::compile_recipe_pack;
use crate::packs::search::compile_search_pack;
use crate::packs::texture::compile_texture_pack;
use crate::packs::ui::compile_ui_pack;
use crate::session::RawExportSession;
use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;
use std::sync::Mutex;

type RuntimePackCompilerFn = fn(&RawExportSession, &Path, bool, bool) -> Result<()>;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum RuntimePackProducerId {
    Browser,
    Recipes,
    Ui,
    Texture,
    Search,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimePackConcurrency {
    Parallel,
    ExclusiveIo,
}

impl RuntimePackProducerId {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Recipes => "recipes",
            Self::Ui => "ui",
            Self::Texture => "texture",
            Self::Search => "search",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimePackInput {
    RawExportManifest,
    BrowserLayout,
    BrowserAtlas,
    RecipeShards,
    NativeUiCaptureFacts,
    UiTemplateCatalog,
    CompiledRecipeUiPayloadIndex,
    TextureAtlas,
    TextureAnimationFacts,
}

impl RuntimePackInput {
    const fn as_str(self) -> &'static str {
        match self {
            Self::RawExportManifest => "raw-export-manifest",
            Self::BrowserLayout => "browser-layout",
            Self::BrowserAtlas => "browser-atlas",
            Self::RecipeShards => "recipe-shards",
            Self::NativeUiCaptureFacts => "native-ui-capture-facts",
            Self::UiTemplateCatalog => "ui-template-catalog",
            Self::CompiledRecipeUiPayloadIndex => "compiled-recipe-ui-payload-index",
            Self::TextureAtlas => "texture-atlas",
            Self::TextureAnimationFacts => "texture-animation-facts",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimePackOwnedOutput {
    File(&'static str),
    Directory(&'static str),
}

impl RuntimePackOwnedOutput {
    const fn path(self) -> &'static str {
        match self {
            Self::File(path) | Self::Directory(path) => path,
        }
    }

    const fn is_directory(self) -> bool {
        matches!(self, Self::Directory(_))
    }
}

pub(crate) struct CompilerThreadPool {
    pool: rayon::ThreadPool,
    thread_count: usize,
}

impl fmt::Debug for CompilerThreadPool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompilerThreadPool")
            .field("thread_count", &self.thread_count)
            .finish_non_exhaustive()
    }
}

impl CompilerThreadPool {
    pub(crate) fn new(threads: Option<usize>) -> Result<Self> {
        let thread_count = match threads {
            Some(0) => bail!("compiler thread count must be at least 1"),
            Some(thread_count) => thread_count,
            None => std::thread::available_parallelism()
                .context("resolve available compiler parallelism")?
                .get(),
        };
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .thread_name(|index| format!("elysium-pack-{index}"))
            .build()
            .with_context(|| {
                format!("build local compiler thread pool with {thread_count} threads")
            })?;
        Ok(Self { pool, thread_count })
    }

    pub(crate) fn thread_count(&self) -> usize {
        self.thread_count
    }

    pub(crate) fn install<OP, R>(&self, operation: OP) -> R
    where
        OP: FnOnce() -> R + Send,
        R: Send,
    {
        self.pool.install(operation)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RuntimePackCompilerDescriptor {
    id: RuntimePackProducerId,
    inputs: &'static [RuntimePackInput],
    dependencies: &'static [RuntimePackProducerId],
    owned_outputs: &'static [RuntimePackOwnedOutput],
    capabilities: &'static [&'static str],
    concurrency: RuntimePackConcurrency,
    run: RuntimePackCompilerFn,
}

impl RuntimePackCompilerDescriptor {
    const fn new(
        id: RuntimePackProducerId,
        inputs: &'static [RuntimePackInput],
        dependencies: &'static [RuntimePackProducerId],
        owned_outputs: &'static [RuntimePackOwnedOutput],
        capabilities: &'static [&'static str],
        concurrency: RuntimePackConcurrency,
        run: RuntimePackCompilerFn,
    ) -> Self {
        Self {
            id,
            inputs,
            dependencies,
            owned_outputs,
            capabilities,
            concurrency,
            run,
        }
    }

    pub(crate) fn id(self) -> &'static str {
        self.id.as_str()
    }

    pub(crate) fn inputs(self) -> Vec<&'static str> {
        self.inputs.iter().map(|input| input.as_str()).collect()
    }

    pub(crate) fn outputs(self) -> Vec<&'static str> {
        runtime_pack_artifact_paths_for_producer(self.id(), false)
    }

    pub(crate) fn debug_outputs(self) -> Vec<&'static str> {
        runtime_pack_artifact_paths_for_producer(self.id(), true)
            .into_iter()
            .filter(|path| !self.outputs().contains(path))
            .collect()
    }

    pub(crate) fn dependencies(self) -> Vec<&'static str> {
        self.dependencies
            .iter()
            .map(|dependency| dependency.as_str())
            .collect()
    }

    pub(crate) fn owned_outputs(self) -> Vec<&'static str> {
        self.owned_outputs
            .iter()
            .map(|output| output.path())
            .collect()
    }

    pub(crate) fn capabilities(self) -> &'static [&'static str] {
        self.capabilities
    }

    pub(crate) fn applies_to(self, scope: CompileScope) -> bool {
        compile_scope_enables_runtime_pack_producer(scope, self.id())
    }

    fn compile(
        self,
        session: &RawExportSession,
        output: &Path,
        strict: bool,
        debug_json: bool,
    ) -> Result<()> {
        (self.run)(session, output, strict, debug_json)
            .with_context(|| format!("compile runtime pack producer {}", self.id()))
    }

    fn catalog_entry(self) -> Value {
        json!({
            "id": self.id(),
            "inputs": self.inputs(),
            "outputs": self.outputs(),
            "debugOnlyOutputs": self.debug_outputs(),
            "dependencies": self.dependencies(),
            "ownership": {
                "mode": "exclusive",
                "paths": self.owned_outputs(),
            },
            "capabilities": self.capabilities(),
            "concurrency": match self.concurrency {
                RuntimePackConcurrency::Parallel => "parallel",
                RuntimePackConcurrency::ExclusiveIo => "exclusive-io",
            },
            "scopes": compile_scopes_for_runtime_pack_producer(self.id()),
        })
    }
}

const NO_DEPENDENCIES: &[RuntimePackProducerId] = &[];
const UI_DEPENDENCIES: &[RuntimePackProducerId] = &[RuntimePackProducerId::Recipes];
// The texture producer has the largest transient allocation profile. In an all-scope
// transaction, run it only after the recipe/UI chain has released its source caches.
// This dependency is ignored by the standalone textures scope because UI is inactive.
const TEXTURE_DEPENDENCIES: &[RuntimePackProducerId] = &[RuntimePackProducerId::Ui];

const BROWSER_INPUTS: &[RuntimePackInput] = &[
    RuntimePackInput::RawExportManifest,
    RuntimePackInput::BrowserLayout,
    RuntimePackInput::BrowserAtlas,
];
const RECIPE_INPUTS: &[RuntimePackInput] = &[
    RuntimePackInput::RawExportManifest,
    RuntimePackInput::RecipeShards,
    RuntimePackInput::NativeUiCaptureFacts,
];
const UI_INPUTS: &[RuntimePackInput] = &[
    RuntimePackInput::RawExportManifest,
    RuntimePackInput::UiTemplateCatalog,
    RuntimePackInput::CompiledRecipeUiPayloadIndex,
];
const TEXTURE_INPUTS: &[RuntimePackInput] = &[
    RuntimePackInput::RawExportManifest,
    RuntimePackInput::TextureAtlas,
    RuntimePackInput::TextureAnimationFacts,
];
const SEARCH_INPUTS: &[RuntimePackInput] = &[
    RuntimePackInput::RawExportManifest,
    RuntimePackInput::BrowserLayout,
];

const BROWSER_OWNED_OUTPUTS: &[RuntimePackOwnedOutput] = &[
    RuntimePackOwnedOutput::File("rust/browser.bin"),
    RuntimePackOwnedOutput::File("rust/groups.bin"),
    RuntimePackOwnedOutput::File("rust/search.bin"),
    RuntimePackOwnedOutput::File("rust/strings.zh_cn.bin"),
    RuntimePackOwnedOutput::File("rust/browser-pack.json"),
    RuntimePackOwnedOutput::File("rust/search-pack.json"),
];
const RECIPE_OWNED_OUTPUTS: &[RuntimePackOwnedOutput] = &[
    RuntimePackOwnedOutput::Directory("recipes/"),
    RuntimePackOwnedOutput::File("rust/recipes.bin"),
    RuntimePackOwnedOutput::File("rust/recipe-pack.json"),
    RuntimePackOwnedOutput::File("rust/recipe-handler-metadata-report.json"),
    RuntimePackOwnedOutput::File("rust/recipe-fragmentation-report.json"),
];
const UI_OWNED_OUTPUTS: &[RuntimePackOwnedOutput] =
    &[RuntimePackOwnedOutput::Directory("rust/ui-pack/")];
const TEXTURE_OWNED_OUTPUTS: &[RuntimePackOwnedOutput] = &[
    RuntimePackOwnedOutput::Directory("textures/"),
    RuntimePackOwnedOutput::Directory("render/"),
    RuntimePackOwnedOutput::File("rust/textures.bin"),
    RuntimePackOwnedOutput::File("rust/atlas.meta.bin"),
    RuntimePackOwnedOutput::File("rust/animations.bin"),
    RuntimePackOwnedOutput::File("rust/texture-pack.json"),
    RuntimePackOwnedOutput::File("rust/missing-texture-report.json"),
    RuntimePackOwnedOutput::File("rust/suspicious-texture-report.json"),
];
const SEARCH_OWNED_OUTPUTS: &[RuntimePackOwnedOutput] = &[
    RuntimePackOwnedOutput::File("rust/search.bin"),
    RuntimePackOwnedOutput::File("rust/strings.zh_cn.bin"),
    RuntimePackOwnedOutput::File("rust/search-pack.json"),
];

const RUNTIME_PACK_COMPILERS: &[RuntimePackCompilerDescriptor] = &[
    RuntimePackCompilerDescriptor::new(
        RuntimePackProducerId::Browser,
        BROWSER_INPUTS,
        NO_DEPENDENCIES,
        BROWSER_OWNED_OUTPUTS,
        &["compiler.browser_pack", "compiler.search_pack"],
        RuntimePackConcurrency::Parallel,
        compile_browser_pack,
    ),
    RuntimePackCompilerDescriptor::new(
        RuntimePackProducerId::Recipes,
        RECIPE_INPUTS,
        NO_DEPENDENCIES,
        RECIPE_OWNED_OUTPUTS,
        &["compiler.recipe_pack"],
        RuntimePackConcurrency::ExclusiveIo,
        compile_recipe_pack,
    ),
    RuntimePackCompilerDescriptor::new(
        RuntimePackProducerId::Ui,
        UI_INPUTS,
        UI_DEPENDENCIES,
        UI_OWNED_OUTPUTS,
        &["compiler.native_ui_pack"],
        RuntimePackConcurrency::Parallel,
        compile_ui_pack,
    ),
    RuntimePackCompilerDescriptor::new(
        RuntimePackProducerId::Texture,
        TEXTURE_INPUTS,
        TEXTURE_DEPENDENCIES,
        TEXTURE_OWNED_OUTPUTS,
        &["compiler.texture_pack"],
        RuntimePackConcurrency::ExclusiveIo,
        compile_texture_pack,
    ),
    RuntimePackCompilerDescriptor::new(
        RuntimePackProducerId::Search,
        SEARCH_INPUTS,
        NO_DEPENDENCIES,
        SEARCH_OWNED_OUTPUTS,
        &["compiler.search_pack"],
        RuntimePackConcurrency::Parallel,
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

fn owned_outputs_conflict(left: RuntimePackOwnedOutput, right: RuntimePackOwnedOutput) -> bool {
    let left_path = left.path().trim_end_matches('/');
    let right_path = right.path().trim_end_matches('/');
    if left_path == right_path {
        return true;
    }
    (left.is_directory() && right_path.starts_with(&format!("{left_path}/")))
        || (right.is_directory() && left_path.starts_with(&format!("{right_path}/")))
}

pub(crate) fn runtime_pack_ownership_conflicts(scope: CompileScope) -> Vec<String> {
    let compilers = runtime_pack_compilers(scope).collect::<Vec<_>>();
    let mut conflicts = Vec::new();
    for (left_index, left) in compilers.iter().enumerate() {
        for right in compilers.iter().skip(left_index + 1) {
            for left_output in left.owned_outputs {
                for right_output in right.owned_outputs {
                    if owned_outputs_conflict(*left_output, *right_output) {
                        conflicts.push(format!(
                            "{}:{} conflicts with {}:{}",
                            left.id(),
                            left_output.path(),
                            right.id(),
                            right_output.path()
                        ));
                    }
                }
            }
        }
    }
    conflicts
}

fn execution_descriptor_waves(
    scope: CompileScope,
) -> Result<Vec<Vec<&'static RuntimePackCompilerDescriptor>>> {
    let active = runtime_pack_compilers(scope).collect::<Vec<_>>();
    let conflicts = runtime_pack_ownership_conflicts(scope);
    if !conflicts.is_empty() {
        bail!(
            "runtime pack ownership conflicts for scope {}: {}",
            scope.as_str(),
            conflicts.join("; ")
        );
    }

    let active_ids = active
        .iter()
        .map(|compiler| compiler.id)
        .collect::<BTreeSet<_>>();
    let mut completed = BTreeSet::new();
    let mut remaining = active;
    let mut waves = Vec::new();
    while !remaining.is_empty() {
        let mut wave = Vec::new();
        let mut deferred = Vec::new();
        for compiler in remaining {
            let ready = compiler
                .dependencies
                .iter()
                .filter(|dependency| active_ids.contains(dependency))
                .all(|dependency| completed.contains(dependency));
            if ready {
                wave.push(compiler);
            } else {
                deferred.push(compiler);
            }
        }
        if wave.is_empty() {
            return Err(anyhow!(
                "runtime pack dependency cycle for scope {}: {}",
                scope.as_str(),
                deferred
                    .iter()
                    .map(|compiler| compiler.id())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        completed.extend(wave.iter().map(|compiler| compiler.id));
        waves.push(wave);
        remaining = deferred;
    }
    Ok(waves)
}

fn execution_descriptor_batches(
    scope: CompileScope,
) -> Result<Vec<Vec<&'static RuntimePackCompilerDescriptor>>> {
    let mut batches = Vec::new();
    for wave in execution_descriptor_waves(scope)? {
        let mut parallel = Vec::new();
        for compiler in wave {
            match compiler.concurrency {
                RuntimePackConcurrency::Parallel => parallel.push(compiler),
                RuntimePackConcurrency::ExclusiveIo => {
                    if !parallel.is_empty() {
                        batches.push(std::mem::take(&mut parallel));
                    }
                    batches.push(vec![compiler]);
                }
            }
        }
        if !parallel.is_empty() {
            batches.push(parallel);
        }
    }
    Ok(batches)
}

#[cfg(test)]
pub(crate) fn runtime_pack_execution_waves(scope: CompileScope) -> Result<Vec<Vec<&'static str>>> {
    execution_descriptor_waves(scope).map(|waves| {
        waves
            .into_iter()
            .map(|wave| wave.into_iter().map(|compiler| compiler.id()).collect())
            .collect()
    })
}

#[cfg(test)]
pub(crate) fn runtime_pack_execution_batches(
    scope: CompileScope,
) -> Result<Vec<Vec<&'static str>>> {
    execution_descriptor_batches(scope).map(|batches| {
        batches
            .into_iter()
            .map(|batch| batch.into_iter().map(|compiler| compiler.id()).collect())
            .collect()
    })
}

pub(crate) fn compile_runtime_packs(
    executor: &CompilerThreadPool,
    session: &RawExportSession,
    output: &Path,
    scope: CompileScope,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    for batch in execution_descriptor_batches(scope)? {
        let results = Mutex::new(Vec::<(usize, Result<()>)>::with_capacity(batch.len()));
        executor.install(|| {
            rayon::scope(|rayon_scope| {
                for (index, compiler) in batch.iter().copied().enumerate() {
                    let results = &results;
                    rayon_scope.spawn(move |_| {
                        let result = compiler.compile(session, output, strict, debug_json);
                        results
                            .lock()
                            .expect("runtime pack result lock poisoned")
                            .push((index, result));
                    });
                }
            });
        });
        let mut results = results
            .into_inner()
            .map_err(|_| anyhow!("runtime pack result lock poisoned"))?;
        results.sort_by_key(|(index, _)| *index);
        for (_, result) in results {
            result?;
        }
        session.release_source_caches();
    }
    Ok(())
}

pub(crate) fn runtime_pack_compiler_catalog() -> Vec<Value> {
    RUNTIME_PACK_COMPILERS
        .iter()
        .map(|compiler| compiler.catalog_entry())
        .collect()
}
