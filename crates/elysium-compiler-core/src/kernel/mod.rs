use crate::cli::CompileScope;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Instant;

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
        stage: &str,
        status: &str,
        duration_ms: u128,
        contract: CompileStageContract,
    ) {
        self.events.push(json!({
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
    name: &'static str,
    contract: CompileStageContract,
    run: fn(&mut CompileKernelContext<'_>) -> anyhow::Result<()>,
}

impl CompileStage {
    pub const fn with_contract(
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

    pub fn run(&self, context: &mut CompileKernelContext<'_>) -> anyhow::Result<()> {
        let start = Instant::now();
        let result = (self.run)(context);
        context.stage_event(
            self.name,
            if result.is_ok() { "ok" } else { "failed" },
            start.elapsed().as_millis(),
            self.contract,
        );
        result
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
    output.join("rust").join("compile-kernel-trace.json")
}
