use crate::cli::CompileScope;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const COMPILE_KERNEL_TRACE_SCHEMA_VERSION: &str = "elysium-compiler/compile-kernel-trace/v1";
pub const COMPILE_KERNEL_TRACE_REPORT_PATH: &str = "rust/compile-kernel-trace.json";

#[derive(Debug)]
pub struct CompileKernelContext<'a> {
    pub input: &'a Path,
    pub output: &'a Path,
    pub scope: CompileScope,
    pub strict: bool,
    pub debug_json: bool,
    events: Vec<Value>,
}

impl<'a> CompileKernelContext<'a> {
    pub fn new(
        input: &'a Path,
        output: &'a Path,
        scope: CompileScope,
        strict: bool,
        debug_json: bool,
    ) -> Self {
        Self {
            input,
            output,
            scope,
            strict,
            debug_json,
            events: Vec::new(),
        }
    }

    pub fn stage_event(
        &mut self,
        module: &str,
        stage: &str,
        status: &str,
        duration_ms: u128,
        contract: CompileStageContract,
    ) {
        self.events.push(json!({
            "module": module,
            "stage": stage,
            "status": status,
            "durationMs": duration_ms,
            "contract": {
                "inputs": contract.inputs,
                "outputs": contract.outputs,
                "capabilities": contract.capabilities,
            },
        }));
    }

    pub fn events(&self) -> &[Value] {
        &self.events
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CompileStageContract {
    pub inputs: &'static [&'static str],
    pub outputs: &'static [&'static str],
    pub capabilities: &'static [&'static str],
}

impl CompileStageContract {
    pub const fn new(
        inputs: &'static [&'static str],
        outputs: &'static [&'static str],
        capabilities: &'static [&'static str],
    ) -> Self {
        Self {
            inputs,
            outputs,
            capabilities,
        }
    }
}

pub struct CompileStage {
    module: &'static str,
    name: &'static str,
    contract: CompileStageContract,
    run: fn(&mut CompileKernelContext<'_>) -> anyhow::Result<()>,
}

impl CompileStage {
    pub const fn module_stage(
        module: &'static str,
        name: &'static str,
        contract: CompileStageContract,
        run: fn(&mut CompileKernelContext<'_>) -> anyhow::Result<()>,
    ) -> Self {
        Self {
            module,
            name,
            contract,
            run,
        }
    }

    pub fn from_descriptor(module: &'static str, descriptor: CompileStageDescriptor) -> Self {
        Self::module_stage(module, descriptor.name, descriptor.contract, descriptor.run)
    }

    pub fn run(&self, context: &mut CompileKernelContext<'_>) -> anyhow::Result<()> {
        let start = Instant::now();
        let result = (self.run)(context);
        context.stage_event(
            self.module,
            self.name,
            if result.is_ok() { "ok" } else { "failed" },
            start.elapsed().as_millis(),
            self.contract,
        );
        result
    }
}

#[derive(Clone, Copy)]
pub struct CompileStageDescriptor {
    name: &'static str,
    contract: CompileStageContract,
    run: fn(&mut CompileKernelContext<'_>) -> anyhow::Result<()>,
}

impl CompileStageDescriptor {
    pub const fn new(
        name: &'static str,
        contract: CompileStageContract,
        run: fn(&mut CompileKernelContext<'_>) -> anyhow::Result<()>,
    ) -> Self {
        Self {
            name,
            contract,
            run,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn contract(&self) -> CompileStageContract {
        self.contract
    }
}

pub struct CompileStageRegistry {
    stages: Vec<CompileStage>,
}

impl CompileStageRegistry {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn register(&mut self, stage: CompileStage) {
        self.stages.push(stage);
    }

    pub fn register_module(&mut self, module: CompileKernelModule) {
        for descriptor in module.stages {
            self.register(CompileStage::from_descriptor(module.name, *descriptor));
        }
    }

    pub fn into_kernel(self) -> CompileKernel {
        CompileKernel::new(self.stages)
    }
}

#[derive(Clone, Copy)]
pub struct CompileKernelModule {
    name: &'static str,
    stages: &'static [CompileStageDescriptor],
}

impl CompileKernelModule {
    pub const fn new(name: &'static str, stages: &'static [CompileStageDescriptor]) -> Self {
        Self { name, stages }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn stages(&self) -> &'static [CompileStageDescriptor] {
        self.stages
    }
}

pub struct CompileKernel {
    stages: Vec<CompileStage>,
}

impl CompileKernel {
    pub fn new(stages: Vec<CompileStage>) -> Self {
        Self { stages }
    }

    pub fn run(&self, context: &mut CompileKernelContext<'_>) -> anyhow::Result<()> {
        for stage in &self.stages {
            stage.run(context)?;
        }
        Ok(())
    }
}

pub fn trace_report_path(output: &Path) -> PathBuf {
    output.join(COMPILE_KERNEL_TRACE_REPORT_PATH)
}
