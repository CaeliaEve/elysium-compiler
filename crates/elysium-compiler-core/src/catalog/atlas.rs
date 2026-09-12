use super::store::Writer;
use super::{Sprite, Texture};
use crate::domain::Asset;
use crate::source::Source;
use anyhow::{ensure, Context, Result};
use image::{GenericImageView, ImageEncoder, RgbaImage};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const PAGE: u32 = 4096;

#[derive(Clone)]
struct Region {
    page: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

struct Atlas<'a> {
    writer: &'a mut Writer,
    image: RgbaImage,
    x: u32,
    y: u32,
    row: u32,
    width: u32,
    pages: Vec<String>,
    regions: BTreeMap<[u8; 32], Region>,
}

pub(super) fn pack(source: &Source, assets: &[Asset], writer: &mut Writer) -> Result<Vec<Texture>> {
    let mut atlas = Atlas {
        writer,
        image: RgbaImage::new(PAGE, PAGE),
        x: 0,
        y: 0,
        row: 0,
        width: 0,
        pages: Vec::new(),
        regions: BTreeMap::new(),
    };
    let files: BTreeMap<_, _> = source
        .files("asset")
        .map(|file| (file.path.as_str(), file))
        .collect();
    let mut pending = Vec::new();
    for asset in assets {
        let bytes = source.read_file(files[asset.path.as_str()])?;
        let format = match files[asset.path.as_str()].encoding.as_str() {
            "png" => image::ImageFormat::Png,
            "webp" => image::ImageFormat::WebP,
            _ => unreachable!("validated source"),
        };
        let mut reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(asset.width);
        limits.max_image_height = Some(asset.height);
        limits.max_alloc = Some(128 * 1024 * 1024);
        reader.limits(limits);
        let pixels = reader.decode().context("decode atlas source")?.into_rgba8();
        let mut frames = Vec::new();
        if asset.frames.is_empty() {
            frames.push((atlas.place(&pixels)?, 1));
        } else {
            for frame in &asset.frames {
                let crop = pixels
                    .view(frame.x, frame.y, frame.width, frame.height)
                    .to_image();
                frames.push((atlas.place(&crop)?, frame.ticks));
            }
        }
        pending.push((asset, frames));
    }
    atlas.flush()?;
    Ok(pending
        .into_iter()
        .map(|(asset, frames)| Texture {
            id: asset.id.clone(),
            width: frames[0].0.width,
            height: frames[0].0.height,
            interpolate: asset.interpolate,
            frames: frames
                .into_iter()
                .map(|(region, ticks)| Sprite {
                    path: atlas.pages[region.page].clone(),
                    x: region.x,
                    y: region.y,
                    width: region.width,
                    height: region.height,
                    ticks,
                })
                .collect(),
        })
        .collect())
}

impl Atlas<'_> {
    fn place(&mut self, pixels: &RgbaImage) -> Result<Region> {
        let (width, height) = pixels.dimensions();
        ensure!(
            width <= PAGE && height <= PAGE,
            "a texture frame exceeds the 4096-pixel page budget"
        );
        let mut digest = Sha256::new();
        digest.update(width.to_le_bytes());
        digest.update(height.to_le_bytes());
        digest.update(pixels.as_raw());
        let digest: [u8; 32] = digest.finalize().into();
        if let Some(region) = self.regions.get(&digest) {
            return Ok(region.clone());
        }
        if self.x + width > PAGE {
            self.x = 0;
            self.y += self.row;
            self.row = 0;
        }
        if self.y + height > PAGE {
            self.flush()?;
        }
        let region = Region {
            page: self.pages.len(),
            x: self.x,
            y: self.y,
            width,
            height,
        };
        image::imageops::replace(&mut self.image, pixels, self.x as i64, self.y as i64);
        self.x += width;
        self.width = self.width.max(self.x);
        self.row = self.row.max(height);
        self.regions.insert(digest, region.clone());
        Ok(region)
    }

    fn flush(&mut self) -> Result<()> {
        if self.width == 0 {
            return Ok(());
        }
        let pixels = self
            .image
            .view(0, 0, self.width, self.y + self.row)
            .to_image();
        let mut bytes = Vec::new();
        image::codecs::webp::WebPEncoder::new_lossless(&mut bytes).write_image(
            pixels.as_raw(),
            pixels.width(),
            pixels.height(),
            image::ExtendedColorType::Rgba8,
        )?;
        self.pages.push(self.writer.image(&bytes)?);
        self.image = RgbaImage::new(PAGE, PAGE);
        self.x = 0;
        self.y = 0;
        self.row = 0;
        self.width = 0;
        Ok(())
    }
}
