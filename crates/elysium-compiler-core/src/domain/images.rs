use super::{content_id, Asset};
use crate::source::{Source, SourceFile};
use anyhow::{bail, ensure, Context, Result};
use image::DynamicImage;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate_metadata(source: &Source, assets: &[Asset]) -> Result<()> {
    let files: BTreeMap<_, _> = source
        .files("asset")
        .map(|file| (file.path.as_str(), file))
        .collect();
    let mut used = BTreeSet::new();
    for asset in assets {
        ensure!(
            asset.id == content_id("asset", asset)?,
            "asset identity mismatch: {}",
            asset.id
        );
        ensure!(
            files.contains_key(asset.path.as_str()),
            "undeclared asset file: {}",
            asset.path
        );
        used.insert(asset.path.as_str());
        ensure!(
            asset.width > 0 && asset.height > 0 && asset.width <= 16384 && asset.height <= 16384,
            "invalid dimensions {}x{}",
            asset.width,
            asset.height
        );
        ensure!(
            u64::from(asset.width) * u64::from(asset.height) <= 16 * 1024 * 1024,
            "asset exceeds 16 million pixels: {}",
            asset.id
        );
        ensure!(
            asset.frames.len() <= 4096 && !asset.source.location.is_empty(),
            "invalid asset source or animation"
        );
        ensure!(
            !asset.interpolate || !asset.frames.is_empty(),
            "still asset cannot interpolate"
        );
        let mut frame_size = None;
        for frame in &asset.frames {
            ensure!(
                frame.width > 0 && frame.height > 0 && frame.width <= 4096 && frame.height <= 4096,
                "invalid dimensions {}x{}",
                frame.width,
                frame.height
            );
            ensure!(
                u64::from(frame.x) + u64::from(frame.width) <= u64::from(asset.width)
                    && u64::from(frame.y) + u64::from(frame.height) <= u64::from(asset.height),
                "animation crop exceeds asset bounds: {}",
                asset.id
            );
            ensure!(
                frame.ticks > 0 && frame.ticks <= 72000,
                "invalid animation duration"
            );
            let size = (frame.width, frame.height);
            if let Some(previous) = frame_size {
                ensure!(
                    size == previous,
                    "animation frames must have equal dimensions"
                );
            }
            frame_size = Some(size);
        }
    }
    ensure!(
        used.len() == files.len(),
        "source declares orphaned image files"
    );
    Ok(())
}

/// Verify the exact bytes and pixels in one read/decode, without retaining a decoded cache.
pub(crate) fn decode(source: &Source, file: &SourceFile, asset: &Asset) -> Result<DynamicImage> {
    let bytes = source.read_file(file)?;
    let format = match file.encoding.as_str() {
        "png" => image::ImageFormat::Png,
        "webp" => image::ImageFormat::WebP,
        _ => bail!("unsupported image encoding"),
    };
    let mut reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(asset.width);
    limits.max_image_height = Some(asset.height);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader
        .decode()
        .with_context(|| format!("decode asset {}", asset.id))?;
    ensure!(
        decoded.width() == asset.width && decoded.height() == asset.height,
        "asset dimensions differ from pixels: {}",
        asset.id
    );
    Ok(decoded)
}

pub(crate) fn verify(source: &Source, assets: &[Asset]) -> Result<()> {
    let files: BTreeMap<_, _> = source
        .files("asset")
        .map(|file| (file.path.as_str(), file))
        .collect();
    for asset in assets {
        let file = files
            .get(asset.path.as_str())
            .context("undeclared asset file")?;
        decode(source, file, asset)?;
    }
    Ok(())
}
