use super::{check::origin, content_id, origin_id, Appearance, Domain, Kind, Origin};
use crate::identity::{resource_name, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    #[schemars(range(min = 1, max = 64))]
    pub count: u32,
    pub channels: BTreeMap<String, u32>,
}

/// Exactly one outcome for a requested construction parameter set.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub probe: Probe,
    pub build: Option<String>,
    pub problem: Option<String>,
}

/// Actual block state observed in an isolated construction preview, including unmodified tile NBT.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub id: String,
    pub registry: String,
    pub meta: u32,
    pub nbt: Option<Nbt>,
    pub item: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BuildMethod {
    Creative,
    Survival,
}

/// A whole construction preview, not a claim that every runtime machine condition passes.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Build {
    pub id: String,
    pub structure: String,
    pub probe: Probe,
    pub method: BuildMethod,
    /// The survival constructor's final result; creative construction has no return code.
    pub result: Option<i32>,
    /// Coordinates are east, down, north, with the controller facing south.
    pub size: [u32; 3],
    pub controller: [u32; 3],
    /// Dummy-world coordinate of local [0, 0, 0]; never a player-world position.
    pub origin: [i32; 3],
    pub palette: Vec<Appearance>,
    /// False when appearance capture was not requested (the data profile).
    pub rendered: bool,
    pub chunks: Vec<String>,
    pub cells: u32,
    pub notes: Vec<String>,
}

/// A StructureLib definition, not a claim that one assembled machine passes its world checks.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Structure {
    pub id: String,
    pub source: Origin,
    pub name: String,
    pub controller: String,
    pub description: Vec<String>,
    /// First requested parameter set, used for definition placement hints.
    pub probe: Probe,
    pub pieces: Vec<Piece>,
    pub variants: Vec<Variant>,
    /// An explicit missing definition is allowed only in a selection snapshot.
    pub problem: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    pub name: String,
    /// Local A/B/C coordinates: right, down, depth. Pieces have no inferred assembly transform.
    pub size: [u32; 3],
    pub anchors: Vec<[u32; 3]>,
    pub rules: Vec<Rule>,
    pub chunks: Vec<String>,
    pub cells: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RuleKind {
    Air,
    Solid,
    Element,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub symbol: String,
    pub kind: RuleKind,
    pub implementation: String,
    /// Advisory getBlocksToPlace results, never an exhaustive legality predicate or bill of materials.
    /// Null means the provider does not enumerate placement stacks.
    pub placements: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub at: [u32; 3],
    /// Index into the declaring piece's rules or the build's block palette.
    pub index: u32,
}

/// Bounded geometry records keep large definitions out of a single source row or HTTP response.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    pub id: String,
    pub cells: Vec<Cell>,
}

pub(super) fn validate(
    domain: &Domain,
    complete: bool,
    probes: &[Probe],
    text: &impl Fn(&str) -> Result<()>,
    reference: &impl Fn(Kind, &str) -> Result<()>,
) -> Result<()> {
    let shapes: BTreeMap<_, _> = domain
        .shapes
        .iter()
        .map(|shape| (&shape.id, shape))
        .collect();
    let mut used = BTreeSet::new();
    let blocks: BTreeMap<_, _> = domain
        .blocks
        .iter()
        .map(|block| (&block.id, block))
        .collect();
    let builds: BTreeMap<_, _> = domain
        .builds
        .iter()
        .map(|build| (&build.id, build))
        .collect();
    let mut used_blocks = BTreeSet::new();
    let mut used_builds = BTreeSet::new();
    let models: BTreeSet<_> = domain.models.iter().map(|model| &model.id).collect();
    let mut used_models = BTreeSet::new();
    let mut tile_positions = BTreeMap::new();
    for block in &domain.blocks {
        ensure!(
            block.id == content_id("block", block)?,
            "block state identity mismatch"
        );
        resource_name(&block.registry)?;
        ensure!(
            block.meta <= 15 && block.registry != "minecraft:air",
            "invalid constructed block state"
        );
        if let Some(nbt) = &block.nbt {
            nbt.validate()?;
            let Nbt::Compound { value } = nbt else {
                anyhow::bail!("block entity NBT must be a compound");
            };
            if ["x", "y", "z"].iter().any(|key| value.contains_key(*key)) {
                let mut position = [0; 3];
                for (axis, key) in ["x", "y", "z"].into_iter().enumerate() {
                    let Some(Nbt::Int { value }) = value.get(key) else {
                        anyhow::bail!("invalid block entity coordinates");
                    };
                    position[axis] = value.parse::<i32>().context("block entity coordinate")?;
                }
                tile_positions.insert(&block.id, position);
            }
        }
        if let Some(item) = &block.item {
            reference(Kind::Item, item)?;
        }
    }
    for shape in &domain.shapes {
        ensure!(
            shape.id == content_id("shape", shape)?,
            "shape identity mismatch"
        );
        ensure!(
            !shape.cells.is_empty() && shape.cells.len() <= 2048,
            "invalid shape chunk size"
        );
    }
    for structure in &domain.structures {
        origin(&structure.source)?;
        ensure!(
            structure.id == origin_id("structure", &structure.source)?,
            "structure identity mismatch"
        );
        reference(Kind::Item, &structure.controller)?;
        text(&structure.name)?;
        ensure!(
            Some(&structure.probe) == probes.first(),
            "structure definition parameters differ from the first requested probe"
        );
        ensure!(
            structure.description.len() <= 1024 && structure.pieces.len() <= 256,
            "structure exceeds its budget"
        );
        for line in &structure.description {
            text(line)?;
        }
        ensure!(
            structure.problem.is_some() == structure.pieces.is_empty(),
            "missing structure definition status"
        );
        if let Some(problem) = &structure.problem {
            text(problem)?;
            ensure!(
                !complete,
                "complete source contains an unavailable structure"
            );
        }
        ensure!(
            structure
                .variants
                .iter()
                .map(|variant| &variant.probe)
                .eq(probes),
            "construction variants differ from requested probes"
        );
        for variant in &structure.variants {
            ensure!(
                variant.build.is_some() != variant.problem.is_some(),
                "construction variant requires exactly one outcome"
            );
            if let Some(problem) = &variant.problem {
                text(problem)?;
                ensure!(
                    !complete,
                    "complete source contains an unavailable construction variant"
                );
                continue;
            }
            let id = variant
                .build
                .as_ref()
                .context("missing construction outcome")?;
            let build = builds.get(id).context("missing construction preview")?;
            ensure!(
                used_builds.insert(id) && build.structure == structure.id,
                "construction preview has an invalid owner"
            );
            ensure!(
                build.id == content_id("build", build)?,
                "construction preview identity mismatch"
            );
            ensure!(
                build.probe == variant.probe,
                "construction parameters differ from the requested variant"
            );
            ensure!(build.notes.len() <= 128, "too many construction messages");
            for note in &build.notes {
                text(note)?;
            }
            ensure!(
                match build.method {
                    BuildMethod::Creative => build.result.is_none(),
                    BuildMethod::Survival => matches!(build.result, Some(-1 | 0)),
                },
                "construction preview did not settle"
            );
            ensure!(
                build.size.iter().all(|&size| size > 0 && size <= 65536),
                "invalid build bounds"
            );
            ensure!(
                build.origin[1] >= 0 && build.origin[1] < 256 && build.size[1] <= 256,
                "invalid preview height"
            );
            ensure!(
                (-32768..=32767).contains(&build.origin[0])
                    && (-32768..=32767).contains(&build.origin[2])
                    && build.origin[0] as i64 + build.size[0] as i64 - 1 <= 32767
                    && build.origin[2] as i64 - build.size[2] as i64 + 1 >= -32768,
                "construction exceeds preview world bounds"
            );
            ensure!(
                build.origin[1] as i64 - build.size[1] as i64 + 1 >= 0
                    && build.origin[0] as i64 + build.controller[0] as i64 == 0
                    && build.origin[1] as i64 - build.controller[1] as i64 == 64
                    && build.origin[2] as i64 - build.controller[2] as i64 == 0,
                "construction coordinate frame does not match its controller"
            );
            ensure!(
                build.palette.len() <= 8192 && !build.palette.is_empty(),
                "invalid build palette"
            );
            let mut palette = BTreeSet::new();
            ensure!(
                !complete || build.rendered,
                "complete source omits construction models"
            );
            for appearance in &build.palette {
                ensure!(
                    blocks.contains_key(&appearance.block)
                        && palette.insert((
                            &appearance.block,
                            &appearance.model,
                            &appearance.problem
                        )),
                    "invalid or duplicate build block reference"
                );
                used_blocks.insert(&appearance.block);
                if build.rendered {
                    ensure!(
                        appearance.model.is_some() || appearance.problem.is_some(),
                        "missing model capture outcome"
                    );
                    if let Some(model) = &appearance.model {
                        ensure!(models.contains(model), "missing construction model");
                        used_models.insert(model);
                    }
                    if let Some(problem) = &appearance.problem {
                        text(problem)?;
                        ensure!(
                            !complete,
                            "complete source contains an unavailable construction model"
                        );
                    }
                } else {
                    ensure!(
                        appearance.model.is_none() && appearance.problem.is_none(),
                        "unrequested appearance has a capture result"
                    );
                }
            }
            ensure!(
                build.cells > 1 && build.cells <= 1_048_576 && build.chunks.len() <= 512,
                "invalid construction geometry budget"
            );
            let mut occupied = BTreeSet::new();
            let mut maximum = [0; 3];
            let mut minimum = [u32::MAX; 3];
            let mut previous = None;
            let mut total = 0;
            let mut used_palette = BTreeSet::new();
            for chunk in &build.chunks {
                let shape = shapes.get(chunk).context("missing construction geometry")?;
                used.insert(chunk);
                for cell in &shape.cells {
                    ensure!(occupied.insert(cell.at), "duplicate construction position");
                    ensure!(
                        (cell.index as usize) < build.palette.len(),
                        "construction cell refers to a missing block"
                    );
                    used_palette.insert(cell.index);
                    for axis in 0..3 {
                        ensure!(
                            cell.at[axis] < build.size[axis],
                            "construction cell outside bounds"
                        );
                        maximum[axis] = maximum[axis].max(cell.at[axis] + 1);
                        minimum[axis] = minimum[axis].min(cell.at[axis]);
                    }
                    if let Some(position) =
                        tile_positions.get(&build.palette[cell.index as usize].block)
                    {
                        ensure!(
                            *position
                                == [
                                    build.origin[0] + cell.at[0] as i32,
                                    build.origin[1] - cell.at[1] as i32,
                                    build.origin[2] - cell.at[2] as i32
                                ],
                            "block entity NBT disagrees with its construction position"
                        );
                    }
                    let order = [cell.at[2], cell.at[1], cell.at[0]];
                    ensure!(
                        previous.is_none_or(|last| last < order),
                        "construction cells are not in depth/row/column order"
                    );
                    previous = Some(order);
                    total += 1;
                }
            }
            ensure!(
                total == build.cells && maximum == build.size && minimum == [0; 3],
                "construction geometry count or bounds mismatch"
            );
            ensure!(
                occupied.contains(&build.controller),
                "construction preview omits its controller"
            );
            ensure!(
                used_palette.len() == build.palette.len(),
                "unused construction palette entry"
            );
        }
        let mut names = BTreeSet::new();
        for piece in &structure.pieces {
            ensure!(
                !piece.name.is_empty() && piece.name.len() <= 256 && names.insert(&piece.name),
                "invalid or duplicate structure piece"
            );
            ensure!(
                piece.size.iter().all(|&size| size > 0 && size <= 65536),
                "invalid structure bounds"
            );
            ensure!(
                piece.cells <= 1_048_576 && piece.chunks.len() <= 512 && piece.anchors.len() <= 64,
                "structure piece exceeds its budget"
            );
            ensure!(
                (piece.cells == 0 || !piece.rules.is_empty()) && piece.rules.len() <= 4096,
                "invalid structure rule count"
            );
            let mut symbols = BTreeSet::new();
            for rule in &piece.rules {
                ensure!(
                    rule.symbol.chars().count() == 1 && symbols.insert(&rule.symbol),
                    "invalid or duplicate structure symbol"
                );
                ensure!(
                    !rule.implementation.is_empty() && rule.implementation.len() <= 512,
                    "missing structure rule implementation"
                );
                if let Some(placements) = &rule.placements {
                    ensure!(
                        placements.len() <= 1024
                            && placements.windows(2).all(|pair| pair[0] < pair[1]),
                        "invalid structure placements"
                    );
                    ensure!(
                        rule.kind == RuleKind::Element || placements.is_empty(),
                        "air/solid rules cannot promise a placement item"
                    );
                    for item in placements {
                        reference(Kind::Item, item)?;
                    }
                }
            }
            let mut positions = BTreeSet::new();
            let mut maximum = [0; 3];
            let mut register = |position: [u32; 3]| -> Result<()> {
                ensure!(positions.insert(position), "duplicate structure position");
                for axis in 0..3 {
                    ensure!(
                        position[axis] < piece.size[axis],
                        "structure cell outside bounds"
                    );
                    maximum[axis] = maximum[axis].max(position[axis] + 1);
                }
                Ok(())
            };
            for &anchor in &piece.anchors {
                register(anchor)?;
            }
            let mut total = 0;
            let mut previous = None;
            for id in &piece.chunks {
                let shape = shapes.get(id).context("missing structure shape")?;
                used.insert(id);
                for cell in &shape.cells {
                    register(cell.at)?;
                    ensure!(
                        (cell.index as usize) < piece.rules.len(),
                        "missing structure rule"
                    );
                    let order = [cell.at[2], cell.at[1], cell.at[0]];
                    ensure!(
                        previous.is_none_or(|last| last < order),
                        "structure cells must follow depth/row/column order"
                    );
                    previous = Some(order);
                    total += 1;
                }
            }
            ensure!(
                total == piece.cells && maximum == piece.size,
                "structure geometry count or bounds mismatch"
            );
        }
    }
    ensure!(
        used.len() == shapes.len(),
        "unreferenced structure geometry"
    );
    ensure!(
        used_blocks.len() == blocks.len()
            && used_builds.len() == builds.len()
            && used_models.len() == models.len(),
        "unreferenced construction records"
    );
    Ok(())
}

impl Probe {
    pub fn validate_all(probes: &[Self]) -> Result<()> {
        ensure!(
            !probes.is_empty()
                && probes.len() <= 16
                && probes.windows(2).all(|pair| pair[0] < pair[1]),
            "structure probes must be 1 to 16 unique sorted parameter sets"
        );
        for probe in probes {
            ensure!(
                probe.count > 0 && probe.count <= 64 && probe.channels.len() <= 32,
                "invalid structure probe parameters"
            );
            for (name, &value) in &probe.channels {
                ensure!(
                    !name.is_empty()
                        && name.len() <= 64
                        && name.bytes().all(|byte| byte.is_ascii_lowercase()
                            || byte.is_ascii_digit()
                            || b"_-".contains(&byte))
                        && value > 0
                        && value <= 65535,
                    "invalid structure channel"
                );
            }
        }
        Ok(())
    }
}
