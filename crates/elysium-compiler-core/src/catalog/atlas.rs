use super::store::Writer;
use super::{Sprite, Texture};
use crate::source::Source;
use anyhow::{ensure, Result};
use image::{GenericImageView, ImageEncoder, RgbaImage};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const PAGE: u32 = 4096;

#[derive(Clone)]
struct Region {
    group: usize,
    page: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    path: Option<String>,
}

struct Atlas {
    edge: u32,
    group: usize,
    image: RgbaImage,
    x: u32,
    y: u32,
    row: u32,
    width: u32,
    pages: Vec<String>,
    regions: BTreeMap<[u8; 32], Region>,
}

pub(super) fn pack(source: &Source, domain: &crate::domain::Domain, writer: &mut Writer) -> Result<Vec<Texture>> {
    let assets = &domain.assets;
    let visuals = &domain.visuals;
    let inventory_textures: BTreeSet<_> = visuals.iter().flat_map(|row| row.data.asset_references()).collect();
    // Native NEI order supplies locality. First paint never decodes the later
    // frames of an animation, and 16/64-pixel icons never pin a 4096-pixel page.
    let mut priority = BTreeMap::<&str, u32>::new();
    for item in &domain.items {
        if let (Some(icon), Some(order)) = (&item.icon, item.order) {
            priority.entry(icon).and_modify(|value| *value = (*value).min(order)).or_insert(order);
        }
    }
    let mut ordered: Vec<_> = assets.iter().collect();
    ordered.sort_by_key(|asset| (priority.get(asset.id.as_str()).copied().unwrap_or(u32::MAX), asset.id.as_str()));
    let mut atlases = [Atlas::new(1024, 0), Atlas::new(1024, 1), Atlas::new(PAGE, 2)];
    let files: BTreeMap<_, _> = source
        .files("asset")
        .map(|file| (file.path.as_str(), file))
        .collect();
    let mut pending = Vec::new();
    for asset in ordered {
        let pixels = crate::domain::images::decode(source, files[asset.path.as_str()], asset)?.into_rgba8();
        let mut frames = Vec::new();
        if inventory_textures.contains(asset.id.as_str()) {
            // These are native model samplers, not browse previews. A tiny key
            // texture must not pin/decode an unrelated 4096x4096 icon page.
            let path = writer.image(&encode(&pixels)?)?;
            frames.push((Region { group: 2, page: 0, x: 0, y: 0, width: pixels.width(), height: pixels.height(), path: Some(path) }, 1));
        } else if asset.frames.is_empty() {
            let group = if pixels.width() <= 64 && pixels.height() <= 64 { 0 } else { 2 };
            frames.push((atlases[group].place(&pixels, writer)?, 1));
        } else {
            for (index, frame) in asset.frames.iter().enumerate() {
                let crop = pixels
                    .view(frame.x, frame.y, frame.width, frame.height)
                    .to_image();
                let group = if frame.width <= 64 && frame.height <= 64 { usize::from(index > 0) } else { 2 };
                frames.push((atlases[group].place(&crop, writer)?, frame.ticks));
            }
        }
        pending.push((asset, frames));
    }
    for atlas in &mut atlases { atlas.flush(writer)?; }
    let mut textures: Vec<_> = pending
        .into_iter()
        .map(|(asset, frames)| Texture {
            id: asset.id.clone(),
            width: frames[0].0.width,
            height: frames[0].0.height,
            interpolate: asset.interpolate,
            visual: asset.visual.clone(),
            frames: frames
                .into_iter()
                .map(|(region, ticks)| Sprite {
                    path: region.path.unwrap_or_else(|| atlases[region.group].pages[region.page].clone()),
                    x: region.x,
                    y: region.y,
                    width: region.width,
                    height: region.height,
                    ticks,
                })
                .collect(),
        })
        .collect();
    textures.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(textures)
}

impl Atlas {
    fn new(edge: u32, group: usize) -> Self {
        Self { edge, group, image: RgbaImage::new(0, 0), x: 0, y: 0, row: 0, width: 0, pages: Vec::new(), regions: BTreeMap::new() }
    }
    fn place(&mut self, pixels: &RgbaImage, writer: &mut Writer) -> Result<Region> {
        let (width, height) = pixels.dimensions();
        ensure!(
            width <= self.edge && height <= self.edge,
            "a texture frame exceeds its page budget"
        );
        let mut digest = Sha256::new();
        digest.update(width.to_le_bytes());
        digest.update(height.to_le_bytes());
        digest.update(pixels.as_raw());
        let digest: [u8; 32] = digest.finalize().into();
        if let Some(region) = self.regions.get(&digest) {
            return Ok(region.clone());
        }
        if self.x + width > self.edge {
            self.x = 0;
            self.y += self.row;
            self.row = 0;
        }
        if self.y + height > self.edge {
            self.flush(writer)?;
        }
        if self.image.width() == 0 { self.image = RgbaImage::new(self.edge, self.edge); }
        let region = Region {
            group: self.group,
            page: self.pages.len(),
            x: self.x,
            y: self.y,
            width,
            height,
            path: None,
        };
        image::imageops::replace(&mut self.image, pixels, self.x as i64, self.y as i64);
        self.x += width;
        self.width = self.width.max(self.x);
        self.row = self.row.max(height);
        self.regions.insert(digest, region.clone());
        Ok(region)
    }

    fn flush(&mut self, writer: &mut Writer) -> Result<()> {
        if self.width == 0 {
            return Ok(());
        }
        let pixels = self
            .image
            .view(0, 0, self.width, self.y + self.row)
            .to_image();
        let bytes = encode(&pixels)?;
        self.pages.push(writer.image(&bytes)?);
        self.image = RgbaImage::new(0, 0);
        self.x = 0;
        self.y = 0;
        self.row = 0;
        self.width = 0;
        Ok(())
    }
}

fn encode(pixels: &RgbaImage) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    image::codecs::webp::WebPEncoder::new_lossless(&mut bytes).write_image(
        pixels.as_raw(), pixels.width(), pixels.height(), image::ExtendedColorType::Rgba8,
    )?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_sampler_lossless_encoding_retains_hidden_rgb() {
        let pixels = RgbaImage::from_raw(2,1,vec![255,3,7,0,17,83,129,133]).unwrap();
        let encoded = encode(&pixels).unwrap();
        let decoded = image::load_from_memory_with_format(&encoded,image::ImageFormat::WebP).unwrap().into_rgba8();
        assert_eq!(decoded.as_raw(),pixels.as_raw());
    }
}
