use super::binding::{hash, read};
use crate::domain::{Asset, AssetKind, Domain};
use crate::provenance::Provenance;
use crate::source::{is_digest, Source};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Pinned native publication receipt. Paths locate inputs but never enter identity.
pub struct CaptureInput {
    pub source: PathBuf,
    pub capture: PathBuf,
    pub sha256: String,
}

pub(super) struct SourceProof {
    source: Source,
    assets: BTreeMap<String, Vec<Asset>>,
    receipt: Value,
    matched: Vec<Value>,
}

impl SourceProof {
    pub(super) fn open(input: &CaptureInput, report: &Value, output: &Path) -> Result<Self> {
        ensure!(
            is_digest(&input.sha256),
            "expected capture digest is required"
        );
        let bytes = read(&input.capture, 256 * 1024)?;
        ensure!(
            hash(&bytes) == input.sha256,
            "capture receipt digest mismatch"
        );
        let job: Value = serde_json::from_slice(&bytes)?;
        let source = Source::open(&input.source)?;
        ensure!(
            job["request"]["scope"].is_null() || source.manifest.scope.mode == "selection",
            "scoped capture cannot claim complete Source coverage"
        );
        ensure!(
            !output.starts_with(source.root()),
            "bindings output must be outside the Source"
        );
        ensure!(
            job["state"] == "succeeded"
                && job["error"].is_null()
                && job["request"]["check"].is_null()
                && job["result"]["id"] == source.manifest.id
                && source.manifest.producer.name == "nesql",
            "capture did not publish this Source"
        );
        let environment: Value = serde_json::to_value(source.environment()?)?;
        let capture = Provenance::verify(&job["provenance"], &environment, &job["request"])?;
        let evidence = Provenance::verify(
            &report["provenance"],
            &report["environment"],
            &report["request"],
        )?;
        ensure!(
            capture.runtime == evidence.runtime
                && capture.session == evidence.session
                && job["request"]["world"].is_string()
                && job["request"]["world"] == report["request"]["world"],
            "resource and Source capture contexts differ"
        );
        let domain = Domain::load(&source)?;
        let mut assets: BTreeMap<String, Vec<Asset>> = BTreeMap::new();
        for asset in domain.assets {
            // Animated atlases, tints and GL captures never qualify merely by
            // sharing a name or by having the same dimensions as a PNG.
            if matches!(asset.source.kind, AssetKind::Resource)
                && asset.frames.is_empty()
                && !asset.interpolate
                && asset.source.location.ends_with(".png")
            {
                assets
                    .entry(asset.source.location.clone())
                    .or_default()
                    .push(asset);
            }
        }
        let job_id = job["id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .context("missing capture job id")?;
        let receipt = json!({"id":source.manifest.id,"scope":source.manifest.scope,"environment":source.manifest.environment,
            "capture":job_id,"sha256":input.sha256,"provenance":job["provenance"]});
        Ok(Self {
            source,
            assets,
            receipt,
            matched: Vec::new(),
        })
    }

    pub(super) fn compare(&mut self, resource: &str, bytes: &[u8]) -> Result<()> {
        let Some(assets) = self.assets.get(resource) else {
            return Ok(());
        };
        for asset in assets {
            let file = self
                .source
                .files("asset")
                .find(|file| file.path == asset.path)
                .context("missing native asset")?;
            ensure!(file.encoding == "png", "direct native resource must be PNG");
            let native = pixels(&self.source.read_file(file)?, asset.width, asset.height)?;
            let local = pixels(bytes, asset.width, asset.height)?;
            ensure!(
                native == local,
                "resource pixels differ from native Source: {resource}"
            );
            self.matched.push(json!({"asset":asset.id,"resource":resource,"width":asset.width,"height":asset.height,"pixels":hash(&local)}));
        }
        Ok(())
    }

    pub(super) fn finish(mut self) -> Result<Value> {
        ensure!(
            !self.matched.is_empty(),
            "no direct Source resource pixels were verified"
        );
        self.matched
            .sort_by(|a, b| a["asset"].as_str().cmp(&b["asset"].as_str()));
        self.receipt["assets"] = json!(self.matched);
        Ok(self.receipt)
    }
}

fn pixels(bytes: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut reader =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(width);
    limits.max_image_height = Some(height);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().context("decode direct resource PNG")?;
    ensure!(
        image.width() == width && image.height() == height,
        "resource dimensions differ from Source"
    );
    Ok(image.into_rgba8().into_raw())
}
