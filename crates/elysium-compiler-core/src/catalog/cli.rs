use super::{
    compile,
    store::{atomic_json, destination},
    Catalog, Contract,
};
use crate::domain::Domain;
use crate::source::Source;
use anyhow::{ensure, Result};
use clap::{Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "elysium-compiler",
    version,
    about = "Compile verified game facts into an immutable NeoNEI catalog"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Match pinned native resource observations to verified local bytes.
    Resolve {
        #[arg(long)]
        resources: PathBuf,
        #[arg(long)]
        report: PathBuf,
        #[arg(long)]
        sha256: String,
        #[arg(long)]
        environment: String,
        /// Also verify direct PNG pixels against a native Source from this session.
        #[arg(long, requires_all = ["capture", "capture_sha256"])]
        source: Option<PathBuf>,
        #[arg(long, requires_all = ["source", "capture_sha256"])]
        capture: Option<PathBuf>,
        #[arg(long, requires_all = ["source", "capture"])]
        capture_sha256: Option<String>,
        #[arg(long)]
        output: PathBuf,
    },
    /// Import pinned static archives, preserving all resource candidates.
    Resources {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Check all source records, identities, references and pixels.
    Inspect {
        #[arg(long)]
        input: PathBuf,
    },
    /// Verify the currently published catalog and every declared file.
    Check {
        #[arg(long)]
        input: PathBuf,
    },
    /// Compile one source snapshot and atomically publish its catalog.
    Compile {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        report: Option<PathBuf>,
    },
    /// Export the shared JSON Schema used to generate consumer bindings.
    Schema {
        #[arg(long)]
        output: PathBuf,
    },
}

pub fn run() -> Result<()> {
    let value = match Cli::parse().command {
        Command::Resolve {
            resources,
            report,
            sha256,
            environment,
            source,
            capture,
            capture_sha256,
            output,
        } => {
            let capture =
                source
                    .zip(capture)
                    .zip(capture_sha256)
                    .map(
                        |((source, capture), sha256)| crate::resources::CaptureInput {
                            source,
                            capture,
                            sha256,
                        },
                    );
            crate::resources::resolve(
                &resources,
                &report,
                &sha256,
                &environment,
                &output,
                capture.as_ref(),
            )?
        }
        Command::Resources { input, output } => crate::resources::import(&input, &output)?,
        Command::Inspect { input } => {
            let source = Source::open(&input)?;
            Domain::load(&source)?;
            let counts: std::collections::BTreeMap<_, _> = source
                .manifest
                .scope
                .collections
                .iter()
                .map(|kind| {
                    let rows = source
                        .manifest
                        .files
                        .iter()
                        .filter(|file| &file.kind == kind)
                        .map(|file| file.rows)
                        .sum::<u64>();
                    (kind, rows)
                })
                .collect();
            json!({"id": source.manifest.id, "environment": source.manifest.environment,
                "scope": source.manifest.scope, "counts": counts})
        }
        Command::Check { input } => {
            let catalog = Catalog::current(&input)?;
            catalog.verify()?;
            json!({"id": catalog.manifest.id, "counts": catalog.manifest.counts})
        }
        Command::Compile {
            input,
            output,
            report,
        } => {
            if let Some(report) = &report {
                let report = destination(report)?;
                let output = destination(&output)?;
                let source = std::fs::canonicalize(&input)?;
                ensure!(
                    !report.starts_with(&output)
                        && !report.starts_with(&source)
                        && !report.is_dir(),
                    "report must be a file outside the source and catalog directories"
                );
            }
            let publication = compile(&input, &output)?;
            if let Some(report) = report {
                // A receipt failure must never roll back an already committed catalog pointer.
                if let Err(error) = atomic_json(&report, &publication) {
                    eprintln!(
                        "Catalog {} published, but report could not be written: {error:#}",
                        publication.id
                    );
                }
            }
            serde_json::to_value(publication)?
        }
        Command::Schema { output } => {
            atomic_json(&output, &schemars::schema_for!(Contract))?;
            json!({"schema": output})
        }
    };
    println!("{}", serde_json::to_string(&value)?);
    Ok(())
}
