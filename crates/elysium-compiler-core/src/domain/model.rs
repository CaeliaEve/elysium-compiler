use super::{check::scalar, content_id, Domain};
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Appearance observed at a particular placement; equal block states may have different models.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub block: String,
    pub model: Option<String>,
    pub problem: Option<String>,
}

/// Native tessellated faces. Texture coordinates refer to one source sprite, not its packed atlas.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub id: String,
    /// Explicit native suppression (for example the second block of a double chest), never a failed capture.
    pub hidden: bool,
    pub faces: Vec<Face>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Pass {
    Solid,
    Blend,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Face {
    pub texture: String,
    pub pass: Pass,
    /// Original winding; triangles repeat their third vertex in the fourth position.
    pub vertices: [Vertex; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Vertex {
    /// Finite float32 decimal strings in local east/down/north block coordinates.
    /// Strings preserve the game's decimal representation across Java, Rust and JavaScript identities.
    pub at: [String; 3],
    /// Finite float32 decimal strings in the sprite's [0,1] range, top row at v=0.
    pub uv: [String; 2],
    /// Native per-vertex tint and shading, in unsigned RRGGBBAA byte order.
    pub color: u32,
}

pub(super) fn validate(domain: &Domain, asset: &impl Fn(&str) -> Result<()>) -> Result<()> {
    for model in &domain.models {
        ensure!(
            model.id == content_id("model", model)?,
            "model identity mismatch"
        );
        ensure!(
            model.hidden == model.faces.is_empty() && model.faces.len() <= 1024,
            "invalid model face budget"
        );
        for face in &model.faces {
            asset(&face.texture)?;
            for vertex in &face.vertices {
                for value in &vertex.at {
                    scalar(value, -256.0, 256.0)?;
                }
                for value in &vertex.uv {
                    scalar(value, 0.0, 1.0)?;
                }
            }
        }
    }
    Ok(())
}
