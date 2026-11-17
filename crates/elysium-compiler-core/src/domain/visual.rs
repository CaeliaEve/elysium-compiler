//! Inventory visual resources. These are bounded, reviewed programs, never
//! source-provided shaders, JavaScript, GUI coordinates, or recipe programs.
use super::{content_id, Asset};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Visual {
    pub id: String,
    pub data: VisualData,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum VisualData {
    /// 65536 native float32 values, each encoded as eight raw IEEE hex digits.
    MathHelper { bits: String },
    JavaRandom {
        stream: u16,
        seed: String,
        #[serde(rename = "haveGaussian")]
        have_gaussian: bool,
        gaussian: String,
    },
    CosmicShader {
        profile: CosmicShaderProfile,
        version: String,
        vertex: String,
        fragment: String,
    },
    Mesh { parts: Vec<InventoryPart> },
    NativeSampler {
        size: [u32; 2],
        atlas: [u32; 2],
        origin: [u32; 2],
        filter: InventorySampler,
        lod: InventoryLod,
        /// One lossless frame sheet for each actually uploaded native mip.
        levels: Vec<String>,
        frames: u32,
        columns: u32,
        timeline: Vec<InventoryStep>,
        phase: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<InventoryStep>,
    },
    CosmicInventory { renderer: String, version: String, draws: Vec<String> },
    ForestryInventory {
        version: String, profile: ForestryProfile, trig: String,
        clock: ForestryClock, draws: Vec<String>,
    },
    ForestryPart {
        mesh: String, wing: i8, state: InventoryState, sampler: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lightmap: Option<String>,
    },
    InventoryLightmap {
        pixels: String, matrix: [String;16], coordinate: [String;4],
        border: [String;4], wrap: [u32;2],
    },
    InventoryFlat {
        vertices: Vec<[String; 12]>,
        state: InventoryState,
        sampler: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        random: Option<String>,
        #[serde(default, rename = "pulseBasis", skip_serializing_if = "Option::is_none")]
        pulse_basis: Option<[String; 16]>,
    },
    InventoryCosmic {
        vertices: Vec<[String; 12]>,
        state: InventoryState,
        shader: String,
        samplers: Vec<String>,
        uniforms: CosmicBindings,
    },
    GalacticraftKey {
        version: String,
        mesh: String,
        trig: String,
        /// Native LWJGL Sys units per second, never assumed to be milliseconds.
        resolution: String,
        sampler: String,
        state: InventoryState,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CosmicShaderProfile { Avaritia, Eternal }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ForestryProfile { Bee, Butterfly }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ButterflyState { Flying, Gliding, Rising, Resting, Hover }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForestryClock {
    pub alive: bool, pub item_hash: i32, pub ticks: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<ButterflyState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_hash: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InventorySampler { pub min: u32, pub mag: u32, pub s: u32, pub t: u32 }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InventoryLod { pub min: String, pub max: String, pub bias: String }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InventoryStep { pub frame: u32, pub ticks: u32 }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CosmicLight {
    pub position: [String; 4],
    pub ambient: [String; 4],
    pub diffuse: [String; 4],
    pub attenuation: [String; 3],
    pub spot_direction: [String; 3],
    pub spot_cutoff: String,
    pub spot_cos_cutoff: String,
    pub spot_exponent: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CosmicBindings {
    pub time: String,
    pub yaw: String,
    pub pitch: String,
    pub external_scale: String,
    pub lightmix: String,
    pub opacity: String,
    pub lightlevel: [String; 3],
    pub cosmicuvs: Vec<String>,
    pub active_lights: String,
    pub texture_unit: u32,
    pub scene_color: [String; 4],
    pub lights: [CosmicLight; 2],
}

impl VisualData {
    pub fn is_program(&self) -> bool { matches!(self, Self::GalacticraftKey { .. } | Self::CosmicInventory { .. } | Self::ForestryInventory { .. }) }
    pub fn visual_references(&self) -> Vec<&str> {
        match self {
            Self::GalacticraftKey { mesh, trig, sampler, .. } => vec![mesh, trig, sampler],
            Self::CosmicInventory { draws, .. } => draws.iter().map(String::as_str).collect(),
            Self::ForestryInventory { trig, draws, .. } => std::iter::once(trig.as_str()).chain(draws.iter().map(String::as_str)).collect(),
            Self::ForestryPart { mesh, sampler, lightmap, .. } => [mesh.as_str(),sampler.as_str()].into_iter().chain(lightmap.as_deref()).collect(),
            Self::InventoryFlat { sampler, random, .. } => std::iter::once(sampler.as_str()).chain(random.as_deref()).collect(),
            Self::InventoryCosmic { shader, samplers, .. } => std::iter::once(shader.as_str()).chain(samplers.iter().map(String::as_str)).collect(),
            _ => vec![],
        }
    }
    pub fn asset_references(&self) -> Vec<&str> {
        match self {
            Self::NativeSampler { levels, .. } => levels.iter().map(String::as_str).collect(),
            Self::InventoryLightmap { pixels, .. } => vec![pixels],
            _ => vec![],
        }
    }
}

/// Publication and validation use the same typed transitive dependency walk.
/// A helper referenced only by another orphan is still an orphan.
pub fn content_closure<'a>(roots: impl IntoIterator<Item = &'a str>, rows: &[Visual]) -> Result<(BTreeSet<String>, BTreeSet<String>)> {
    let index: BTreeMap<_, _> = rows.iter().map(|row| (row.id.as_str(), &row.data)).collect();
    let mut pending: Vec<String> = roots.into_iter().map(str::to_owned).collect();
    let (mut visuals, mut assets) = (BTreeSet::new(), BTreeSet::new());
    while let Some(id) = pending.pop() {
        if !visuals.insert(id.clone()) { continue; }
        ensure!(visuals.len() <= 100_000, "inventory visual closure exceeds resource budget");
        let data = index.get(id.as_str()).context("inventory resource dependency is missing")?;
        pending.extend(data.visual_references().into_iter().map(str::to_owned));
        assets.extend(data.asset_references().into_iter().map(str::to_owned));
    }
    Ok((visuals, assets))
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InventoryPart {
    pub offset: [String; 3],
    pub pivot: [String; 3],
    /// Native Z, Y, X radians, applied with ModelRenderer's float degree conversion.
    pub rotation: [String; 3],
    /// Position XYZ, packed-normal XYZ and UV; two ordered triangles per native quad.
    pub vertices: Vec<[String; 8]>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InventoryLight {
    pub position: [String; 4],
    pub ambient: [String; 4],
    pub diffuse: [String; 4],
    pub specular: [String; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryState {
    pub model_view: [String; 16],
    pub projection: [String; 16],
    pub color: [String; 4],
    pub ambient: [String; 4],
    pub material_ambient: [String; 4],
    pub material_diffuse: [String; 4],
    pub material_specular: [String; 4],
    pub emission: [String; 4],
    pub lighting: bool,
    pub color_material: bool,
    pub normalize: bool,
    pub rescale_normal: bool,
    pub lights: Vec<InventoryLight>,
    pub depth: bool,
    pub depth_write: bool,
    pub depth_func: u32,
    pub blend: bool,
    pub blend_src_rgb: u32,
    pub blend_dst_rgb: u32,
    pub blend_src_alpha: u32,
    pub blend_dst_alpha: u32,
    pub alpha_test: bool,
    pub alpha_func: u32,
    pub alpha_ref: String,
    pub cull: bool,
    pub cull_face: u32,
    pub front_face: u32,
}

pub fn float(bits: &str) -> Result<f32> {
    ensure!(bits.len() == 8 && bits.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
        "native visual float must contain eight lowercase IEEE digits");
    let value = f32::from_bits(u32::from_str_radix(bits, 16)?);
    ensure!(value.is_finite(), "native visual value is not finite");
    Ok(value)
}
pub fn double(bits: &str) -> Result<f64> {
    ensure!(bits.len() == 16 && bits.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
        "native visual double must contain sixteen lowercase IEEE digits");
    let value = f64::from_bits(u64::from_str_radix(bits, 16)?);
    ensure!(value.is_finite(), "native visual double is not finite"); Ok(value)
}
fn values<const N: usize>(data: &[String; N], limit: f32) -> Result<()> {
    for data in data { ensure!(float(data)?.abs() <= limit, "native visual value exceeds budget"); }
    Ok(())
}
fn state(data: &InventoryState) -> Result<()> {
    values(&data.model_view, 1e6)?; values(&data.projection, 1e6)?;
    for color in [&data.color, &data.ambient, &data.material_ambient, &data.material_diffuse, &data.material_specular, &data.emission] {
        values(color, 1.0)?;
    }
    ensure!(!data.lighting || data.material_specular[..3].iter().all(|v| float(v).is_ok_and(|v| v == 0.0)),
        "inventory specular material requires a reviewed lighting program");
    ensure!(data.lights.len() <= 8, "too many inventory lights");
    for light in &data.lights {
        values(&light.position, 1e6)?;
        ensure!(!data.lighting || float(&light.position[3])? == 0.0, "inventory positional lighting requires attenuation semantics");
        for color in [&light.ambient, &light.diffuse, &light.specular] { values(color, 1.0)?; }
    }
    for function in [data.depth_func, data.alpha_func] { ensure!((0x0200..=0x0207).contains(&function), "invalid inventory comparison"); }
    for factor in [data.blend_src_rgb, data.blend_dst_rgb, data.blend_src_alpha, data.blend_dst_alpha] {
        ensure!([0, 1, 0x300, 0x301, 0x302, 0x303, 0x304, 0x305, 0x306, 0x307, 0x308].contains(&factor), "unreviewed inventory blend factor");
    }
    ensure!((0.0..=1.0).contains(&float(&data.alpha_ref)?), "invalid inventory alpha reference");
    ensure!([0x404, 0x405, 0x408].contains(&data.cull_face) && [0x900, 0x901].contains(&data.front_face), "invalid inventory culling");
    Ok(())
}

fn validate_sampler(data: &VisualData, assets: &BTreeMap<&str, &Asset>) -> Result<()> {
    let VisualData::NativeSampler { size, atlas, origin, filter, lod, levels, frames, columns, timeline, phase, anchor } = data else {
        anyhow::bail!("missing native sampler resource");
    };
    ensure!(size.iter().all(|v| (1..=4096).contains(v)) && atlas.iter().all(|v| (1..=65536).contains(v))
        && (0..2).all(|i| u64::from(origin[i]) + u64::from(size[i]) <= u64::from(atlas[i])), "invalid native sampler region");
    ensure!((1..=4097).contains(frames) && (1..=*frames).contains(columns) && !levels.is_empty() && levels.len() <= 13,
        "invalid native sampler frame/mip census");
    ensure!([0x2600,0x2601,0x2700,0x2701,0x2702,0x2703].contains(&filter.min)
        && [0x2600,0x2601].contains(&filter.mag) && [0x2901,0x812f].contains(&filter.s) && [0x2901,0x812f].contains(&filter.t),
        "unreviewed native sampler filter/wrap");
    ensure!(float(&lod.min)?.abs() <= 1000.0 && float(&lod.max)?.abs() <= 1000.0
        && float(&lod.min)? <= float(&lod.max)? && float(&lod.bias)?.abs() <= 16.0, "invalid native sampler LOD");
    let sparse = origin != &[0,0] || size != atlas;
    if sparse {
        ensure!(filter.mag == 0x2600 && [0x2600,0x2700,0x2702].contains(&filter.min), "blurred native sampler is missing atlas neighbours");
        let alignment = 1u32 << (levels.len() - 1);
        ensure!(size.iter().chain(origin).all(|v| v % alignment == 0) && size.iter().all(|v| *v >= alignment),
            "native sampler mip footprint crosses an atlas region");
    }
    if [0x2600,0x2601].contains(&filter.min) { ensure!(levels.len() == 1, "unfiltered native sampler contains unused mip levels"); }
    ensure!(!timeline.is_empty() && timeline.len() <= 4096, "invalid native sampler timeline");
    let mut cycle = 0u64;
    let mut used = BTreeSet::new();
    for step in timeline {
        ensure!(step.frame < *frames && (1..=72000).contains(&step.ticks), "invalid native sampler frame step");
        used.insert(step.frame);cycle += u64::from(step.ticks);
    }
    if let Some(anchor) = anchor {
        ensure!(anchor.frame < *frames && anchor.ticks > 0 && u64::from(anchor.ticks) <= cycle, "invalid native sampler upload anchor");
        used.insert(anchor.frame);
    }
    ensure!(used.len() == *frames as usize && cycle <= 72000 && u64::from(*phase) < cycle, "invalid native sampler phase or orphan frame");
    let rows = frames.div_ceil(*columns);
    let mut pixels = 0u64;
    for (level, id) in levels.iter().enumerate() {
        let asset = assets.get(id.as_str()).context("native sampler mip asset is missing")?;
        let width = (size[0] >> level).max(1) * columns;
        let height = (size[1] >> level).max(1) * rows;
        ensure!(width <= 4096 && height <= 4096 && asset.width == width && asset.height == height
            && asset.visual.is_none() && asset.frames.is_empty() && !asset.interpolate,
            "native sampler mip is not an exact still frame sheet");
        pixels += u64::from(width) * u64::from(height);
    }
    ensure!(pixels <= 16 * 1024 * 1024, "native sampler mip sheets exceed pixel budget");
    Ok(())
}

fn validate_lightmap(data:&VisualData,assets:&BTreeMap<&str,&Asset>)->Result<()> {
    let VisualData::InventoryLightmap {pixels,matrix,coordinate,border,wrap}=data else {anyhow::bail!("missing native inventory lightmap");};
    values(matrix,1e6)?;values(coordinate,1e6)?;values(border,1.0)?;
    ensure!(border.iter().all(|v|float(v).is_ok_and(|v|v>=0.0)) && wrap.iter().all(|v|[0x2900,0x812f].contains(v)),
        "unreviewed lightmap border/wrap");
    let q=(0..4).map(|i|Ok(float(&matrix[i*4+3])?*float(&coordinate[i])?)).collect::<Result<Vec<f32>>>()?.into_iter().sum::<f32>();
    ensure!(q.is_finite()&&q!=0.0,"native lightmap has an invalid homogeneous coordinate");
    let asset=assets.get(pixels.as_str()).context("native lightmap pixels are missing")?;
    ensure!(asset.width==16&&asset.height==16&&asset.frames.is_empty()&&!asset.interpolate&&asset.visual.is_none(),
        "native lightmap must be its uploaded 16 by 16 still texture");
    Ok(())
}

fn inventory_vertices(vertices: &[[String; 12]]) -> Result<()> {
    ensure!(!vertices.is_empty() && vertices.len() % 4 == 0 && vertices.len() <= 24576, "inventory draw lost ordered native quads");
    for vertex in vertices {
        for (index, bits) in vertex.iter().enumerate() {
            let number = float(bits)?;
            ensure!(number.abs() <= if index < 3 { 256.0 } else { 1.0 }, "inventory draw vertex exceeds bounds");
            if index >= 6 { ensure!(number >= 0.0, "negative inventory UV/color"); }
        }
    }
    Ok(())
}
fn sampler_uv(data: &VisualData, min_u: f32, max_u: f32, min_v: f32, max_v: f32) -> Result<()> {
    let VisualData::NativeSampler { size, atlas, origin, .. } = data else { anyhow::bail!("missing native sampler"); };
    for (axis, (low, high)) in [(min_u,max_u),(min_v,max_v)].into_iter().enumerate() {
        let end = (origin[axis] + size[axis]) as f32 / atlas[axis] as f32;
        let sparse = origin != &[0,0] || size != atlas;
        ensure!(low.is_finite() && high.is_finite() && low < high && low >= origin[axis] as f32 / atlas[axis] as f32
            && if sparse { high < end } else { high <= end }, "inventory UVs escape their native sampler region");
    }
    Ok(())
}
fn inventory_uv(vertices: &[[String; 12]], sampler: &VisualData) -> Result<()> {
    let (mut min_u,mut max_u,mut min_v,mut max_v)=(f32::INFINITY,f32::NEG_INFINITY,f32::INFINITY,f32::NEG_INFINITY);
    for vertex in vertices {
        let (u,v)=(float(&vertex[6])?,float(&vertex[7])?);min_u=min_u.min(u);max_u=max_u.max(u);min_v=min_v.min(v);max_v=max_v.max(v);
    }
    sampler_uv(sampler,min_u,max_u,min_v,max_v)
}
fn cosmic_bindings(data: &CosmicBindings) -> Result<()> {
    ensure!(float(&data.time)? == 0.0 && float(&data.yaw)? == 0.0 && float(&data.pitch)? == 0.0
        && float(&data.external_scale)? == 25.0 && data.texture_unit == 0, "cosmic shader was not captured in its native inventory branch");
    ensure!((0.0..=1.0).contains(&float(&data.lightmix)?) && (0.0..=16.0).contains(&float(&data.opacity)?)
        && (0.0..=8.0).contains(&float(&data.active_lights)?), "invalid native cosmic light/opacity binding");
    values(&data.lightlevel,16.0)?;values(&data.scene_color,2.0)?;
    ensure!(data.cosmicuvs.len() == 40, "native cosmic symbol census changed");
    for value in &data.cosmicuvs { ensure!((0.0..=1.0).contains(&float(value)?), "invalid native cosmic UV"); }
    for light in &data.lights {
        values(&light.position,1e6)?;values(&light.ambient,1.0)?;values(&light.diffuse,1.0)?;
        values(&light.attenuation,1e6)?;values(&light.spot_direction,1e6)?;
        ensure!(light.attenuation.iter().all(|v|float(v).is_ok_and(|v|v>=0.0))
            && light.attenuation.iter().any(|v|float(v).is_ok_and(|v|v>0.0)), "invalid native light attenuation");
        let cutoff=float(&light.spot_cutoff)?;
        ensure!((0.0..=90.0).contains(&cutoff) || cutoff == 180.0, "invalid native spotlight cutoff");
        ensure!((-1.0..=1.0).contains(&float(&light.spot_cos_cutoff)?) && (0.0..=128.0).contains(&float(&light.spot_exponent)?), "invalid native spotlight binding");
    }
    Ok(())
}

pub fn validate(rows: &[Visual], assets: &[Asset]) -> Result<()> {
    let index: BTreeMap<_, _> = rows.iter().map(|row| (row.id.as_str(), &row.data)).collect();
    ensure!(rows.len() <= 100_000 && rows.len() == index.len(), "invalid native visual resource census");
    let textures: BTreeMap<_, _> = assets.iter().map(|row| (row.id.as_str(), row)).collect();
    let mut roots = Vec::new();
    let mut random_streams = BTreeSet::new();
    for asset in assets {
        if let Some(root) = &asset.visual {
            ensure!(asset.frames.is_empty() && !asset.interpolate, "program preview cannot also be a timeline");
            ensure!(index.get(root.as_str()).is_some_and(|data| data.is_program()), "missing inventory visual program");
            roots.push(root.as_str());
        }
    }
    for row in rows {
        ensure!(row.id == content_id("visual", row)?, "native visual identity mismatch: {}", row.id);
        match &row.data {
            VisualData::CosmicShader { profile, version, vertex, fragment } => {
                let (expected_version, expected_fragment) = match profile {
                    CosmicShaderProfile::Avaritia => ("1.77", "275a574e59940cf966078b38e66aa0de00e1dbee280013b47692f4264943f34d"),
                    CosmicShaderProfile::Eternal => ("1.2.3", "c074f2a43a76204d5a31ea7aeba4cbd06dda5000997c5bd5119913f532d148b4"),
                };
                ensure!(version == expected_version && fragment == expected_fragment
                    && vertex == "31a4078eeb6694f0625813f2d88aa152ddd84a38763da03e1ebdd81c0a476824",
                    "native cosmic shader differs from its reviewed profile");
            }
            VisualData::JavaRandom { stream, seed, gaussian, .. } => {
                ensure!((1..=4096).contains(stream) && random_streams.insert(*stream), "native Random stream authority is duplicated");
                ensure!(!seed.is_empty() && seed.len() <= 15 && seed.bytes().all(|c| c.is_ascii_digit())
                    && (seed == "0" || !seed.starts_with('0')) && seed.parse::<u64>().is_ok_and(|seed| seed < (1u64 << 48)),
                    "invalid native Random seed");
                double(gaussian)?;
            }
            VisualData::MathHelper { bits } => {
                ensure!(bits.len() == 65536 * 8, "invalid native MathHelper lookup length");
                for value in bits.as_bytes().chunks_exact(8) {
                    ensure!(float(std::str::from_utf8(value)?)?.abs() <= 1.0, "invalid native sine lookup value");
                }
            }
            VisualData::Mesh { parts } => {
                ensure!(!parts.is_empty() && parts.len() <= 128, "invalid inventory model part census");
                let mut count = 0;
                for part in parts {
                    values(&part.offset, 256.0)?; values(&part.pivot, 256.0)?; values(&part.rotation, 1000.0)?;
                    ensure!(!part.vertices.is_empty() && part.vertices.len() % 6 == 0, "inventory model lost ordered quads");
                    count += part.vertices.len();
                    ensure!(count <= 24576, "inventory geometry exceeds vertex budget");
                    for vertex in &part.vertices {
                        for (i, v) in vertex.iter().enumerate() {
                            let v = float(v)?;
                            ensure!(v.abs() <= if i < 3 { 256.0 } else { 1.0 }, "inventory vertex exceeds bounds");
                            if i >= 6 { ensure!(v >= 0.0, "negative inventory texture coordinate"); }
                        }
                    }
                }
            }
            VisualData::NativeSampler { .. } => validate_sampler(&row.data, &textures)?,
            VisualData::InventoryLightmap { .. } => validate_lightmap(&row.data,&textures)?,
            VisualData::ForestryPart {mesh,wing,state:native,sampler,lightmap} => {
                ensure!((-1..=1).contains(wing),"invalid native Forestry wing role");state(native)?;
                ensure!(matches!(index.get(mesh.as_str()),Some(VisualData::Mesh {parts}) if parts.len()==1),"Forestry draw needs one native model part");
                ensure!(matches!(index.get(sampler.as_str()),Some(VisualData::NativeSampler {size,atlas,origin,frames,..})
                    if size==atlas&&origin==&[0,0]&&*frames==1),"Forestry model lost its standalone native texture");
                if let Some(id)=lightmap {ensure!(matches!(index.get(id.as_str()),Some(VisualData::InventoryLightmap {..})),"native Forestry lightmap is missing");}
                let VisualData::Mesh {parts}=index[mesh.as_str()] else {unreachable!()};
                if *wing!=0 {ensure!(float(&parts[0].rotation[0])?==0.0,"dynamic Forestry wing already contains a baked Z angle");}
            }
            VisualData::ForestryInventory {version,profile,trig,clock,draws} => {
                ensure!(version=="4.10.17"&&matches!(index.get(trig.as_str()),Some(VisualData::MathHelper {..})),"unreviewed Forestry inventory version/lookup");
                let butterfly=*profile==ForestryProfile::Butterfly;
                ensure!(clock.state.is_some()==butterfly&&clock.state_hash.is_some()==butterfly&& (clock.alive||clock.item_hash==0),"Forestry item and entity clocks are not distinct");
                ensure!(if butterfly {draws.len()==5}else{draws.len()==10||draws.len()==11},"Forestry native part census changed");
                let mut count=0;let mut seen=BTreeSet::new();
                for (i,id) in draws.iter().enumerate(){
                    ensure!(seen.insert(id),"Forestry native part was repeated");
                    let Some(VisualData::ForestryPart {mesh,wing,..})=index.get(id.as_str()) else {anyhow::bail!("Forestry scene contains a non-model draw");};
                    let expected=if butterfly {match i {0=>1,3=>-1,_=>0}}else{match i {8=>-1,9=>1,_=>0}};
                    ensure!(*wing==expected,"Forestry native part/wing order changed");
                    let Some(VisualData::Mesh {parts})=index.get(mesh.as_str()) else {anyhow::bail!("Forestry native mesh is missing");};
                    count+=parts.iter().map(|p|p.vertices.len()).sum::<usize>();
                }
                ensure!(count<=24576,"Forestry scene exceeds native vertex budget");
            }
            VisualData::CosmicInventory { renderer, version, draws } => {
                let expected = match renderer.as_str() {
                    "fox.spiteful.avaritia.render.CosmicItemRenderer" | "fox.spiteful.avaritia.render.CosmicBowRenderer" => "1.77",
                    "singulariteam.eternalsingularity.render.EternalItemRenderer" => "1.2.3",
                    "gregtech.common.render.items.UniversiumRenderer" => "5.09.51.482",
                    _ => anyhow::bail!("unreviewed native cosmic renderer"),
                };
                ensure!(version == expected && !draws.is_empty() && draws.len() <= 64, "invalid native cosmic scene version/census");
                let (mut shaders,mut vertices,mut pulses) = (0,0,0);
                for draw in draws {
                    match index.get(draw.as_str()).context("native inventory layer is missing")? {
                        VisualData::InventoryFlat { random, vertices: data, .. } => {
                            ensure!(random.is_none() || expected == "1.2.3", "native pulse is not an Eternal renderer");
                            pulses += usize::from(random.is_some());vertices += data.len();
                        }
                        VisualData::InventoryCosmic { shader, vertices: data, .. } => {
                            let Some(VisualData::CosmicShader { profile, .. }) = index.get(shader.as_str()) else { anyhow::bail!("native cosmic shader is missing"); };
                            ensure!(matches!(profile, CosmicShaderProfile::Eternal) == (expected == "1.2.3"), "native cosmic shader profile does not match its renderer");shaders += 1;vertices += data.len();
                        }
                        _ => anyhow::bail!("native scene contains a non-draw resource"),
                    }
                }
                ensure!(shaders > 0, "native cosmic shader is disabled or was not observed");
                ensure!(vertices <= 24576 && pulses <= 1, "native cosmic scene exceeds vertex/pulse budget");
            }
            VisualData::InventoryFlat { vertices, state: native, sampler, random, pulse_basis } => {
                inventory_vertices(vertices)?;state(native)?;
                let texture = index.get(sampler.as_str()).context("native flat sampler is missing")?;
                inventory_uv(vertices, texture)?;
                ensure!(random.is_some() == pulse_basis.is_some(), "native pulse lost its random stream/basis");
                if let Some(random) = random {
                    ensure!(matches!(index.get(random.as_str()), Some(VisualData::JavaRandom { .. })), "native pulse stream is missing");
                    values(pulse_basis.as_ref().unwrap(),1e6)?;
                    ensure!(vertices.len() == 4, "native pulse is not one quad");
                    for (i,vertex) in vertices.iter().enumerate() {
                        let (x,y) = (if i >= 2 {16.0} else {0.0}, if i == 1 || i == 2 {16.0} else {0.0});
                        ensure!(float(&vertex[0])? == x && float(&vertex[1])? == y
                            && float(&vertex[2])? == 0.0, "native pulse lost its offset-expanded quad rule");
                    }
                }
            }
            VisualData::InventoryCosmic { vertices, state: native, shader, samplers, uniforms } => {
                inventory_vertices(vertices)?;state(native)?;cosmic_bindings(uniforms)?;
                ensure!(matches!(index.get(shader.as_str()), Some(VisualData::CosmicShader { .. })) && samplers.len() == 11, "native cosmic draw lost its shader/symbol samplers");
                for sampler in samplers { ensure!(matches!(index.get(sampler.as_str()), Some(VisualData::NativeSampler { .. })), "native cosmic sampler is missing"); }
                inventory_uv(vertices,index[samplers[0].as_str()])?;
                for (i,sampler) in samplers[1..].iter().enumerate() {
                    let uv=&uniforms.cosmicuvs[i*4..i*4+4];
                    sampler_uv(index[sampler.as_str()],float(&uv[0])?,float(&uv[2])?,float(&uv[1])?,float(&uv[3])?)?;
                }
            }
            VisualData::GalacticraftKey { version, mesh, trig, resolution, sampler, state: native } => {
                ensure!(version == "3.3.13-GTNH", "unreviewed Galacticraft key renderer version");
                ensure!(matches!(index.get(mesh.as_str()), Some(VisualData::Mesh { parts }) if parts.len() == 5), "key requires its five native model parts");
                ensure!(matches!(index.get(trig.as_str()), Some(VisualData::MathHelper { .. })), "native key lookup is missing");
                ensure!(!resolution.is_empty() && !resolution.starts_with('0') && resolution.bytes().all(|c| c.is_ascii_digit())
                    && resolution.parse::<u64>().is_ok_and(|v| v > 0 && v <= i64::MAX as u64), "invalid native Sys resolution");
                ensure!(matches!(index.get(sampler.as_str()), Some(VisualData::NativeSampler { size, atlas, origin, frames, .. })
                    if size == atlas && origin == &[0,0] && *frames == 1), "key model sampler must retain its whole native texture");
                state(native)?;
            }
        }
    }
    for root in roots.iter().copied().collect::<BTreeSet<_>>() {
        let mut pending = vec![root];let mut visuals = BTreeSet::new();let mut pixels = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !visuals.insert(id) { continue; }
            ensure!(visuals.len() <= 1024, "native inventory program exceeds resource closure budget");
            let data = index[id];pending.extend(data.visual_references());pixels.extend(data.asset_references());
        }
        let size: u64 = pixels.into_iter().map(|id|u64::from(textures[id].width)*u64::from(textures[id].height)).sum();
        ensure!(size <= 32*1024*1024, "native inventory program exceeds pixel closure budget");
    }
    let (used, _) = content_closure(roots, rows)?;
    ensure!(used.len() == rows.len(), "orphaned inventory visual resource");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_float_bits_preserve_negative_zero_and_reject_non_finite() {
        assert_eq!(float("80000000").unwrap().to_bits(), (-0.0f32).to_bits());
        assert_eq!(float("3f800000").unwrap(), 1.0);
        for invalid in ["3F800000", "3f80000", "7f800000", "7fc00000", "--------"] { assert!(float(invalid).is_err()); }
        assert_eq!(double("8000000000000000").unwrap().to_bits(), (-0.0f64).to_bits());
        for invalid in ["7ff0000000000000", "7ff8000000000000", "3FF0000000000000", "00000000"] { assert!(double(invalid).is_err()); }
    }
    #[test]
    fn resources_cannot_be_published_without_a_preview_owner() {
        let mut row = Visual { id: String::new(), data: VisualData::MathHelper { bits: "00000000".repeat(65536) } };
        row.id = content_id("visual", &row).unwrap();
        assert!(validate(&[row], &[]).unwrap_err().to_string().contains("orphaned"));
    }
    #[test]
    fn native_sampler_requires_every_mip_and_a_real_timeline_phase() {
        let mut data = serde_json::json!({"kind":"nativeSampler","size":[2,2],"atlas":[8,8],"origin":[2,2],
            "filter":{"min":9986,"mag":9728,"s":10497,"t":10497},
            "lod":{"min":"c47a0000","max":"447a0000","bias":"00000000"},
            "frames":2,"columns":2,"levels":["asset_level0","asset_level1"],
            "timeline":[{"frame":1,"ticks":3},{"frame":0,"ticks":2}],"phase":4});
        let assets: Vec<Asset> = [("asset_level0",4,2),("asset_level1",2,1)].into_iter().map(|(id,width,height)|
            serde_json::from_value(serde_json::json!({"id":id,"path":"assets/owned.png","width":width,"height":height,
                "frames":[],"interpolate":false,"source":{"kind":"capture","location":"owned"}})).unwrap()).collect();
        let index = assets.iter().map(|row|(row.id.as_str(),row)).collect();
        validate_sampler(&serde_json::from_value(data.clone()).unwrap(),&index).unwrap();
        data["phase"] = 5.into();
        assert!(validate_sampler(&serde_json::from_value(data.clone()).unwrap(),&index).is_err());
        data["phase"] = 4.into();data["filter"]["min"] = 9987.into();
        assert!(validate_sampler(&serde_json::from_value(data).unwrap(),&index).is_err());
    }
    #[test]
    fn lightmap_keeps_clamp_border_and_rejects_an_atlas_or_timeline() {
        let mut data=serde_json::json!({"kind":"inventoryLightmap","pixels":"asset_light",
            "matrix":["3b800000","00000000","00000000","00000000","00000000","3b800000","00000000","00000000",
                "00000000","00000000","3b800000","00000000","3d000000","3d000000","3d000000","3f800000"],
            "coordinate":["43700000","43700000","00000000","3f800000"],"border":["00000000","00000000","00000000","00000000"],"wrap":[10496,10496]});
        let mut asset:Asset=serde_json::from_value(serde_json::json!({"id":"asset_light","path":"assets/light.png","width":16,"height":16,
            "frames":[],"interpolate":false,"source":{"kind":"capture","location":"native lightmap"}})).unwrap();
        validate_lightmap(&serde_json::from_value(data.clone()).unwrap(),&BTreeMap::from([("asset_light",&asset)])).unwrap();
        asset.width=32;
        assert!(validate_lightmap(&serde_json::from_value(data.clone()).unwrap(),&BTreeMap::from([("asset_light",&asset)])).is_err());
        asset.width=16;data["wrap"][0]=10497.into();
        assert!(validate_lightmap(&serde_json::from_value(data).unwrap(),&BTreeMap::from([("asset_light",&asset)])).is_err());
    }
}
