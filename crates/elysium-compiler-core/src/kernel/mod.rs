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

    pub fn stage_event(&mut self, stage: &str, status: &str, duration_ms: u128) {
        self.events.push(json!({
            "stage": stage,
            "status": status,
            "durationMs": duration_ms,
        }));
    }

    pub fn events(&self) -> &[Value] {
        &self.events
    }
}

pub struct CompileStage {
    name: &'static str,
    run: fn(&mut CompileKernelContext<'_>) -> anyhow::Result<()>,
}

impl CompileStage {
    pub const fn new(
        name: &'static str,
        run: fn(&mut CompileKernelContext<'_>) -> anyhow::Result<()>,
    ) -> Self {
        Self { name, run }
    }

    pub fn run(&self, context: &mut CompileKernelContext<'_>) -> anyhow::Result<()> {
        let start = Instant::now();
        let result = (self.run)(context);
        context.stage_event(
            self.name,
            if result.is_ok() { "ok" } else { "failed" },
            start.elapsed().as_millis(),
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
