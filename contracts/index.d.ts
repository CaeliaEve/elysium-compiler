/* Generated from the Rust contract. Run cargo schema and npm run generate to update. */

/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AssetKind".
 */
export type AssetKind = "resource" | "capture" | "block" | "fluid" | "entity";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Nbt".
 */
export type Nbt =
  | {
      type: "byte";
      value: string;
    }
  | {
      type: "short";
      value: string;
    }
  | {
      type: "int";
      value: string;
    }
  | {
      type: "long";
      value: string;
    }
  | {
      type: "float";
      value: string;
    }
  | {
      type: "double";
      value: string;
    }
  | {
      type: "byte_array";
      value: string;
    }
  | {
      type: "string";
      value: string;
    }
  | {
      element: string;
      type: "list";
      value: Nbt[];
    }
  | {
      type: "compound";
      value: {
        [k: string]: Nbt;
      };
    }
  | {
      type: "int_array";
      value: string[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BuildMethod".
 */
export type BuildMethod = "creative" | "survival";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Kind".
 */
export type Kind = "item" | "fluid";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitKind".
 */
export type CircuitKind = "line" | "parts";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Pass".
 */
export type Pass = "solid" | "blend";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PropertyValue".
 */
export type PropertyValue =
  | {
      kind: "integer";
      value: string;
    }
  | {
      kind: "decimal";
      value: string;
    }
  | {
      amount: string;
      kind: "quantity";
      unit: Unit;
    }
  | {
      kind: "text";
      text: string;
    }
  | {
      kind: "flag";
      value: boolean;
    }
  | {
      kind: "reference";
      target: Reference;
    }
  | {
      kind: "list";
      values: PropertyValue[];
    }
  | {
      kind: "map";
      values: {
        [k: string]: PropertyValue;
      };
    }
  | {
      kind: "symbol";
      namespace: string;
      value: string;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Unit".
 */
export type Unit =
  "count" | "tick" | "eu" | "eu_per_tick" | "mb" | "kelvin" | "pascal" | "rpm" | "mana" | "lp" | "vis" | "percent";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Consumption".
 */
export type Consumption =
  | {
      kind: "consume";
    }
  | {
      kind: "keep";
    }
  | {
      kind: "damage";
      points: number;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Match".
 */
export type Match =
  | {
      kind: "exact";
    }
  | {
      exclusive: boolean;
      kind: "ore";
      name: string;
    }
  | {
      kind: "wildcard";
      meta: boolean;
      nbt: boolean;
    }
  | {
      absent: string[];
      keys: string[];
      kind: "tags";
      meta: boolean;
      present: string[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MagicKind".
 */
export type MagicKind = "arcane" | "crucible" | "infusion";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Edit".
 */
export type Edit =
  | {
      kind: "patch";
      limits: {
        [k: string]: number;
      };
      set: {
        [k: string]: Nbt;
      };
    }
  | {
      base: Stack;
      /**
       * None copies all root keys. A nonempty list filters only the root merge.
       */
      keys?: string[] | null;
      kind: "merge";
      /**
       * Both input and base must be armor or have at least one native tool class.
       */
      tools: boolean;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OutputRole".
 */
export type OutputRole = "result" | "return";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ResearchFlag".
 */
export type ResearchFlag =
  "auto" | "concealed" | "hidden" | "lost" | "round" | "secondary" | "special" | "stub" | "virtual";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SpeciesKind".
 */
export type SpeciesKind = "bee" | "tree";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RuleKind".
 */
export type RuleKind = "air" | "solid" | "element";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Element".
 */
export type Element =
  | {
      asset: string;
      height: number;
      kind: "clip";
      track: string;
      width: number;
      x: number;
      y: number;
      z: number;
    }
  | {
      asset: string;
      height: number;
      kind: "sprite";
      width: number;
      x: number;
      y: number;
      z: number;
    }
  | {
      direction: Direction;
      height: number;
      kind: "slot";
      slot: number;
      substance: Kind;
      width: number;
      x: number;
      y: number;
      z: number;
    }
  | {
      height: number;
      /**
       * Index into the owning recipe's magic aspect costs, never an inventory item.
       */
      index: number;
      kind: "cost";
      width: number;
      x: number;
      y: number;
      z: number;
    }
  | {
      align: Align;
      color: number;
      kind: "text";
      text: string;
      x: number;
      y: number;
      z: number;
    }
  | {
      color: number;
      height: number;
      kind: "rectangle";
      width: number;
      x: number;
      y: number;
      z: number;
    }
  | {
      height: number;
      kind: "tooltip";
      lines: string[];
      width: number;
      x: number;
      y: number;
      z: number;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Direction".
 */
export type Direction = "input" | "output";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Align".
 */
export type Align = "left" | "center" | "right";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Table".
 */
export type Table =
  | {
      kind: "aspects";
      records: Aspect[];
    }
  | {
      kind: "blocks";
      records: Block[];
    }
  | {
      kind: "browse";
      records: Entry[];
    }
  | {
      kind: "builds";
      records: Build[];
    }
  | {
      kind: "categories";
      records: Category[];
    }
  | {
      kind: "circuits";
      records: Circuit[];
    }
  | {
      kind: "fluids";
      records: Fluid[];
    }
  | {
      kind: "groups";
      records: Group[];
    }
  | {
      kind: "index";
      records: RecipeIndex[];
    }
  | {
      kind: "items";
      records: Item[];
    }
  | {
      kind: "lineage";
      records: Lineage[];
    }
  | {
      kind: "links";
      records: Links[];
    }
  | {
      kind: "materials";
      records: Material[];
    }
  | {
      kind: "models";
      records: Model[];
    }
  | {
      kind: "mutations";
      records: Mutation[];
    }
  | {
      kind: "recipes";
      records: Recipe[];
    }
  | {
      kind: "research";
      records: Research[];
    }
  | {
      kind: "shapes";
      records: Shape[];
    }
  | {
      kind: "species";
      records: Species[];
    }
  | {
      kind: "strings";
      records: Text[];
    }
  | {
      kind: "structures";
      records: Structure[];
    }
  | {
      kind: "textures";
      records: Texture[];
    }
  | {
      kind: "topics";
      records: Topic[];
    }
  | {
      kind: "tracks";
      records: Track[];
    }
  | {
      kind: "views";
      records: View[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "TopicKind".
 */
export type TopicKind = "material" | "circuit" | "bee" | "tree" | "structure" | "aspect" | "research";

/**
 * Owns the schema exported to consumers; it is not a runtime payload itself.
 */
export interface Contract {
  catalog: Manifest;
  domain: Domain;
  environment: Environment;
  pointer: Pointer;
  source: SourceManifest;
  table: Table;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Manifest".
 */
export interface Manifest {
  compiler: string;
  counts: {
    [k: string]: number;
  };
  environment: string;
  files: File[];
  format: "elysium.catalog";
  id: string;
  revision: 10;
  scope: string;
  source: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "File".
 */
export interface File {
  bytes: number;
  encoding: string;
  first?: string | null;
  kind: string;
  last?: string | null;
  path: string;
  rows: number;
  sha256: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Domain".
 */
export interface Domain {
  aspects: Aspect[];
  assets: Asset[];
  blocks: Block[];
  builds: Build[];
  categories: Category[];
  circuits: Circuit[];
  fluids: Fluid[];
  groups: Group[];
  items: Item[];
  materials: Material[];
  models: Model[];
  mutations: Mutation[];
  recipes: Recipe[];
  research: Research[];
  shapes: Shape[];
  species: Species[];
  strings: Text[];
  structures: Structure[];
  tracks: Track[];
  views: View[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Aspect".
 */
export interface Aspect {
  color: number;
  /**
   * Empty for a primal aspect; otherwise the two components used by the game, including repeats.
   */
  components: string[];
  description: string;
  /**
   * Observed player knowledge; null when its client cache has not arrived.
   */
  discovered?: boolean | null;
  icon?: string | null;
  id: string;
  name: string;
  source: Origin;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Origin".
 */
export interface Origin {
  handler: string;
  key: string;
  owner: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Asset".
 */
export interface Asset {
  /**
   * Empty means a still image; otherwise every frame specifies its crop and tick duration.
   */
  frames: Frame[];
  height: number;
  id: string;
  interpolate: boolean;
  path: string;
  source: AssetOrigin;
  width: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Frame".
 */
export interface Frame {
  height: number;
  ticks: number;
  width: number;
  x: number;
  y: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AssetOrigin".
 */
export interface AssetOrigin {
  kind: AssetKind;
  location: string;
}
/**
 * Actual block state observed in an isolated construction preview, including unmodified tile NBT.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Block".
 */
export interface Block {
  id: string;
  item?: string | null;
  meta: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * A whole construction preview, not a claim that every runtime machine condition passes.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Build".
 */
export interface Build {
  cells: number;
  chunks: string[];
  /**
   * @minItems 3
   * @maxItems 3
   */
  controller: [number, number, number];
  id: string;
  method: BuildMethod;
  notes: string[];
  /**
   * Dummy-world coordinate of local [0, 0, 0]; never a player-world position.
   *
   * @minItems 3
   * @maxItems 3
   */
  origin: [number, number, number];
  palette: Appearance[];
  probe: Probe;
  /**
   * False when appearance capture was not requested (the data profile).
   */
  rendered: boolean;
  /**
   * The survival constructor's final result; creative construction has no return code.
   */
  result?: number | null;
  /**
   * Coordinates are east, down, north, with the controller facing south.
   *
   * @minItems 3
   * @maxItems 3
   */
  size: [number, number, number];
  structure: string;
}
/**
 * Appearance observed at a particular placement; equal block states may have different models.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Appearance".
 */
export interface Appearance {
  block: string;
  model?: string | null;
  problem?: string | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Probe".
 */
export interface Probe {
  channels: {
    [k: string]: number;
  };
  count: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Category".
 */
export interface Category {
  icon?: Reference | null;
  id: string;
  machines: Reference[];
  name: string;
  order: number;
  source: Origin;
  view?: string | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Reference".
 */
export interface Reference {
  id: string;
  kind: Kind;
}
/**
 * A circuit family or component family defined by the game's diagram provider. Adjacent steps describe progression, not a fabricated crafting recipe.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Circuit".
 */
export interface Circuit {
  boards: string[];
  id: string;
  kind: CircuitKind;
  name: string;
  order: number;
  source: Origin;
  steps: CircuitStep[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitStep".
 */
export interface CircuitStep {
  item: string;
  tier?: Tier | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Tier".
 */
export interface Tier {
  level: number;
  name: string;
  /**
   * Nominal voltage in EU per packet.
   */
  voltage: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Fluid".
 */
export interface Fluid {
  density: number;
  gaseous: boolean;
  icon?: string | null;
  id: string;
  luminosity: number;
  name: string;
  nbt?: Nbt | null;
  /**
   * The actual global Forge fluid key, which need not contain a namespace.
   */
  registry: string;
  /**
   * Kelvin.
   */
  temperature: number;
  viscosity: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Group".
 */
export interface Group {
  collapsed: boolean;
  id: string;
  members: string[];
  name?: string | null;
  order: number;
  representative: string;
  source: Origin;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Item".
 */
export interface Item {
  /**
   * Actual ItemArmor classification; durability alone does not identify armor or tools.
   */
  armor: boolean;
  /**
   * Thaumcraft's returned object tags; null means it provided no value, not zero aspects.
   */
  aspects?: AspectAmount[] | null;
  durability: number;
  icon?: string | null;
  id: string;
  meta: number;
  /**
   * Reference to a localized, Minecraft-formatted string.
   */
  name: string;
  nbt?: Nbt | null;
  /**
   * NEI order; null denotes an item discovered only through a recipe.
   */
  order?: number | null;
  registry: string;
  stackLimit: number;
  /**
   * Exact OreDictionary membership reported by the game.
   */
  tags: string[];
  tools: {
    [k: string]: number;
  };
  tooltip: string[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AspectAmount".
 */
export interface AspectAmount {
  amount: string;
  aspect: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Material".
 */
export interface Material {
  /**
   * ARGB color reported by the registry.
   */
  color: number;
  components: Component[];
  /**
   * The source's formula verbatim; not parsed or guessed from an item tooltip.
   */
  formula: string;
  id: string;
  name: string;
  parts: MaterialPart[];
  source: Origin;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Component".
 */
export interface Component {
  /**
   * Relative component count from the material definition, not a recipe quantity.
   */
  amount: string;
  material: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MaterialPart".
 */
export interface MaterialPart {
  /**
   * Material units in one item; null when the source does not define a conversion.
   */
  content?: Ratio | null;
  /**
   * Registry form, e.g. plate, wireGt01, molten. It is not inferred from a name.
   */
  key: string;
  target: Reference;
}
/**
 * An exact amount of base material, independent of item counts or fluid volume.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Ratio".
 */
export interface Ratio {
  denominator: string;
  numerator: string;
}
/**
 * Native tessellated faces. Texture coordinates refer to one source sprite, not its packed atlas.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Model".
 */
export interface Model {
  faces: Face[];
  /**
   * Explicit native suppression (for example the second block of a double chest), never a failed capture.
   */
  hidden: boolean;
  id: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Face".
 */
export interface Face {
  pass: Pass;
  texture: string;
  /**
   * Original winding; triangles repeat their third vertex in the fourth position.
   *
   * @minItems 4
   * @maxItems 4
   */
  vertices: [Vertex, Vertex, Vertex, Vertex];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Vertex".
 */
export interface Vertex {
  /**
   * Finite float32 decimal strings in local east/down/north block coordinates. Strings preserve the game's decimal representation across Java, Rust and JavaScript identities.
   *
   * @minItems 3
   * @maxItems 3
   */
  at: [string, string, string];
  /**
   * Native per-vertex tint and shading, in unsigned RRGGBBAA byte order.
   */
  color: number;
  /**
   * Finite float32 decimal strings in the sprite's [0,1] range, top row at v=0.
   *
   * @minItems 2
   * @maxItems 2
   */
  uv: [string, string];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Mutation".
 */
export interface Mutation {
  /**
   * Unmodified base rate in 0..1; housing, mode, research and world conditions are not applied.
   */
  chance: Chance;
  /**
   * Localized descriptions from the provider, not executable eligibility rules.
   */
  conditions: string[];
  /**
   * The mutation's actual result template can differ from the species' registered default.
   */
  genes: Gene[];
  /**
   * Actual implementation class; conditions remain descriptive, not executable rules.
   */
  handler: string;
  /**
   * Content identity: Forestry exposes no mutation registry key.
   */
  id: string;
  /**
   * Zero-based occurrence among identical registrations, retaining independent attempts.
   */
  occurrence: number;
  /**
   * An unordered pair, canonically sorted. Self-crosses retain two equal entries.
   *
   * @minItems 2
   * @maxItems 2
   */
  parents: [string, string];
  result: string;
  secret: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Chance".
 */
export interface Chance {
  denominator: string;
  numerator: string;
}
/**
 * A populated chromosome of a registered template, not a player's individual genome.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Gene".
 */
export interface Gene {
  allele: string;
  dominant: boolean;
  key: string;
  name: string;
  /**
   * An API-provided scalar or vector. Providers with executable behavior have no invented value.
   */
  value?: PropertyValue | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Recipe".
 */
export interface Recipe {
  category: string;
  /**
   * Ticks, when the source defines a duration.
   */
  duration?: string | null;
  /**
   * Signed EU/t; negative values denote generation.
   */
  energy?: string | null;
  grid?: Grid | null;
  id: string;
  inputs: Input[];
  magic?: MagicRecipe | null;
  order: number;
  outputs: Output[];
  /**
   * Namespaced properties with an explicit value type and unit.
   */
  properties: {
    [k: string]: Property;
  };
  source: Origin;
  view?: string | null;
}
/**
 * Crafting arrangement is recipe data and remains available when no view is captured.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Grid".
 */
export interface Grid {
  /**
   * Row-major item input slot references, with explicit empty cells.
   */
  cells: (number | null)[];
  height: number;
  mirror: boolean;
  width: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Input".
 */
export interface Input {
  choices: Choice[];
  kind: Kind;
  slot: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Choice".
 */
export interface Choice {
  /**
   * Positive signed-64-bit integer: item count or fluid millibuckets.
   */
  amount: string;
  consume: Consumption;
  id: string;
  returns: Remainder[];
  rule: Match;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Remainder".
 */
export interface Remainder {
  amount: string;
  id: string;
  kind: Kind;
}
/**
 * Registered recipe costs before equipment discounts and observed research prerequisites.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MagicRecipe".
 */
export interface MagicRecipe {
  /**
   * Vis for arcane crafting; essentia for crucible and infusion recipes.
   */
  aspects: AspectAmount[];
  /**
   * Item input slot containing the central infusion ingredient.
   */
  central?: number | null;
  /**
   * The configured creative-mode waiver bypasses Vis checks/consumption. An empty arcane cost list is usable only with this waiver, not free in survival.
   */
  creative: boolean;
  /**
   * Base infusion instability, independent of the player's altar and stabilizers.
   */
  instability?: number | null;
  kind: MagicKind;
  /**
   * Optional Salis self-payment. Ordinary arcane output samples assume a separate wand in the workbench's supply slot; this mode requires it empty.
   */
  payment?: Payment | null;
  research: ResearchLink[];
}
/**
 * The consumed input pays the arcane cost using its OLD components and the player's equipment discount. After payment, the new wand retains the remaining charge up to capacity, or zeros these tags. No fixed paid-output sample is implied.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Payment".
 */
export interface Payment {
  capacity: number;
  /**
   * Primal aspect id -> root NBT charge key. Charge/capacity are hundredths of Vis.
   */
  charges: {
    [k: string]: string;
  };
  input: number;
  preserve: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ResearchLink".
 */
export interface ResearchLink {
  completed?: boolean | null;
  /**
   * Null is permitted only for an unregistered @ knowledge flag, as defined by Thaumcraft.
   */
  id?: string | null;
  key: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Output".
 */
export interface Output {
  amount: string;
  chance: Chance;
  /**
   * With a change, id/amount are the first input choice's example, not a fixed result.
   */
  change?: Change | null;
  id: string;
  kind: Kind;
  role: OutputRole;
  slot: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Change".
 */
export interface Change {
  action: Edit;
  input: number;
  /**
   * One native result per input choice, in matching order; excluded from recipe identity.
   */
  samples: Stack[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Stack".
 */
export interface Stack {
  amount: string;
  id: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Property".
 */
export interface Property {
  name: string;
  value: PropertyValue;
}
/**
 * Research prerequisites and observed knowledge, not a claim that the player can currently unlock it.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Research".
 */
export interface Research {
  aspectTriggers: string[];
  aspects: AspectAmount[];
  category: string;
  categoryName: string;
  completed?: boolean | null;
  complexity: number;
  entityTriggers: string[];
  flags: ResearchFlag[];
  hiddenParents: ResearchLink[];
  icon?: string | null;
  id: string;
  itemTriggers: string[];
  name: string;
  parents: ResearchLink[];
  /**
   * @minItems 2
   * @maxItems 2
   */
  position: [number, number];
  siblings: ResearchLink[];
  source: Origin;
  text: string;
  texture?: string | null;
  warp: number;
}
/**
 * Bounded geometry records keep large definitions out of a single source row or HTTP response.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Shape".
 */
export interface Shape {
  cells: Cell[];
  id: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Cell".
 */
export interface Cell {
  /**
   * @minItems 3
   * @maxItems 3
   */
  at: [number, number, number];
  /**
   * Index into the declaring piece's rules or the build's block palette.
   */
  index: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Species".
 */
export interface Species {
  authority: string;
  binomial: string;
  blacklisted: boolean;
  counted: boolean;
  description: string;
  dominant: boolean;
  /**
   * Whether the tree species supports its default template's fruit family.
   */
  fruitCompatible?: boolean | null;
  genes: Gene[];
  humidity: string;
  id: string;
  kind: SpeciesKind;
  members: Member[];
  name: string;
  /**
   * Natural bee activity; distinct from the template's nocturnal tolerance chromosome.
   */
  nocturnal?: boolean | null;
  products: Produce[];
  secret: boolean;
  /**
   * The root UID is the handler; the species allele UID is the key.
   */
  source: Origin;
  specialties: Produce[];
  /**
   * Native climate symbols from Forestry, not guessed biome or numeric ranges.
   */
  temperature: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Member".
 */
export interface Member {
  /**
   * Actual item form, e.g. queen, drone, sapling, pollen.
   */
  form: string;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Produce".
 */
export interface Produce {
  amount: string;
  /**
   * Bee base rate per production cycle. Tree product lists supply no probability: null.
   */
  chance?: Chance | null;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Text".
 */
export interface Text {
  id: string;
  locale: string;
  text: string;
}
/**
 * A StructureLib definition, not a claim that one assembled machine passes its world checks.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Structure".
 */
export interface Structure {
  controller: string;
  description: string[];
  id: string;
  name: string;
  pieces: Piece[];
  /**
   * First requested parameter set, used for definition placement hints.
   */
  probe: Probe;
  /**
   * An explicit missing definition is allowed only in a selection snapshot.
   */
  problem?: string | null;
  source: Origin;
  variants: Variant[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Piece".
 */
export interface Piece {
  anchors: [number, number, number][];
  cells: number;
  chunks: string[];
  name: string;
  rules: Rule[];
  /**
   * Local A/B/C coordinates: right, down, depth. Pieces have no inferred assembly transform.
   *
   * @minItems 3
   * @maxItems 3
   */
  size: [number, number, number];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Rule".
 */
export interface Rule {
  implementation: string;
  kind: RuleKind;
  /**
   * Advisory getBlocksToPlace results, never an exhaustive legality predicate or bill of materials. Null means the provider does not enumerate placement stacks.
   */
  placements?: string[] | null;
  symbol: string;
}
/**
 * Exactly one outcome for a requested construction parameter set.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Variant".
 */
export interface Variant {
  build?: string | null;
  probe: Probe;
  problem?: string | null;
}
/**
 * Shared clipping timeline, independent of the texture or recipe using it.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Track".
 */
export interface Track {
  /**
   * One repeating native UI cycle, compressed by consecutive equal states; never recipe duration.
   */
  frames: Motion[];
  id: string;
}
/**
 * A discrete native UI state. Areas reveal matching fractions of the texture and destination.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Motion".
 */
export interface Motion {
  /**
   * Non-overlapping [left, top, right, bottom] rectangles in [0,1], as finite float32 decimal strings. An empty list hides the element for this state.
   */
  areas: [string, string, string, string][];
  ticks: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "View".
 */
export interface View {
  elements: Element[];
  height: number;
  id: string;
  width: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Environment".
 */
export interface Environment {
  game: string;
  inputs: EnvironmentFile[];
  /**
   * Digests of knowledge states used by adapters; no player names or world locations.
   */
  knowledge: {
    [k: string]: string;
  };
  loader: string;
  locale: string;
  mods: Mod[];
  /**
   * Construction parameters, ordered by count then channel name/value pairs.
   *
   * @minItems 1
   * @maxItems 16
   */
  probes:
    | [Probe]
    | [Probe, Probe]
    | [Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe]
    | [Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe, Probe];
  /**
   * Selected resource packs, in the game's priority order.
   */
  resources: string[];
  settings: {
    [k: string]: string;
  };
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnvironmentFile".
 */
export interface EnvironmentFile {
  path: string;
  sha256: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Mod".
 */
export interface Mod {
  id: string;
  name: string;
  sha256: string;
  version: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Pointer".
 */
export interface Pointer {
  format: "elysium.catalog-pointer";
  id: string;
  revision: 10;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SourceManifest".
 */
export interface SourceManifest {
  environment: string;
  files: SourceFile[];
  format: "elysium.source";
  id: string;
  producer: Producer;
  revision: 10;
  scope: Scope;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SourceFile".
 */
export interface SourceFile {
  bytes: number;
  decodedBytes: number;
  encoding: string;
  kind: string;
  path: string;
  rows: number;
  sha256: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Producer".
 */
export interface Producer {
  name: string;
  version: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Scope".
 */
export interface Scope {
  collections: string[];
  mode: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Entry".
 */
export interface Entry {
  group?: string | null;
  icon?: string | null;
  id: string;
  kind: Kind;
  meta?: number | null;
  name: string;
  order?: number | null;
  registry: string;
  /**
   * Precomputed search text includes registry, display name, tooltip, ore tags and pinyin.
   */
  terms: string;
  tooltip: string[];
}
/**
 * Compact recipe metadata for filtering and counts without decoding full ingredient payloads.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RecipeIndex".
 */
export interface RecipeIndex {
  category: string;
  handler: string;
  id: string;
  owner: string;
  targets: string[];
}
/**
 * Mutation adjacency, so one species page never decodes the entire breeding graph.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Lineage".
 */
export interface Lineage {
  crosses: string[];
  id: string;
  origins: string[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Links".
 */
export interface Links {
  id: string;
  recipes: string[];
  topics: string[];
  uses: string[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Texture".
 */
export interface Texture {
  frames: Sprite[];
  height: number;
  id: string;
  interpolate: boolean;
  width: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Sprite".
 */
export interface Sprite {
  height: number;
  path: string;
  ticks: number;
  width: number;
  x: number;
  y: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Topic".
 */
export interface Topic {
  icon?: Reference | null;
  id: string;
  /**
   * A standalone atlas image when this topic has no item/fluid icon.
   */
  image?: string | null;
  kind: TopicKind;
  name: string;
  order: number;
  terms: string;
}


export declare function assertManifest(value: unknown): asserts value is Manifest;
export declare const formats: { readonly source: { readonly name: string; readonly revision: number }; readonly catalog: { readonly name: string; readonly revision: number } };
export declare const collections: readonly Table['kind'][];
export declare const topicKinds: readonly TopicKind[];
export declare function assertPointer(value: unknown): asserts value is Pointer;
export declare function assertTable(value: unknown): asserts value is Table;
export declare function decodeTable(bytes: Uint8Array, kind?: Table['kind']): Table;
export declare function canonical(value: unknown): string;
