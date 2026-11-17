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
 * A native display provider can intentionally supply geometry before gameplay completion.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BuildCompletion".
 */
export type BuildCompletion = "nativeDisplayBeforeControllerTick" | "nativeDisplayBeforeUnavailableTileTick";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ConstructionFacing".
 */
export type ConstructionFacing = "north" | "south";
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
 * via the `definition` "ProgramData".
 */
export type ProgramData =
  | {
      kind: "beePoolSettings";
      rows: BeePoolSettings[];
    }
  | {
      kind: "beePoolChromosomes";
      rows: string[];
    }
  | {
      kind: "beePoolDefinitions";
      rows: BeePoolDefinition[];
    }
  | {
      kind: "beePoolAlleles";
      rows: BeePoolRegistration[];
    }
  | {
      kind: "beePoolAliases";
      rows: BeePoolAlias[];
    }
  | {
      kind: "beePoolTemplates";
      rows: BeePoolTemplate[];
    }
  | {
      kind: "beePoolDefaults";
      rows: (string | null)[];
    }
  | {
      kind: "beePoolForest";
      rows: (BeePoolPair | null)[];
    }
  | {
      kind: "beePoolForms";
      rows: BeePoolForm[];
    }
  | {
      kind: "beePoolEntries";
      rows: BeePoolSessionEntry[];
    }
  | {
      kind: "beePoolKeys";
      rows: Nbt[];
    }
  | {
      kind: "beePoolCacheEntries";
      rows: BeePoolCacheEntry[];
    }
  | {
      kind: "beePoolSegments";
      rows: BeePoolCacheSegment[];
    }
  | {
      kind: "xuMaterials";
      rows: XuNativeMaterial[];
    }
  | {
      kind: "xuNames";
      rows: FmpNativeName[];
    }
  | {
      kind: "xuAliases";
      rows: FmpNativeAlias[];
    }
  | {
      kind: "xuSettings";
      rows: XuNativeSettings[];
    }
  | {
      kind: "fmpMaterials";
      rows: FmpNativeMaterial[];
    }
  | {
      kind: "fmpNames";
      rows: FmpNativeName[];
    }
  | {
      kind: "fmpAliases";
      rows: FmpNativeAlias[];
    }
  | {
      kind: "fmpSaws";
      rows: Saw[];
    }
  | {
      kind: "fmpSettings";
      rows: FmpNativeSettings[];
    }
  | {
      kind: "forestryFermenter";
      rows: FermenterRecipe[];
    }
  | {
      kind: "forestryFermenterFuels";
      rows: FermenterFuel[];
    }
  | {
      kind: "forestryMoistener";
      rows: MoistenerRecipe[];
    }
  | {
      kind: "forestryMoistenerFuels";
      rows: MoistenerFuel[];
    }
  | {
      kind: "forestryCarpenter";
      rows: CarpenterRecipe[];
    }
  | {
      kind: "fabricatorRecipes";
      rows: FabricatorRecipe[];
    }
  | {
      kind: "fabricatorSmelting";
      rows: FabricatorSmeltingRecipe[];
    }
  | {
      kind: "squeezerRecipes";
      rows: SqueezerRecipe[];
    }
  | {
      kind: "squeezerContainers";
      rows: SqueezerContainer[];
    }
  | {
      kind: "squeezerFluids";
      rows: FilledContainer[];
    }
  | {
      kind: "squeezerCallbacks";
      rows: SqueezerCallback[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolSessionEntry".
 */
export type BeePoolSessionEntry =
  | {
      alchemy: boolean;
      cachedOutput: BeePoolStack;
      conjuration: boolean;
      displayInput: BeePoolStack;
      form: string;
      inputSpecies: string;
      kind: "bee";
      mana: number;
      outputSpecies: string;
      prototype: BeePoolStack;
    }
  | {
      alchemy: boolean;
      conjuration: boolean;
      kind: "item";
      mana: number;
      output: BeePoolStack;
      predicates: BeePoolItemPredicate[];
    }
  | {
      alchemy: boolean;
      coercions?: BeePoolSubtileObservation[];
      conjuration: boolean;
      key: string;
      kind: "mini";
      mana: number;
      output: BeePoolStack;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AliasPolicy".
 */
export type AliasPolicy = "singleHop";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "XuLookupState".
 */
export type XuLookupState = "unused" | "ready";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MissingPolicy".
 */
export type MissingPolicy = "idZero";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SawNbt".
 */
export type SawNbt = "freshWhenDamageable" | "preserve";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CompoundPolicy".
 */
export type CompoundPolicy = "readOnlyIdZero";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ContainerPolicy2".
 */
export type ContainerPolicy2 =
  | {
      kind: "none";
    }
  | {
      kind: "fixed";
      leave: boolean;
      result: ContainerResult;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ModifierPolicy".
 */
export type ModifierPolicy =
  | {
      kind: "constant";
    }
  | {
      kind: "treeSappiness";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FuelKey".
 */
export type FuelKey =
  | {
      kind: "stack";
      stack: NativeStack;
    }
  | {
      kind: "item";
      registry: string;
    }
  | {
      kind: "ore";
      members: NativeStack[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Ingredient".
 */
export type Ingredient = NativeStack | NativeStack[];
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PlanUse".
 */
export type PlanUse =
  | {
      kind: "keep";
      waxPlans: WaxPlan[];
    }
  | {
      kind: "wax";
      maxDamage: number;
      waxPlans: WaxPlan[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ContainerKey".
 */
export type ContainerKey =
  | {
      kind: "stack";
      stack: ForestryStack;
    }
  | {
      kind: "item";
      registry: string;
    }
  | {
      kind: "ore";
      members: ForestryStack[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SqueezerCallback".
 */
export type SqueezerCallback =
  | {
      kind: "unsupported";
      registry: string;
    }
  | {
      kind: "noFluid";
      registry: string;
    }
  | {
      kind: "forge";
      registry: string;
    }
  | {
      kind: "ic2";
      registry: string;
    }
  | {
      compoundTags: string[];
      drainedRegistries: string[];
      kind: "noContainerRule";
      maxMeta: number;
      minMeta: number;
      readEffect: SqueezerReadEffect;
      registry: string;
    }
  | {
      kind: "gtFixed";
      registry: string;
      rows: SqueezerGtFluid[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SqueezerReadEffect".
 */
export type SqueezerReadEffect = "preserve" | "ensureCompound" | "gtppPump";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtExecution".
 */
export type GtExecution = "unresolvedMachineContext" | "referenceOnly";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtFluidConsumption".
 */
export type GtFluidConsumption = "nativeSlots";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtItemConsumption".
 */
export type GtItemConsumption = "nativeMatchingGroups";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtProjection".
 */
export type GtProjection = "nativeSlots" | "specializedProcess";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtSampler".
 */
export type GtSampler = "gtIntegralChancedMultiplier";
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
      chance: Chance;
      kind: "chance";
    }
  | {
      kind: "staged";
    }
  | {
      kind: "upto";
    }
  | {
      kind: "allocated";
    }
  | {
      kind: "reserve";
    }
  | {
      kind: "wear";
    }
  | {
      kind: "stack";
    }
  | {
      kind: "buffer";
    }
  | {
      kind: "pedestals";
    }
  | {
      kind: "damage";
      points: number;
    }
  | {
      cost: number;
      kind: "gtTool";
      owner: string;
      sample: GtToolSample;
    }
  | {
      callback: NativeContainerCallback;
      kind: "nativeContainer";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeContainerCallback".
 */
export type NativeContainerCallback =
  "ic2Cutter" | "botaniaCosmetic" | "botaniaBlackHole" | "thaumicMortar" | "railFirestone";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Match".
 */
export type Match =
  | {
      kind: "nativePoolInput";
    }
  | {
      kind: "matterManipulatorInput";
    }
  | {
      kind: "matterManipulatorUpgrade";
    }
  | {
      kind: "nativeZeroFluid";
      policy: NativeZeroFluidPolicy;
    }
  | {
      kind: "fmpMaterial";
      materialId: number;
      nativeText?: string | null;
      program: string;
    }
  | {
      /**
       * @minItems 1
       * @maxItems 1000
       */
      imprintIds: [number, ...number[]];
      kind: "calInput";
      role: Role2;
    }
  | {
      kind: "bannerInput";
      maxPatterns: number;
    }
  | {
      kind: "exact";
    }
  | {
      kind: "facadeTexture";
    }
  | {
      end: number;
      kind: "ae2CrystalSeed";
      start: number;
    }
  | {
      kind: "ae2EnderDust";
      metadata: number;
      ore: string;
    }
  | {
      kind: "witcheryOvenNbt";
      meta: boolean;
      product: OvenByproduct;
    }
  | {
      kind: "chiselSource";
      meta: boolean;
      nbt: boolean;
    }
  | {
      kind: "chiselTool";
      source: number;
    }
  | {
      kind: "distilleryJar";
    }
  | {
      input: number;
      kind: "creativeCore";
    }
  | {
      kind: "enchanterBook";
    }
  | {
      kind: "lumbermillTank";
    }
  | {
      code: number;
      kind: "nativeMetadataKey";
    }
  | {
      keys: string[];
      kind: "bloodmagic-altar";
      tagged: boolean;
    }
  | {
      kind: "integration";
    }
  | {
      kind: "forestry";
    }
  | {
      kind: "untagged";
      meta: boolean;
    }
  | {
      kind: "buildcraft";
      subtypes: boolean;
      wildcard: boolean;
    }
  | {
      filter: SoulFilter;
      kind: "soul";
    }
  | {
      kind: "ae";
    }
  | {
      kind: "infusion";
      ores: string[];
      template: string;
    }
  | {
      analyzed: boolean;
      kind: "member";
      root: string;
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
      key: string;
      kind: "string_tag";
    }
  | {
      absent: string[];
      kind: "metadata";
      nbt: boolean;
      /**
       * Nonnegative raw metadata, or Railcraft's native -1 sentinel.
       */
      value: number;
    }
  | {
      exclude: boolean;
      kind: "metadataSet";
      nbt: boolean;
      /**
       * @minItems 1
       * @maxItems 65536
       */
      values: [number, ...number[]];
    }
  | {
      callback: NativeItemMetadataCallback;
      kind: "nativeItemMetadata";
      nbt: boolean;
      value: number;
    }
  | {
      absent: string[];
      keys: string[];
      kind: "tags";
      meta: boolean;
      present: string[];
    }
  | {
      keys: string[];
      kind: "without_tags";
    }
  | {
      base: Match;
      exclude: MatchCase[];
      kind: "except";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeZeroFluidPolicy".
 */
export type NativeZeroFluidPolicy = "gregtechCost" | "gregtechPresence" | "vatMatch" | "binnieIncubator";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Role2".
 */
export type Role2 = "machine" | "imprint";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OvenByproduct".
 */
export type OvenByproduct = "none" | "foul" | "exhale" | "hint" | "breath" | "whiff" | "reek" | "odour";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeItemMetadataCallback".
 */
export type NativeItemMetadataCallback = "botaniaManaMirror";
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
      kind: "nativePoolResult";
    }
  | {
      kind: "matterManipulator";
    }
  | {
      kind: "ic2Advanced";
    }
  | {
      kind: "gtCrafting";
    }
  | {
      base: Stack;
      kind: "craftingNbt";
    }
  | {
      kind: "fmpResult";
    }
  | {
      kind: "xuResult";
    }
  | {
      imprint: number;
      kind: "calImprint";
    }
  | {
      color: number;
      kind: "bannerPattern";
      pattern: string;
    }
  | {
      kind: "chiselTool";
      mode: ChiselMode;
      source: number;
    }
  | {
      kind: "gendustryGenome";
      operation: GendustryOperation;
      slots: {
        [k: string]: number;
      };
    }
  | {
      kind: "gendustryGeneSample";
      operation: GendustryOperation;
      slots: {
        [k: string]: number;
      };
    }
  | {
      kind: "gendustryGeneTemplate";
      operation: GendustryOperation;
      slots: {
        [k: string]: number;
      };
    }
  | {
      kind: "gendustryReplication";
      operation: GendustryOperation;
      slots: {
        [k: string]: number;
      };
    }
  | {
      kind: "gendustryMutation";
      operation: GendustryOperation;
      slots: {
        [k: string]: number;
      };
    }
  | {
      kind: "integration";
    }
  | {
      base: string;
      kind: "soul";
    }
  | {
      base: string;
      kind: "floatingFlower";
    }
  | {
      base: string;
      kind: "nativeFloatingFlower";
    }
  | {
      kind: "mapScaling";
    }
  | {
      kind: "runic";
    }
  | {
      kind: "nativeInfusion";
    }
  | {
      base: Stack;
      keys?: string[] | null;
      kind: "nativeInfusionMerge";
      queries?: {
        [k: string]: Invocation;
      } | null;
      tools: boolean;
    }
  | {
      kind: "disguiseInfusion";
    }
  | {
      kind: "analyze";
    }
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
      kind: "salisWand";
      limits: {
        [k: string]: number;
      };
      metadata: number;
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
    }
  | {
      base: Stack;
      config: number;
      /**
       * One native InventoryWithFilterOptions.writeCustomNBT result per config choice, in the same order. Numeric item lookup happens in the pinned game registry; these observations are not a promise to replay arbitrary NBT.
       */
      configurations: {
        [k: string]: Nbt;
      }[];
      kind: "filter";
      metadata?: number | null;
    }
  | {
      kind: "append";
      path: string;
      value: Nbt;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ChiselMode".
 */
export type ChiselMode = "single" | "stack";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryOperation".
 */
export type GendustryOperation =
  "imprint" | "sample" | "transposeSample" | "transposeTemplate" | "mutate" | "replicate" | "extract" | "merge";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Query".
 */
export type Query =
  | {
      eligible: boolean;
      kind: "stable";
    }
  | {
      kind: "spell";
    }
  | {
      kind: "darkSteel";
      spoon: string;
    }
  | {
      capacity: Capacity;
      fallback?: boolean | null;
      kind: "nanotool";
      mode?: number | null;
      nonempty?: boolean | null;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Capacity".
 */
export type Capacity =
  | {
      bits: string;
      kind: "constant";
      owner: string;
    }
  | {
      bits: string;
      kind: "field";
      owner: string;
    }
  | {
      battery?: Battery | null;
      kind: "amun";
    };
/**
 * Correlated amounts, evaluated without arbitrary sampling.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Quantity".
 */
export type Quantity =
  | {
      kind: "spaceAcquisition";
      nominal: string;
    }
  | {
      kind: "lootBag";
    }
  | {
      kind: "bloodAltar";
      nominal: string;
    }
  | {
      kind: "harvest";
      nominal: string;
    }
  | {
      kind: "squeezer";
      nominal: string;
    }
  | {
      kind: "soul";
      nominal: string;
    }
  | {
      kind: "grinding";
      nominal: string;
      threshold: string;
    }
  | {
      kind: "sharedRoll";
      nominal: string;
      threshold: string;
    }
  | {
      kind: "harmony";
      nominal: string;
      outcome: HarmonyOutcome;
    }
  | {
      after: number[];
      input: number;
      kind: "draw";
      limit: string;
    }
  | {
      after: number[];
      input: number;
      kind: "remainder";
    }
  | {
      branch: string;
      condition?: string | null;
      group: string;
      kind: "branch";
      nominal: string;
      parameters?: {
        [k: string]: string;
      } | null;
      threshold?: string | null;
    }
  | {
      condition?: string | null;
      kind: "potential";
      nominal: string;
      parameters?: {
        [k: string]: string;
      } | null;
      sample?: string | null;
      stat: string;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "HarmonyOutcome".
 */
export type HarmonyOutcome = "success" | "failure";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OutputRole2".
 */
export type OutputRole2 = "result" | "return";
/**
 * Native shared process, not independent probabilities on individual outputs. See docs/harmony.md for state, energy, rounding and correlated yield rules.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Process".
 */
export type Process =
  | {
      catalyst: Catalyst;
      display: Display;
      index: number;
      kind: "nativeBotaniaPool";
      mana: number;
      matching: Matching2;
      nativeRecipe: string;
      program: string;
      scope: Scope5;
      settlement: PoolSettlement;
      version: Version;
    }
  | {
      catalyst: Catalyst;
      kind: "botaniaPool";
      mana: number;
      matching: PoolMatching;
      nativeRecipe: string;
      selection?: PoolSelection | null;
      settlement: PoolSettlement;
      version: Version;
    }
  | {
      cooldown: number;
      earlier: Selector[][][];
      earlierHeads?: number[];
      head?: Head | null;
      kind: "botaniaRunic";
      livingrock: string;
      mana: number;
      matching: RunicMatching;
      nameTagDefault?: string | null;
      nativeRecipe: string;
      predicates: Selector[][];
      rune: string;
      scope: RunicScope;
      settlement: RunicSettlement;
      version: Version;
    }
  | {
      brew?: string | null;
      container?: number | null;
      inputSlots: (number | null)[];
      kind: "botaniaWorld";
      machine: BotaniaWorldMachine;
      mana?: number | null;
      predicates: BotaniaWorldSelector[][];
      priority: BotaniaWorldSelector[][][];
      seed?: string | null;
      selector: number;
      speciesDominance: {
        [k: string]: boolean;
      };
      version: string;
      worldResult?: BotaniaWorldBlock | null;
    }
  | {
      consumption: Consumption2;
      height: number;
      kind: "aeCrafting";
      matching: Matching3;
      mirror: boolean;
      nativeRecipe: string;
      predicates: (Selector2[] | null)[];
      shaped: boolean;
      version: Version2;
      width: number;
    }
  | {
      boosters: string[];
      flag: string;
      kind: "wirelessTerminalUpgrade";
      nativeRecipe: string;
      /**
       * Registered instances of the native terminal/booster classes.
       */
      terminals: string[];
      version: Version3;
      wirelessVersion: WirelessVersion;
    }
  | {
      ignoredOutputs: number;
      kind: "projectRedCrafting";
      minimumSlots: number;
      predicates: (ProjectRedPredicate | null)[];
      shaped: boolean;
      version: string;
    }
  | {
      amount: number;
      bufferCapacity: number;
      /**
       * @minItems 4
       * @maxItems 4
       */
      color: [number, number, number, number];
      input: number;
      kind: "bloodCalcinator";
      lpPerTick: number;
      mainCapacity: number;
      name: string;
      orb: number;
      reagent: string;
      transferPerTick: number;
      version: string;
      workTicks: number;
    }
  | {
      acceleratedStep: number;
      containers: (InventoryItemPrototype | null)[][];
      earlier: BloodAlchemyPriority[];
      kind: "bloodAlchemy";
      lpPerProgress: number;
      minimumOrb: number;
      orb: number;
      orbs: BloodAlchemyOrb[];
      result: InventoryItemPrototype;
      /**
       * @minItems 5
       * @maxItems 5
       */
      templates: [
        InventoryItemSelector | null,
        InventoryItemSelector | null,
        InventoryItemSelector | null,
        InventoryItemSelector | null,
        InventoryItemSelector | null
      ];
      version: string;
      workProgress: number;
    }
  | {
      height: number;
      kind: "bloodOrbCrafting";
      mirror: boolean;
      orbs: BloodAlchemyOrb[];
      predicates: (BloodOrbCraftingPredicate | null)[];
      shaped: boolean;
      version: string;
      width: number;
    }
  | {
      baseInput: number;
      baseSlot: number;
      definitions: ImbuingDefinition[];
      kind: "randomThingsImbuing";
      outputSlot: number;
      reagentSlots: number;
      result: InventoryItemPrototype;
      selected: number;
      thresholdBits: string;
      version: string;
    }
  | {
      definitions: SolderingDefinition[];
      heat: number;
      kind: "logisticsSoldering";
      powerDisabled: boolean;
      result: InventoryItemPrototype;
      resultTag: SolderingResultTag;
      selected: number;
      solderPredicate: SolderingSolderPredicate;
      solderSlot: number;
      version: string;
      workProgress: number;
    }
  | {
      expression: string;
      ingredient: number;
      inputMetadata: number;
      kind: "nativeBrewing";
      lingering: boolean;
      maximumPotions: number;
      outputMetadata: number;
      potion: number;
      preserveNbt: boolean;
      resultRegistry: string;
      version: string;
      workTicks: number;
    }
  | {
      baseCost: number;
      bookPolicy: BookPolicy;
      cost: number;
      costPerLevel: number;
      enchantment: number;
      itemsPerLevel: number;
      kind: "enderioEnchanter";
      level: number;
      materialPolicy: MaterialPolicy;
      maxLevel: number;
      offeredMaximum: string;
      offeredMinimum: string;
      pickupPolicy: PickupPolicy;
      version: Version4;
    }
  | {
      consumption: Debit;
      height?: number | null;
      kind: "creativeCoreCrafting";
      matching: Matching4;
      predicates: (CreativeCorePredicate | null)[];
      shaped: boolean;
      version: Version5;
      width: number;
    }
  | {
      charges: string[][];
      kind: "ic2Advanced";
      layout: Ic2AdvancedLayout;
      nativeIndex: number;
      prototype: Stack;
      scope: Ic2AdvancedScope;
      states: GtCraftingWitness[];
      version: Ic2AdvancedVersion;
      visibility: Ic2AdvancedVisibility;
    }
  | {
      base: Stack;
      kind: "gtCrafting";
      prototype: Stack;
      rules: GtCraftingRules;
      scope: GtCraftingScope;
      /**
       * One actual native result witness per distinct rule-relevant result state. Index-aligned with output.change.samples, never a Cartesian tuple table.
       */
      states: GtCraftingWitness[];
      /**
       * Native simulated discharge for each concrete offered count-one specimen.
       */
      transfers: number[][];
      version: GtCraftingVersion;
    }
  | {
      kind: "craftingNbt";
    }
  | {
      craftingListed: boolean;
      kind: "ic2CropPrediction";
      parents: CropKey[];
      points: number;
      result: CropKey;
      total: number;
      usageListedFor: CropKey[];
    }
  | {
      baseAmount: string;
      batchLimit?: number | null;
      circuitItemRole?: string | null;
      durationLaw: string;
      energyPerParallel?: string | null;
      environmentLaw: string;
      fluid: string;
      kind: "spaceAcquisition";
      modules: Module[];
      operation: Operation2;
      outputLaw: string;
      pipeItemRole?: string | null;
      pipeLaw?: string | null;
      planet: string;
      powerLaw: string;
      selectionLaw: string;
      selector: number;
      toolDebit: string;
    }
  | {
      imprint: number;
      /**
       * @minItems 1
       * @maxItems 1000
       */
      imprintIds: [number, ...number[]];
      kind: "calImprint";
      machine: number;
      nativeRecipe: string;
      occupiedLimit: number;
      tag: string;
    }
  | {
      cells: (XuNativeCell | null)[];
      kind: "xuMicroblock";
      layout: (number | null)[];
      materialId?: number | null;
      materialTexts: (string | null)[];
      nativeIndex: number;
      nativeRecipe: string;
      program: string;
      prototype: Stack;
      scope: XuNativeScope;
      states: XuNativeState[];
    }
  | {
      branch: Branch;
      kind: "fmpMicroblock";
      layout: (number | null)[];
      materialId: number;
      program: string;
    }
  | {
      allowFortune: boolean;
      bag: string;
      budgetLaw: string;
      buffered: CachedGroup[];
      debitLaw: string;
      emptyDraw: string;
      group: number;
      groups: Group[];
      kind: "lootBagOpening";
      limitLaw: string;
      nativeItem: string;
      quantityLaw: string;
      retryLimit: number;
      retryState: string;
      shuffleLaw: string;
    }
  | {
      banner: number;
      color: number;
      colorOres: ColorOre[];
      dyes: number[];
      kind: "bannerPattern";
      layout: (number | null)[];
      maxPatterns: number;
      mode: BannerMode;
      nativeRecipe: string;
      palette: Dye[];
      pattern: number;
      rules: PatternRule[];
    }
  | {
      display: string;
      healthDecrement: string;
      inventorySize: number;
      kind: "extraUtilsSoul";
      minimumHealth: string;
      modifierUuid: string;
      nativeLifecycle: string;
      nativeRecipe: string;
      pending: string;
      referenceHealth: string;
      sword: string;
    }
  | {
      batchSteps: number;
      displayedTicks: number;
      freshBatches: string;
      input: string;
      inventoryLimit: number;
      kind: "decayChest";
      maxTicks: number;
      nativeIndex: number;
      nativeItem: string;
      nativeRecipe: string;
      nativeTile: string;
      output: string;
      replacement: DecayChestReplacement;
      scheduler: DecayChestScheduler;
      scope: DecayChestScope;
      stateLaw?: DecayChestStateLaw | null;
      version: string;
    }
  | {
      energyPerOperation: number;
      energyPolicy: SolarEnergyPolicy;
      input: string;
      inputAmount: string;
      inputDebit: SolarInputDebit;
      kind: "molecularTransformer";
      nativeIndex: number;
      nativeRecord: string;
      nativeTile: string;
      output: string;
      outputAmount: string;
      outputPolicy: SolarOutputPolicy;
      priority: SolarPriority;
      version: string;
    }
  | {
      basicItem?: string | null;
      capacity: CircuitCapacity;
      directCall: CircuitDirectCall;
      hard: boolean;
      inputDebit: CircuitDebit;
      kind: "circuitFabricator";
      matcher: CircuitMatcher;
      maxExtractGj: number;
      nativeIndex: number;
      nativeRegistry: string;
      nativeTile: string;
      output: string;
      outputAmount: string;
      outputPolicy: CircuitOutput;
      overflow: CircuitOverflow;
      priority: CircuitPriority;
      processingTicks: number;
      quick: boolean;
      rawOutputAmount: string;
      /**
       * Native inventory slots 1..5. A null is required empty, not an unchecked cell.
       *
       * @minItems 5
       * @maxItems 5
       */
      slots: [string | null, string | null, string | null, string | null, string | null];
      version: string;
      work: CircuitWork;
    }
  | {
      admission: RefineryAdmission;
      containerPolicy: RefineryContainers;
      countdownTicks: number;
      freshStartupTicks: number;
      hard: boolean;
      input: string;
      kind: "galacticraftRefinery";
      legacyFuel: boolean;
      maxExtractGj: number;
      nativeTile: string;
      normalizedOil: string;
      output: string;
      outputPolicy: RefineryOutputPolicy;
      scope: RefineryScope;
      tankCapacity: number;
      transferAmount: string;
      version: string;
      work: RefineryWork;
    }
  | {
      cookTicks: number;
      debit: DistilleryDebit;
      earlier: DistilleryAdmission[];
      jar: string;
      jarPolicy: DistilleryJarPolicy;
      jars: number;
      kind: "witcheryDistillery";
      matching: DistilleryMatching;
      nativeCount: number;
      nativeIndex: number;
      nativeTile: string;
      outputPolicy: DistilleryOutputPolicy;
      /**
       * @minItems 4
       * @maxItems 4
       */
      outputs: [
        DistilleryPrototype | null,
        DistilleryPrototype | null,
        DistilleryPrototype | null,
        DistilleryPrototype | null
      ];
      powerPerWorkTickBits: string;
      /**
       * @minItems 2
       * @maxItems 2
       */
      reagents: [DistilleryReagent | null, DistilleryReagent | null];
      workPolicy: DistilleryWorkPolicy;
    }
  | {
      action: AeWorldAction;
      kind: "ae2WorldAcquisition";
      material: AeWorldMaterial;
      /**
       * Sealed role-to-definition map copied from the native callback. This keeps material roles bound to concrete identities across compilation.
       */
      nativeDefinitions: {
        [k: string]: NativeIdentity;
      };
      result: string;
      resultIdentity: NativeIdentity;
      sourceIdentities: NativeIdentity[][];
      sources: string[][];
    }
  | {
      kind: "ae2Facade";
      nativeRecipe: string;
      outputCount: number;
      texture: string;
      /**
       * Native item-to-block metadata normalization may change the displayed texture metadata.
       */
      textureMetadata?: number | null;
    }
  | {
      baseChance: number;
      baseTicks: number;
      byproduct: OvenByproduct;
      byproductItem?: string | null;
      earlier: OvenPrior[];
      filteredDouble: boolean;
      filteredFunnelChance: number;
      jarMode: JarMode;
      jarThreshold: number;
      kind: "witcheryOven";
      maximumFunnels: number;
      nbtClassified: boolean;
      normalFunnelChance: number;
      selector: number;
      ticksPerFunnel: number;
    }
  | {
      byproduct: OvenByproduct;
      byproductItem?: string | null;
      definitions: NativeOvenDefinition[];
      filteredDouble: boolean;
      furnace: NativeOvenSmelting[];
      inputName?: string | null;
      jarMode: JarMode;
      jarStock: number;
      kind: "witcheryNativeOven";
      roles: {
        [k: string]: string;
      };
      selected: number;
      version: string;
      witcherySapling: string;
    }
  | {
      baseOutput: number;
      bottlesPerItem: number;
      dimension: number;
      familiarType: number;
      hatBonus: number;
      kind: "witcheryKettle";
      power: number;
      selector: number;
      waterMillibuckets: number;
    }
  | {
      actionIndex: number;
      /**
       * Earlier original recipes in this action, after expansion to item/meta keys.
       */
      earlier: string[][];
      expandedIndex: number;
      /**
       * Current action-list item/meta keys. Order is immaterial, multiplicity matters.
       */
      keys: string[];
      kind: "witcheryRitual";
      power: number;
      waterMillibuckets: number;
    }
  | {
      actionIndex: number;
      capacity: number;
      dispatch: NativeRitualDispatch[];
      kind: "witcheryNativeRitual";
      names: {
        [k: string]: string;
      };
      patterns: NativeRitualActionKey[][];
      scope: string;
      selected: number;
      trigger: NativeRitualActionKey;
      triggerPower: number;
      triggerTarget: number;
      version: string;
      waterMillibuckets: number;
    }
  | {
      baseTicks: number;
      bonus: boolean;
      earlier: InfernalPrior[];
      input: InfernalInput;
      inputCount: number;
      kind: "witcheryInfernal";
      maximumBellows: number;
      noBellowsChance: number;
      oreKey?: string | null;
      perBellowsChance: number;
      selector: number;
      specialFuel: boolean;
      ticksPerBellows: number;
      visDivisor: number;
    }
  | {
      canChisel: CanChisel;
      damagePoints: number;
      damagePolicy: DamagePolicy;
      kind: "chisel";
      mode: ChiselMode;
      onChisel: OnChisel;
      selection: Selection3;
      sourceNbt: SourceNbt;
      sourceSlot: number;
      sources: SourceSelection[];
      target: string;
      targetCount: number;
      targetTag: TargetTag;
      toolSlot: number;
      tools: Tool[];
    }
  | {
      draws: number;
      kind: "harvestDrops";
      mode: HarvestMode;
      /**
       * Native registry order; zero-weight candidates cannot be selected.
       */
      weights: number[];
    }
  | {
      canBeFilled: boolean;
      consumptionRate: number;
      drainRate: number;
      earlier: PriorPredicate[];
      inputCount: WholeStack;
      kind: "bloodmagic-altar";
      lp: number;
      nativeIndex: number;
      nbt: AltarNbt;
      nbtKeys: string[];
      numeric: JavaNumbers;
      orb?: AltarOrb | null;
      recheckResultOnCompletion: boolean;
      result: AltarResult;
      selection: RegistrySelection;
      serverWorld: boolean;
      startRequiresPositiveTank: boolean;
      tier: number;
      upgrades: AltarUpgrades;
    }
  | {
      activationCost: number;
      activationGate: RitualActivation;
      activeRitual: boolean;
      components: RitualComponent[];
      cooldown: number;
      cooldownRule: BindingCooldown;
      crystalLevel: number;
      earlier: PriorPredicate[];
      eventGate: boolean;
      /**
       * @minItems 6
       * @maxItems 6
       */
      itemBounds: [number, number, number, number, number, number];
      kind: "bloodmagic-binding";
      nativeIndex: number;
      networkGate: BindingNetworkGate;
      ownerNetwork: boolean;
      refreshCost: number;
      refreshDebit: BindingDebit;
      result: BindingResult;
      ritual: RitualKey;
      selection: RegistrySelection;
      /**
       * @minItems 3
       * @maxItems 3
       */
      spawnOffset: [number, number, number];
      state: BindingState;
      structureRule: RitualStructure;
    }
  | {
      activationCost: number;
      activationGate: RitualActivation;
      activeRitual: boolean;
      components: RitualComponent[];
      cooldown: number;
      cost: number;
      crystalLevel: number;
      deactivateAfterSummon: boolean;
      destroyBlocks: boolean;
      earlier: PriorPredicate[];
      eventGate: boolean;
      fallback: MeteorComponent;
      filler: MeteorComponent[];
      fillerChance: number;
      generation: MeteorGeneration;
      /**
       * @minItems 6
       * @maxItems 6
       */
      itemBounds: [number, number, number, number, number, number];
      kind: "bloodmagic-meteor";
      nativeIndex: number;
      ores: MeteorComponent[];
      ownerNetwork: boolean;
      radius: number;
      reagentCost: number;
      reagentIdentities: ReagentIdentity[];
      reagents: MeteorModifier[];
      refreshCost: number;
      result: MeteorResult;
      ritual: RitualKey;
      selection: RegistrySelection;
      spawnY: number;
      structureRule: RitualStructure;
    }
  | {
      kind: "forestryFabricator";
      operation: FabricatorOperation;
      program: string;
      selector: number;
    }
  | {
      fuel?: number | null;
      kind: "forestryProcessing";
      machine: ProcessingMachine;
      program: string;
      selector: number;
      variation?: number | null;
    }
  | {
      callback?: BottleCallback | null;
      fluid: string;
      kind: "forestryBottler";
      requirement: NativeFluid;
      result: NativeStack;
      retainedInput: NativeStack;
    }
  | {
      firstOreName: string;
      fluidDrainLimit: number;
      fluidGate: number;
      inputPolicy: InputPolicy;
      kind: "binnieLumbermill";
      nativeCount: number;
      nativeIndex: number;
      outputPolicy: NativeOutputPolicy;
      queuedTicks: number;
      rfPerWorkTick: number;
      startTicks: number;
      tankPolicy: TankPolicy;
      version: NativeVersion;
      workPolicy: WorkPolicy;
      workTicks: number;
    }
  | ((
      | {
          alteration: AcclimatiserAlteration;
          attemptInterval: number;
          chromosome: number;
          consumption: AcclimatiserConsumption;
          domain: Domain2;
          effect: number;
          humidity: Effect[];
          inputGate: ToleranceGate;
          kind?: "binnie_genetics";
          machine: "acclimatiser";
          rfPerTick: number;
          selection: AcclimatiserSelection;
          systemOrdinal: number;
          systems: ToleranceSystem[];
          temperature: Effect[];
          toleranceType: ToleranceKind;
        }
      | {
          baseTicks: number;
          discovery: Discovery;
          domain: Domain2;
          dyeChargePerCycle: number;
          grade: number;
          kind?: "binnie_genetics";
          machine: "sequencer";
          outputGate: SequenceOutputGate;
          ownerRequired: boolean;
          progressEncoding: SequenceEncoding;
          progressWrite: SequenceProgress;
          resume: SequenceResume;
          rfPerTick: number;
          strength: SequenceStrength;
        }
      | {
          baseRf: number;
          baseTicks: number;
          conversion: Conversion;
          domain: Domain2;
          dyePerTick: number;
          inputGate: AnalysisGate;
          kind?: "binnie_genetics";
          machine: "analyser";
          transformation: AnalysisTransform;
        }
      | {
          alleleClassGate: boolean;
          bacteria?: number | null;
          baseRf: number;
          baseTicks: number;
          beeChromosomeOffset: BeeOffset;
          chargesGate: ChargesGate;
          defunctGate: DefunctGate;
          domain: Domain2;
          drain?: Drain | null;
          inputGate: ManipulationGate;
          kind?: "binnie_genetics";
          machine: "splicer";
          mismatchComparison: MismatchComparison;
          rootGate: SerumRootGate;
          serum: SerumKind;
          serumDamage: number;
          tankGate?: number | null;
          timeScale: TimeScale;
          transformation: GeneTransform;
        }
      | {
          baseRf: number;
          baseTicks: number;
          chromosomeSelection: ChromosomeSelection;
          consumptionThreshold: number;
          domain: Domain2;
          enzymeCharge: number;
          ethanol: number;
          homologSelection: HomologSelection;
          invalidGenome: InvalidGenome;
          kind?: "binnie_genetics";
          machine: "isolator";
          outputGate: IsolationOutputGate;
        }
      | {
          alleleClassGate: boolean;
          bacteria?: number | null;
          baseRf: number;
          baseTicks: number;
          beeChromosomeOffset: BeeOffset;
          chargesGate: ChargesGate;
          defunctGate: DefunctGate;
          domain: Domain2;
          drain?: Drain | null;
          inputGate: ManipulationGate;
          kind?: "binnie_genetics";
          machine: "inoculator";
          mismatchComparison: MismatchComparison;
          rootGate: SerumRootGate;
          serum: SerumKind;
          serumDamage: number;
          tankGate?: number | null;
          timeScale: TimeScale;
          transformation: GeneTransform;
        }
      | {
          baseRf: number;
          baseTicks: number;
          deleteWholeInputStack: boolean;
          domain: Domain2;
          drain: Drain;
          enzymeCharge: number;
          ethanolPerDna: number;
          kind?: "binnie_genetics";
          machine: "genepool";
          state: PersistentState;
          tankGate: number;
          yield: DnaYield;
        }
      | {
          attemptInterval: number;
          earlier: IncubatorRecipe[];
          fluidGate: IncubatorFluidGate;
          kind?: "binnie_genetics";
          machine: "incubator";
          nativeIndex: number;
          outputGate: IncubatorOutputGate;
          recipe: IncubatorRecipe;
          rfPerTick: number;
          selection: IncubatorSelection;
        }
      | {
          bacteriaPerGene: number;
          baseRf: number;
          baseTicks: number;
          chargeChange: number;
          chargeGate: ChargesGate;
          dnaPerGene: number;
          domain: Domain2;
          drain: Drain;
          goldCharge: number;
          goldOptional: boolean;
          goldTimeScale: number;
          kind?: "binnie_genetics";
          machine: "polymeriser";
          state: PersistentState;
          tankGate: number;
          timeScale: PolymeriserTimeScale;
        }
    ) & {
      kind: "binnie_genetics";
    })
  | {
      config: GendustryConfig;
      kind: "gendustryGenetics";
      labwareThreshold?: number | null;
      operation: GendustryOperation;
      outputPolicy: GendustryOutputPolicy;
      rngBound: number;
      roots: GendustryRoot[];
      selector: number;
      slots: {
        [k: string]: number;
      };
      sources?: GendustryDnaSource[] | null;
    }
  | {
      kind: "gendustryTemplate";
      maximumSamples: number;
      minimumSamples: number;
      operation: GendustryOperation;
      roots: GendustryRoot[];
      samples: number[];
      scanOrder: string;
      template: number;
    }
  | {
      config: GendustryFluidConfig;
      fluid: string;
      input: number;
      kind: "gendustryFluid";
      machine: GendustryFluidMachine;
      sources: GendustryDnaSource[];
      version: string;
    }
  | {
      chance: Chance;
      kind: "forestrySqueezer";
      program: string;
      selector: SqueezerSelector;
      time: number;
    }
  | {
      kind: "buildcraftIntegration";
      rule: IntegrationRule;
    }
  | {
      /**
       * Per input slot and choice: container left in that input slot when consuming a single item. Stacks >1 with a container are not consumed. Air choice returns still describe the separate air-output slot.
       *
       * @minItems 2
       * @maxItems 2
       */
      containers: [(Stack | null)[], (Stack | null)[]];
      heat: number;
      kind: "ic2Blast";
    }
  | {
      kind: "unstableCasting";
    }
  | {
      capacity: number;
      delay: string;
      earlier: Stack[][];
      energy: number;
      /**
       * @minItems 2
       * @maxItems 2
       */
      filling: [string[], string[]];
      kind: "buildcraftRefinery";
    }
  | {
      energy: number;
      kind: "buildcraftAssembly";
    }
  | {
      fuel: RailBlastFuelSupply;
      kind: "railcraftBlast";
      registeredTime: number;
    }
  | {
      earlier: CraftingSelector[];
      kind: "extremeCrafting";
      nativeIndex: number;
      /**
       * Complete intersection of item types admitted by both occupied cells. The Item.isRepairable branch is audited by the native producer. Maximum is the signed native no-argument field, never a guessed tool quantity.
       */
      repair: Repair[];
    }
  | {
      copyNbt?: boolean;
      earlier: CraftingSelector[];
      kind: "rolling";
      powered: boolean;
    }
  | {
      copyNbt?: boolean;
      earlier: CraftingSelector[];
      enderFlux: string;
      kind: "qed";
    }
  | {
      earlier: CraftingSelector[];
      kind: "galaxyspace-assembly";
      repair: Repair[];
    }
  | {
      kind: "floatingFlowers";
      special: number[];
    }
  | {
      coercions: Coercion2[];
      kind: "nativeFloatingFlowers";
      special: number[];
      version: string;
    }
  | {
      /**
       * Native aspect tag -> stable aspect identity for interpreting essence NBT.
       */
      aspects: {
        [k: string]: string;
      };
      count: number;
      inkNames: string[];
      kind: "arcaneSpecial";
      nativeRecipe: string;
      operation: Operation3;
      /**
       * Native item roles, not a complete enumeration of admissible NBT states.
       */
      roles: {
        [k: string]: string;
      };
    }
  | {
      baseAspects: AspectAmount[];
      baseInstability: number;
      declaredInput: Template;
      effect: Effect2;
      kind: "nativeInfusion";
      nativeRecipe: string;
      pattern: LegacyPattern;
      pedestalLimit: number;
      pedestals: PedestalReplacement[];
      randomAspects?: RandomAspects | null;
      /**
       * Loaded native class contains the GTNH replacement Mixin state/helper.
       */
      replacementsActive: boolean;
    }
  | {
      components: Comparison2[];
      kind: "disguiseInfusion";
      mode: OreMode;
      nativeRecipe: string;
      pedestalLimit: number;
      states: State[];
    }
  | {
      jars: InventoryItemSelector[];
      kind: "stickyJar";
      nativeRecipe: string;
    }
  | {
      altarPowerBits: string;
      definitions: Definition[];
      kind: "witcherySpinning";
      selected: number;
      version: string;
      workTicks: number;
    }
  | {
      definitions: Definition2[];
      kind: "witchingSpinning";
      selected: number;
      version: string;
    }
  | {
      capacity: number;
      drains: boolean;
      earlier: SoulSelector[];
      energy: number;
      experience: number;
      kind: "soul";
      levels: number;
      spawner: boolean;
    }
  | {
      balls: GrindingBall[];
      blocked: MatchCase[];
      bonus: boolean;
      earlier: GrindingRequirement[];
      energy: number;
      kind: "sag";
      oreBlocked: MatchCase[];
      slot: number;
    }
  | {
      energy: number;
      kind: "alloy";
      slots: number[];
    }
  | {
      energy: number;
      kind: "splice";
      slots: number[];
    }
  | {
      energy: number;
      extra: VatConsumption[];
      kind: "vat";
      zeroOutput?: string | null;
    }
  | {
      cost: number;
      itemsPerLevel: number;
      kind: "enchanter";
      level: number;
      maxLevel: number;
    }
  | {
      bottom?: string | null;
      kind: "inscriber";
      mode: InscriberMode;
      namePress?: string | null;
      top?: string | null;
    }
  | {
      kind: "mapScaling";
    }
  | {
      charge: number;
      kind: "runic";
    }
  | {
      /**
       * Observed final charge of the supplied center, never a constant for its whole item class.
       */
      charge: number;
      kind: "nativeRunic";
      nativeRecipe: string;
      pattern: LegacyPattern;
      pedestalLimit: number;
    }
  | {
      chargeCapacity: number;
      cost: number;
      enforceStaff: boolean;
      kind: "salisReplacement";
      material?: WandMaterial | null;
      nativeRecipe: string;
      part: Part;
      paymentCondition: PaymentCondition;
      preserve: boolean;
      resultCapacity: number;
      roles: Role3[];
      sceptre: boolean;
      selfPayment: boolean;
      staff: boolean;
      staffAttributes?: Nbt | null;
      targetStaff: boolean;
    }
  | {
      allowed: Upgrade[];
      codec: Codec;
      defaultUpgrade?: Upgrade | null;
      kind: "matterManipulatorUpgrade";
      manipulator: string;
      mappings: Mapping[];
      metaItems: string[];
      nativeRecipe: string;
      states: State2[];
      tier: number;
      upgrade: Upgrade;
      upgrades: Upgrade[];
      version: string;
    }
  | {
      /**
       * Round-trip decimal representation of the native binary64 base chance.
       */
      chance: string;
      compressionTier: number;
      helium: string;
      hydrogen: string;
      kind: "harmony";
      mode: HarmonyMode;
      outputEu: string;
      rocketTier: number;
      startEu: string;
      ticks: string;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Catalyst".
 */
export type Catalyst = "none" | "alchemy" | "conjuration";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Display".
 */
export type Display = "constructorPrototype";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Matching2".
 */
export type Matching2 = "callbacksBeforeCatalystFirstEligible";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Scope5".
 */
export type Scope5 = "serialOwnedSnapshot";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PoolSettlement".
 */
export type PoolSettlement = "serverDroppedItemUnitAndSignedManaUpdate";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Version".
 */
export type Version = "1.12.28-GTNH";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PoolMatching".
 */
export type PoolMatching = "firstEligibleNativeRegistryEntry";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PoolPredicate".
 */
export type PoolPredicate =
  | {
      kind: "item";
      metadata: number;
      registry: string;
    }
  | {
      key: string;
      kind: "subTileKey";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "HeadMatching".
 */
export type HeadMatching = "packedNameTagsEachRejectLocalizedDefault";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "HeadOutput".
 */
export type HeadOutput = "freshPlayerSkullFromLastNameTagOrInitialName";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "HeadVisibility".
 */
export type HeadVisibility = "craftingNbtOutputQueryOnly";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RunicMatching".
 */
export type RunicMatching = "physicalFirstRemainingPredicate";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RunicScope".
 */
export type RunicScope = "stablePackedInputsWithoutCachedOverride";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RunicSettlement".
 */
export type RunicSettlement = "clearAltarRefundRunesAndConsumeDroppedLivingrock";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BotaniaWorldMachine".
 */
export type BotaniaWorldMachine = "daisy" | "petal" | "trade" | "brewery";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Consumption2".
 */
export type Consumption2 = "workbenchOnePerOccupiedCellThenNativeContainer";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Matching3".
 */
export type Matching3 = "columnOffsetsUnmirroredThenOptionalMirror" | "rowMajorFirstRemainingPredicate";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Version2".
 */
export type Version2 = "rv3-beta-695-GTNH" | "1.12.7";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Version3".
 */
export type Version3 = "1.4.120-gtnh";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "WirelessVersion".
 */
export type WirelessVersion = "1.12.7";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProjectRedPredicateKind".
 */
export type ProjectRedPredicateKind = "item" | "ore" | "micro";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SolderingResultTag".
 */
export type SolderingResultTag = "prototype" | "freshRandomUuid";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SolderingSolderPredicate".
 */
export type SolderingSolderPredicate = "anyItem";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BookPolicy".
 */
export type BookPolicy = "runtimePresenceClearWholeSlot";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MaterialPolicy".
 */
export type MaterialPolicy = "nativeCountDeterminesLevelDebitDivisorTimesLevelNoContainers";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PickupPolicy".
 */
export type PickupPolicy = "creativeBypassXpElseSignedLevelDebit";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Version4".
 */
export type Version4 = "2.9.28";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Debit".
 */
export type Debit = "workbenchOnePerOccupiedCellThenNativeContainer";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Matching4".
 */
export type Matching4 = "columnOffsetsUnmirroredThenMirror" | "rowMajorFirstMatchRemoveFirstEqual";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CreativeCorePredicate".
 */
export type CreativeCorePredicate =
  | {
      kind: "item";
      minimum: number;
      registry: string;
    }
  | {
      kind: "itemstack";
      metadata: number;
      minimum: number;
      nbt?: Nbt | null;
      needNBT: boolean;
      registry: string;
    }
  | {
      kind: "ore";
      minimum: number;
      name: string;
      selectors: Selector3[];
    }
  | {
      identity: string[];
      kind: "block";
      members: string[];
      minimum: number;
    }
  | {
      identity: string[];
      kind: "material";
      members: string[];
      minimum: number;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Version5".
 */
export type Version5 = "1.5.14-GTNH";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Ic2AdvancedScope".
 */
export type Ic2AdvancedScope = "exportedCountOneCandidates";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Ic2AdvancedVersion".
 */
export type Ic2AdvancedVersion = "2.2.828-experimental";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtCraftingScope".
 */
export type GtCraftingScope = "exportedCandidates";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtCraftingVersion".
 */
export type GtCraftingVersion = "5.09.51.482";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Operation2".
 */
export type Operation2 = "pump" | "siphon";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "XuNativeCell".
 */
export type XuNativeCell =
  | number
  | {
      stack: XuNativeLiteralStack;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "XuNativeScope".
 */
export type XuNativeScope = "nativeRegistryWitnesses";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Branch".
 */
export type Branch = "hollow" | "gluing" | "thinning" | "splitting" | "hollowFill";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BannerMode".
 */
export type BannerMode = "mask" | "catalyst" | "catalystNoDye";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DecayChestReplacement".
 */
export type DecayChestReplacement = "wholeStoredStackToOne";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DecayChestScheduler".
 */
export type DecayChestScheduler = "loadedServerInt32Modulo20";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DecayChestScope".
 */
export type DecayChestScope = "freshUntagged" | "nativeStateLaw";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DecayChestStateLaw".
 */
export type DecayChestStateLaw = "tickableItemBatchV1";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SolarEnergyPolicy".
 */
export type SolarEnergyPolicy = "totalEuBufferedAndOutputSpace";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SolarInputDebit".
 */
export type SolarInputDebit = "onWorkStartWithoutContainers";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SolarOutputPolicy".
 */
export type SolarOutputPolicy = "emptyCopiesPrototypeMergeKeepsExistingTags";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SolarPriority".
 */
export type SolarPriority = "firstItemMetadataNoFallback";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitCapacity".
 */
export type CircuitCapacity = "emptyOrOriginalCountSignedIntWithinLimits";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitDirectCall".
 */
export type CircuitDirectCall = "debitsEvenIfOutputRejected";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitDebit".
 */
export type CircuitDebit = "onCompletionOneWithoutContainers";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitMatcher".
 */
export type CircuitMatcher = "itemLiteralMetadataIgnoreCountAndTags";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitOutput".
 */
export type CircuitOutput = "emptyCopiesPrototypeMergeKeepsExistingTags";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitOverflow".
 */
export type CircuitOverflow = "mergeAbove64SpawnsUntaggedUnitsIgnoringSpawnResult";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitPriority".
 */
export type CircuitPriority = "firstMapEncounter";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CircuitWork".
 */
export type CircuitWork = "consecutiveEligibleTicksResetOnBlocked";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RefineryAdmission".
 */
export type RefineryAdmission = "caseSensitiveOilPrefixCanonicalKeepsTagsAlternateDiscardsTags";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RefineryContainers".
 */
export type RefineryContainers = "tankOperationHasNoConsumedItemContainers";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RefineryOutputPolicy".
 */
export type RefineryOutputPolicy = "drainBeforeTryFillIgnoringRejectedFuel";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RefineryScope".
 */
export type RefineryScope = "fluidTankConversion";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RefineryWork".
 */
export type RefineryWork = "poweredEnabledCountdownResetOnBlocked";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DistilleryDebit".
 */
export type DistilleryDebit = "oneEachOccupiedSignedJarDeltaNoContainers";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DistilleryMetadata".
 */
export type DistilleryMetadata = "exact" | "ignored";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DistilleryJarPolicy".
 */
export type DistilleryJarPolicy = "runtimeCountOnlyClayJarAdmission";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DistilleryMatching".
 */
export type DistilleryMatching = "subtypeMetadataIgnoreCountAndTagsEitherOrder";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DistilleryOutputPolicy".
 */
export type DistilleryOutputPolicy = "emptyCopyMatchingMergeKeepsTagsMismatchedSkipThenDebit";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DistilleryWorkPolicy".
 */
export type DistilleryWorkPolicy = "successTicksPauseOnPowerResetBlockedCarryRecipeChanges";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AeWorldAction".
 */
export type AeWorldAction =
  | {
      kind: "chargedOre";
    }
  | {
      kind: "meteoriteLoot";
    }
  | {
      kind: "fluix";
      liquidTicks: number;
    }
  | {
      explosion: boolean;
      freshFrequency: boolean;
      kind: "singularity";
    }
  | {
      kind: "growth";
      liquidMaterial: string;
      progressEnd: number;
      progressStart: number;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AeWorldMaterial".
 */
export type AeWorldMaterial =
  | "certusQuartzCrystalCharged"
  | "logicProcessorPress"
  | "calcProcessorPress"
  | "engProcessorPress"
  | "fluixCrystal"
  | "qESingularity"
  | "purifiedCertusQuartzCrystal"
  | "purifiedNetherQuartzCrystal"
  | "purifiedFluixCrystal";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "JarMode".
 */
export type JarMode = "absent" | "conditional";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InfernalInput".
 */
export type InfernalInput = "stack" | "ore";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CanChisel".
 */
export type CanChisel = "nativeItemChiselAlways";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DamagePolicy".
 */
export type DamagePolicy = "vanillaCreativeUnbreakingUnbreakable";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OnChisel".
 */
export type OnChisel = "configuredAllowChiselDamage";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Selection3".
 */
export type Selection3 = "nativeLookupExactPartitionsThenWildcard";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SourceNbt".
 */
export type SourceNbt = "ignored";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "TargetTag".
 */
export type TargetTag = "chiselTarget";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ToolOwner".
 */
export type ToolOwner = "team.chisel.item.chisel.ItemChisel";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "HarvestMode".
 */
export type HarvestMode = "garden" | "fish" | "animal";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "WholeStack".
 */
export type WholeStack = "whole-stack";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AltarNbt".
 */
export type AltarNbt = "ignore" | "native-presence-no-compound-at-required-keys";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "JavaNumbers".
 */
export type JavaNumbers = "java-int-float-operations";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OrbCapacity".
 */
export type OrbCapacity = "int-maxEssence-times-float-1-plus-0.04-orbCapacityRunes";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OrbDebit".
 */
export type OrbDebit = "only-LP-accepted-by-network";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "KeptOrb".
 */
export type KeptOrb = "keep";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OrbOwnerTag".
 */
export type OrbOwnerTag = "ownerName";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AltarResult".
 */
export type AltarResult = "stack-conversion" | "soul-network-charge";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RegistrySelection".
 */
export type RegistrySelection = "first-matching-registry-entry";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AltarAcceleration".
 */
export type AltarAcceleration = "native-buffer-transfer-tick-period";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AltarCapacity".
 */
export type AltarCapacity = "int-10000-times-float-1.14-power-betterRunes-plus-0.35-capacityRunes";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AltarDrain".
 */
export type AltarDrain = "int-base-drain-times-float-0.8-power-speedRunes";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AltarQuickness".
 */
export type AltarQuickness = "max-1-floor-100-times-0.8-power-quicknessRunes";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AltarRate".
 */
export type AltarRate = "tier-below-2-base-else-int-base-times-1-plus-0.25-speedRunes";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RitualActivation".
 */
export type RitualActivation = "native-crystal-level-and-owner-network-and-event";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BindingCooldown".
 */
export type BindingCooldown = "decrement-per-affordable-refresh-then-spawn-at-nonpositive";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BindingNetworkGate".
 */
export type BindingNetworkGate = "refresh-cost-required-in-both-idle-and-pending-branches";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BindingDebit".
 */
export type BindingDebit = "only-idle-scan-branch-including-no-match";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BindingResult".
 */
export type BindingResult = "world-entity-after-cooldown";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RitualKey".
 */
export type RitualKey = "AW006Binding" | "AW019FallingTower";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BindingState".
 */
export type BindingState = "var1-is-selected-index-plus-one";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RitualStructure".
 */
export type RitualStructure = "native-valid-rotated-component-structure";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PlacementCallback".
 */
export type PlacementCallback = "net.minecraft.item.ItemBlock" | "gregtech.common.blocks.ItemOres";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativePlacementOutcome".
 */
export type NativePlacementOutcome = "runtime-native-callback";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorComponentChoice".
 */
export type MeteorComponentChoice = "world-nextInt-totalWeight-cumulative-subtraction";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorDestroy".
 */
export type MeteorDestroy = "invert-config-on-any-active-invertExplosionBlockDamage";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorEligibility".
 */
export type MeteorEligibility = "air-after-optional-explosion";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorEmptyPool".
 */
export type MeteorEmptyPool = "native-default-fallback";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorExplosion".
 */
export type MeteorExplosion = "unless-any-active-disableExplosions-strength-modified-radius-times-4";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorFillerOverride".
 */
export type MeteorFillerOverride =
  "concatenate-active-reagent-fillers-in-active-reagent-order-unless-filtered-first-is-default";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorFillerChance".
 */
export type MeteorFillerChance = "native-float-extrema-raw-adjustment-then-relative-increase-else-decrease-clamp-0-100";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorJointCounts".
 */
export type MeteorJointCounts = "one-selected-block-per-eligible-site-not-independent-yields";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorPlacement".
 */
export type MeteorPlacement =
  "subtract-every-weight-skip-null-or-nonblock-then-call-native-placeBlockAt-stop-even-if-false";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorPlacementSuccess".
 */
export type MeteorPlacementSuccess = "runtime-world-and-native-block-callback-gate";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorPoolChoice".
 */
export type MeteorPoolChoice = "world-nextFloat-times-100-less-than-modified-fillerChance";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorRadius".
 */
export type MeteorRadius = "max-1-base-plus-largest-positive-and-most-negative-radiusChange";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorReagentPredicate".
 */
export type MeteorReagentPredicate = "any-required-reagent-identity-or-no-requirement";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorShape".
 */
export type MeteorShape = "integer-offsets-strict-sphere-radius-plus-half";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorResult".
 */
export type MeteorResult = "spatial-weighted-blocks";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FabricatorOperation".
 */
export type FabricatorOperation = "craft" | "smelt";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProcessingMachine".
 */
export type ProcessingMachine = "fermenter" | "moistener" | "carpenter";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CallbackKind".
 */
export type CallbackKind = "forge" | "ic2";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InputPolicy".
 */
export type InputPolicy = "normalFirstOreLogWoodRuntimeItemAndLiteralMetadata";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeOutputPolicy".
 */
export type NativeOutputPolicy = "emptyPrototypeMatchingMergeKeepsTagsBestEffortThenDebit";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "TankPolicy".
 */
export type TankPolicy = "normalWaterAdmissionRuntimeVolumeOnlyPartialDrain";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeVersion".
 */
export type NativeVersion = "2.5.24";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "WorkPolicy".
 */
export type WorkPolicy = "cachedStateFloatProgressPauseResetAndReselectAtFinish";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AcclimatiserAlteration".
 */
export type AcclimatiserAlteration = "activeBoundsExpandClampFiveThenWidthProbabilityRandomHomolog";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AcclimatiserConsumption".
 */
export type AcclimatiserConsumption = "firstItemEqualReagentEvenUnchanged";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ToleranceGate".
 */
export type ToleranceGate = "nativeCanAlterActiveTolerance";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AcclimatiserSelection".
 */
export type AcclimatiserSelection = "firstSystemThenUniformValidOccupiedSlot";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ToleranceKind".
 */
export type ToleranceKind = "Temperature" | "Humidity" | "PH";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Discovery".
 */
export type Discovery = "worldOwnerGeneTracker";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SequenceOutputGate".
 */
export type SequenceOutputGate = "emptyVialCountBelow64";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SequenceEncoding".
 */
export type SequenceEncoding = "signedByteAfterIntegerTruncation";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SequenceProgress".
 */
export type SequenceProgress = "truncatedProgressInteger";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SequenceResume".
 */
export type SequenceResume = "storedSequencedInteger";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SequenceStrength".
 */
export type SequenceStrength = "oneMinusSquaredQualityTimesThreeQuarters";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Conversion".
 */
export type Conversion = "nativeBreedingSystemFirstConversion" | "none";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AnalysisGate".
 */
export type AnalysisGate = "analysableAndUnanalysed";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AnalysisTransform".
 */
export type AnalysisTransform = "nativeAnalyseRewrite" | "nativeSequenceAnalysedFlag";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeeOffset".
 */
export type BeeOffset = "humidityAndLaterMinusOne";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ChargesGate".
 */
export type ChargesGate = "nonzero" | "damagePositive";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DefunctGate".
 */
export type DefunctGate = "allGenesAlreadyBothHomologsUid";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Drain".
 */
export type Drain = "floatAccumulatorOneMillibucketPerTick";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ManipulationGate".
 */
export type ManipulationGate = "nativeDnaManipulable";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MismatchComparison".
 */
export type MismatchComparison = "alleleObjectIdentity";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SerumRootGate".
 */
export type SerumRootGate = "firstSerumGeneRootMember";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SerumKind".
 */
export type SerumKind = "nativeSingleOrArray";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "TimeScale".
 */
export type TimeScale = "mismatchedGenesHalfIncrement" | "geneCount";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GeneTransform".
 */
export type GeneTransform = "analyseThenSetBothHomologs";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ChromosomeSelection".
 */
export type ChromosomeSelection = "uniformNonNullNativeChromosome";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "HomologSelection".
 */
export type HomologSelection = "uniformActiveInactive";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InvalidGenome".
 */
export type InvalidGenome = "noGeneNoConsumption";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "IsolationOutputGate".
 */
export type IsolationOutputGate = "emptySequenceSlot";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PersistentState".
 */
export type PersistentState = "accumulatorsPersistBetweenCycles";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DnaYield".
 */
export type DnaYield = "beeDrone10OtherBee30MatedBee50Other10";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "IncubatorCallback".
 */
export type IncubatorCallback = "fixed" | "larvaeToDrone";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "IncubatorFluidGate".
 */
export type IncubatorFluidGate = "amountOnlyAfterSelection";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "IncubatorOutputGate".
 */
export type IncubatorOutputGate = "nativeRoomCheck";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "IncubatorSelection".
 */
export type IncubatorSelection = "firstRegistryMatchCached";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PolymeriserTimeScale".
 */
export type PolymeriserTimeScale = "geneCountTimesCatalyst";
/**
 * A displayed NEI result is one observation of the machine callback. These variants retain the other outcomes of the same input tuple and RNG draw.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryOutputPolicy".
 */
export type GendustryOutputPolicy =
  | {
      kind: "direct";
    }
  | {
      kind: "sampler";
      sampleMeta: number;
      sampleRegistry: string;
    }
  | {
      kind: "imprinter";
      wasteRegistry: string;
    }
  | {
      emptyCandidatesReturnNull: boolean;
      kind: "mutatron";
      wasteRegistry: string;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryRootKind".
 */
export type GendustryRootKind = "bee" | "tree" | "butterfly" | "other";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryMutationSetting".
 */
export type GendustryMutationSetting = "enabled" | "disabled" | "requirements";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryFluidMachine".
 */
export type GendustryFluidMachine = "protein" | "mutagen";
/**
 * Stable positions in one content-addressed native registry snapshot.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SqueezerSelector".
 */
export type SqueezerSelector =
  | {
      index: number;
      kind: "ordinary";
    }
  | {
      container: number;
      filled: number;
      kind: "container";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "IntegrationRule".
 */
export type IntegrationRule =
  | {
      energy: number;
      expansions: GateExpansion[];
      kind: "gate";
      maximum: number;
      primary: string[];
      red: Chip;
    }
  | {
      blocks: {
        [k: string]: string;
      };
      energy: number;
      facade: string;
      kind: "facade";
      maximum: number;
      plug: string;
      primary: string[];
      wire: string;
      wires: string[];
    }
  | {
      boards: {
        [k: string]: string;
      };
      empty: string;
      energy: number;
      kind: "robot";
      maximum: number;
      primary: string[];
      robot: string;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RailFuelBurn".
 */
export type RailFuelBurn = "scaffoldZeroBlaze1000LavaAmountScrapZeroNativeFurnaceExceptionZero";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RailFuelCache".
 */
export type RailFuelCache = "semiStrictOfferedNbtAbsentReusesLastItemMetaValue";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RailFuelDepletion".
 */
export type RailFuelDepletion = "nonPositiveNullUnitNativeContainerOtherwiseDecrement";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RailFuelFilter".
 */
export type RailFuelFilter = "nativeItemSubtypeExactNbtEitherWildcard";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RailFuelReload".
 */
export type RailFuelReload = "positiveInputRecipeOutputRoomUnpausedBudgetAtMostTenPositiveBurnAddSignedInt";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RailFuelTick".
 */
export type RailFuelTick = "masterServerBurnBudgetMinusFiveEvenIdle";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Operation3".
 */
export type Operation3 = "primalCharm" | "photoDeveloping" | "nodeAttunement";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Effect2".
 */
export type Effect2 =
  | {
      kind: "declared";
    }
  | {
      key: string;
      kind: "golem";
      namespace: string;
    }
  | {
      items: string[];
      key: string;
      kind: "oldFamiliar";
      previous?: string | null;
    }
  | {
      kind: "revolver";
      registered: (number | null)[];
      target: number;
      upgrades: RevolverUpgrade[];
    }
  | {
      augments: FamiliarAugment[];
      items: string[];
      kind: "familiar";
      locale: string;
      names: {
        [k: string]: number | null;
      };
      requiredPreviousLevel: number;
      target: number;
    }
  | {
      definitionWand: boolean;
      kind: "wand";
      name: string;
      prerequisites: string[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FamiliarCondition".
 */
export type FamiliarCondition =
  | {
      kind: "skip";
    }
  | {
      kind: "any";
      required?: (number | null)[] | null;
    }
  | {
      kind: "all";
      required?: (number | null)[] | null;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OreMode".
 */
export type OreMode = "Default" | "Strict" | "Relaxed" | "None";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Predicate".
 */
export type Predicate =
  | {
      kind: "never";
    }
  | {
      exclusive: boolean;
      kind: "ore";
      name: string;
    }
  | {
      kind: "items";
      nbt: boolean;
      templates: Template[];
    }
  | {
      kind: "any";
      options: Predicate[];
    }
  | {
      kind: "deferred";
      rule: Predicate;
      template: Template;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "StartupMatcher".
 */
export type StartupMatcher = "legacy" | "itemAndTags" | "enhanced";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RandomAspects".
 */
export type RandomAspects = {
  bases: AspectAmount[];
  halfCap: boolean;
  kind: "gadomancy";
  refreshMillis: number;
  semantics: CostSemantics;
};
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CostSemantics".
 */
export type CostSemantics = "gadomancyRandomCostsV1";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Need".
 */
export type Need =
  | {
      kind: "stack";
      template: InventoryItemSelector;
    }
  | {
      kind: "list";
      templates: InventoryItemSelector[];
    }
  | {
      kind: "never";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InscriberMode".
 */
export type InscriberMode = "inscribe" | "press";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Part".
 */
export type Part = "rod" | "cap";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PaymentCondition".
 */
export type PaymentCondition = "loadedSalisWorkbenchMixinAndNativePlayerPayment";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Role3".
 */
export type Role3 = "center" | "part" | "screw" | "conductor";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Codec".
 */
export type Codec = "mm0051GsonV1";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Origin2".
 */
export type Origin2 = "nativeItemList" | "registeredItemClass" | "declaredBitsetWitness";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "HarmonyMode".
 */
export type HarmonyMode = "single" | "parallel";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ReferenceData".
 */
export type ReferenceData =
  | {
      consuming: boolean;
      entries: number;
      kind: "cropsNhDefinition";
      nativeVersion: string;
      section: CropsNhSection;
      semantics: CropsNhSemantics;
      source: Origin;
    }
  | {
      alternateSeeds: CropsNhStack[];
      breedingThresholdBits: number;
      crop: CropsNhCropKey;
      crossingThresholdBits: number;
      dropChanceBits: number;
      drops: CropsNhDrop[];
      duplicationCatalysts: CropsNhStack[];
      growthDuration: number;
      hidden: boolean;
      kind: "cropsNhCrop";
      likedBiomes: string[];
      machineBreedingTier: number;
      minSeedBedTier: number;
      requirements: CropsNhRequirement[];
      soil?: CropsNhSoil | null;
      tier: number;
    }
  | {
      crop: CropsNhCropKey;
      kind: "cropsNhAlternateSeed";
      meta?: number | null;
      registry: string;
    }
  | {
      kind: "cropsNhMutation";
      machineDuration: number;
      machineEut: number;
      output: CropsNhCropKey;
      parents: CropsNhCropKey[];
      requirements: CropsNhRequirement[];
    }
  | {
      kind: "cropsNhMutationPool";
      members: CropsNhCropKey[];
      pool: string;
    }
  | {
      backend: UsageBackend;
      consuming: boolean;
      counts: UsageCounts;
      kind: "usageDefinition";
      locale: string;
      nativeVersion: string;
      ready: boolean;
      scope: UsageScope;
      source: Origin;
    }
  | {
      input: UsageInput;
      kind: "usageObservation";
      nativeIndex: number;
      result: UsageResult;
    }
  | {
      content: UsageDocumentContent;
      key: string;
      kind: "usageDocument";
    }
  | {
      base64: string;
      bytes: number;
      height: number;
      key: string;
      kind: "usageResource";
      location: string;
      mime: UsageMime;
      sha256: string;
      width: number;
    }
  | {
      consuming: boolean;
      core: {
        [k: string]: string;
      };
      counts: ProjectBlueCounts;
      gridHeight: number;
      gridWidth: number;
      kind: "projectBlueDefinition";
      nativeRules: number;
      pickup: ProjectBluePickup;
      ready: boolean;
      source: Origin;
    }
  | {
      anchored: boolean;
      ignored: number[];
      index: number;
      inputs: number[];
      kind: "projectBlueRule";
      materialPolicy: ProjectBlueMaterialPolicy;
      mirrored: boolean;
      mode: ProjectBlueMode;
      outputAmount: number;
      recipeClass: string;
      requiredEmpty: number[];
    }
  | {
      damageable?: boolean | null;
      kind: "projectBlueItem";
      maxDamage?: number | null;
      registry: string;
      roles: ProjectBlueItemRole[];
    }
  | {
      block?: string | null;
      key: string;
      kind: "projectBlueCachedMaterial";
      metadata?: number | null;
      name?: string | null;
    }
  | {
      block: string;
      itemBlock: boolean;
      key: string;
      kind: "projectBlueRegisteredMaterial";
      metadata: number;
      owner?: string | null;
      sourceBlock?: string | null;
      sourceMetadata: number;
    }
  | {
      key: string;
      kind: "projectBlueMaterialAlias";
      target: string;
    }
  | {
      cells: (ProjectBlueStack | null)[];
      kind: "projectBlueExample";
      materialKey?: string | null;
      output: ProjectBlueStack;
      rule: number;
    }
  | {
      apiClass: string;
      bufferCapacity: number;
      bufferLeakBiasBits: string;
      bufferLeakCast: BufferLeakCast;
      bufferLeakDivisor: number;
      bufferLeakPhase: BufferLeakPhase;
      byproduct: ByproductStage;
      consuming: boolean;
      counts: LatheCounts;
      energyPerCut: number;
      hardnessGate: HardnessGate;
      intakeDirections: string[];
      intakeSide: IntakeSide;
      intakeSource: IntakeSource;
      kind: "latheDefinition";
      machineClass: string;
      minimumRadius: number;
      outputSpaceCheck: SpaceCheck;
      ready: boolean;
      source: Origin;
      toolApiClass: string;
      toolWear: number;
    }
  | {
      amount: string;
      item: string;
      kind: "latheSample";
      panelOrder: number;
      rawMeta: number;
      rawNbt?: Nbt | null;
      role: SampleRole;
    }
  | {
      callbackClass: string;
      hardness: number;
      initial: number[];
      kind: "latheWorkpiece";
      width: number;
      workpiece: string;
    }
  | {
      hardness: number;
      item: string;
      kind: "latheState";
      lanes: number[];
      transitions: LatheTransition[];
      width: number;
      workpiece: string;
    }
  | {
      callbackClass: string;
      damage: number;
      kind: "latheTool";
      maximum: number;
      tool: string;
    }
  | {
      after?: string | null;
      damage: number;
      hardness: number;
      item: string;
      kind: "latheToolState";
      maximum: number;
      tool: string;
    }
  | {
      colorOrder: ColorOrder;
      consuming: boolean;
      dyes: FireworkDye[];
      effects: FireworkEffect[];
      examples: number;
      fadePolicy: FadePolicy;
      flightPolicy: FlightPolicy;
      gridHeight: number;
      gridWidth: number;
      gunpowder: string;
      kind: "fireworkDefinition";
      outputAmount: string;
      paper: string;
      ready: boolean;
      recipeClass: string;
      rocket: string;
      shapes: FireworkShape[];
      source: Origin;
      star: string;
      unitsPerCell: number;
    }
  | {
      failure?: FireworkFailure | null;
      /**
       * @minItems 9
       * @maxItems 9
       */
      inputs: [
        string | null,
        string | null,
        string | null,
        string | null,
        string | null,
        string | null,
        string | null,
        string | null,
        string | null
      ];
      kind: "fireworkExample";
      label: string;
      output?: FireworkOutput | null;
    }
  | {
      consuming: boolean;
      containerPolicy: ContainerPolicy;
      fuels: number;
      kind: "furnaceFuelDefinition";
      ready: boolean;
      referenceSmeltTicks: number;
      scope: Scope2;
      source: Origin;
      unitsPerBurn: number;
    }
  | {
      burnTicks: number;
      candidates: Candidate[];
      kind: "furnaceFuel";
    }
  | {
      consuming: boolean;
      counts: StructureCatalogCounts;
      family: Family;
      kind: "structureCatalogDefinition";
      matching: Matching;
      projection: Projection;
      ready: boolean;
      source: Origin;
    }
  | {
      channelDefinitions: {
        [k: string]: StructureChannelDefinition;
      };
      channels: {
        [k: string]: number;
      };
      controller: string;
      hasData: boolean;
      hasTierData: boolean;
      kind: "structureCatalogController";
      maxTier: number;
      ordinal: number;
      origin: Origin;
      structure: string;
      tierItems: StructureTierItem[];
    }
  | {
      items: string[];
      kind: "structureComponentLookup";
      nativeHash: string;
      structures: string[];
    }
  | {
      consuming: boolean;
      counts: QuestDeclarationCounts;
      kind: "questsDefinition";
      ready: boolean;
      registered: number;
      registry: string[];
      scope: QuestCaptureScope;
      settings: QuestWorldSettings;
      source: Origin;
      visible: string[];
    }
  | {
      description: QuestDeclaredText;
      icon?: QuestBigItemStack | null;
      kind: "questDeclaration";
      name: QuestDeclaredText;
      ordinal: number;
      policy: QuestDeclaredPolicy;
      prerequisites: QuestPrerequisite[];
      questUuid: string;
      rewards: QuestRewardDeclaration[];
      tasks: QuestTaskDeclaration[];
      visibility: QuestVisibilityRule;
    }
  | {
      consuming: boolean;
      counts: BartworksOreCounts;
      enabledWeight: number;
      kind: "bartworksOresDefinition";
      layerDiscovery: BartworksOreLayerDiscovery;
      nativeOverlayPolicy: BartworksOreOverlayPolicy;
      ready: boolean;
      registeredMaterials: number;
      source: Origin;
    }
  | {
      activeOrdinal?: number | null;
      bwMask: number;
      density: number;
      dimension: BartworksOreDimension;
      enabled: boolean;
      generatorClass: string;
      key: string;
      kind: "bartworksOreLayer";
      maxY: number;
      minY: number;
      ordinal: number;
      replacement: BartworksOreReplacement;
      roles: BartworksOreRole[];
      size: number;
      weight: number;
    }
  | {
      defaultName: string;
      key: string;
      kind: "bartworksOreMaterial";
      meta: number;
      name: string;
      ordinal: number;
      oreFeature: boolean;
      samples: BartworksOreSample[];
      smallOreFeature: boolean;
    }
  | {
      kind: "bartworksOreLookup";
      layers: string[];
      meta: number;
      ordinal: number;
    }
  | {
      consuming: boolean;
      counts: GtMineralCounts;
      dropLookup: GtMineralDropLookup[];
      dustMaterials: GtMineralMaterial[];
      family: GtMineralFamily;
      hostPolicy: GtMineralHostPolicy;
      kind: "gtMineralsDefinition";
      ready: boolean;
      restrictBiomeSupport?: boolean | null;
      shareSemantics: GtMineralShareSemantics;
      source: Origin;
    }
  | {
      configured: GtMineralSettings;
      dimensions: GtMineralDimension[];
      displayDimensions: string[];
      displayHeight: string;
      displayName?: string | null;
      drops: GtMineralDrop[];
      enabledByDefault: boolean;
      generator?: GtMineralGenerator | null;
      key: string;
      kind: "gtMineralEntry";
      nativeWeight: number;
      ordinal: number;
      roles: GtMineralRole[];
    }
  | {
      denominator: number;
      dimension: string;
      key: string;
      kind: "gtMineralDimensionShare";
      nativeShare?: string | null;
      nonFinite?: GtMineralNonFinite | null;
      numerator: number;
      ordinal: number;
    }
  | {
      amountGeneration: AmountGeneration;
      consuming: boolean;
      counts: Counts;
      kind: "undergroundFluidsDefinition";
      ready: boolean;
      selection: Selection;
      source: Origin;
    }
  | {
      dimension: string;
      displayDimension?: string | null;
      entries: number;
      kind: "undergroundFluidDimension";
      nullTickets: number;
      totalWeight: number;
    }
  | {
      decreasePerOperation: number;
      dimension: string;
      fluid?: string | null;
      kind: "undergroundFluidEntry";
      maxAmount: number;
      minAmount: number;
      ordinal: number;
      registry: string;
      selectedTickets: number;
      weight: number;
    }
  | {
      dimension: string;
      kind: "undergroundFluidDisplay";
      maxAmount: number;
      minAmount: number;
      nativeChance: number;
      ordinal: number;
      registry: string;
    }
  | {
      ancestorSecrets: AncestorSecrets;
      ancestorSelection: AncestorSelection;
      consuming: boolean;
      counts: Counts2;
      kind: "beeTreeDefinition";
      nativeTreeDepth: number;
      nativeVisibleParentDepth: number;
      ready: boolean;
      root: string;
      showDuplicateTrees: boolean;
      showSecret: boolean;
      source: Origin;
      specimens: Specimens;
    }
  | {
      drone: string;
      kind: "beeTreeSpecies";
      name: string;
      princess: string;
      species: string;
    }
  | {
      chance: Chance;
      conditions: string[];
      eligible: boolean;
      handler: string;
      kind: "beeTreeMutation";
      ordinal: number;
      /**
       * @minItems 2
       * @maxItems 2
       */
      parents: [string, string];
      result: string;
      secret: boolean;
      template: (string | null)[];
    }
  | {
      consuming: boolean;
      counts: Counts3;
      kind: "forestryAddonsDefinition";
      memberTypes: {
        [k: string]: number;
      };
      mode: Mode;
      outputOrder: OutputOrder;
      probability: Probability;
      ready: boolean;
      root: string;
      showSecret?: boolean | null;
      source: Origin;
      specimens: Specimens2;
    }
  | {
      jubilance?: string | null;
      jubilanceProvider?: string | null;
      kind: "forestryAddonsSpecies";
      listed: boolean;
      name: string;
      ordinal: number;
      outputs: number;
      species: string;
      specimens: Specimen[];
      template: (string | null)[];
    }
  | {
      amount: string;
      item: string;
      kind: "forestryAddonsOutput";
      ordinal: number;
      rate: Scalar;
      role: OutputRole;
      species: string;
    }
  | {
      base: Scalar;
      conditions?: string[] | null;
      eligible: boolean;
      kind: "forestryAddonsMutation";
      ordinal: number;
      owner: string;
      /**
       * @minItems 2
       * @maxItems 2
       */
      parents: [string, string];
      result: string;
      secret: boolean;
      template: (string | null)[];
    }
  | {
      automationExtraction: AutomationExtraction;
      config: Config;
      consuming: boolean;
      core: string;
      fuelMatching: FuelMatching;
      fuels: Fuel[];
      kind: "draconicReactorDefinition";
      manualAccess: ManualAccess;
      manualExtraction: ManualExtraction;
      manualStackUpgradeThreshold: number;
      operation: Operation;
      products: Product[];
      ready: boolean;
      scope: Scope3;
      source: Origin;
    }
  | {
      consuming: boolean;
      dimensionBlacklist: number[];
      enabled: boolean;
      enchanting: Enchanting;
      entries: number;
      kind: "infernalDropsDefinition";
      quantitySource: QuantitySource;
      ready: boolean;
      scope: Scope4;
      selection: Selection2;
      source: Origin;
      tiers: number;
    }
  | {
      entries: number;
      kind: "infernalTier";
      rarity: number;
      requestedMaxModifiers: number;
      requestedMinModifiers: number;
      tier: Tier;
    }
  | {
      amount: string;
      enchantability: number;
      enchantmentKind: EnchantmentKind;
      entry: number;
      item: string;
      kind: "infernalPoolItem";
      tier: Tier;
    }
  | {
      consuming: boolean;
      drops: number;
      hiddenMode: boolean;
      kind: "mobDropsDefinition";
      mobs: number;
      ready: boolean;
      source: Origin;
    }
  | {
      additionalInfo: string[];
      damageWeightTotal: number;
      displayEntity: string;
      drops: number;
      entity: string;
      entityClass: string;
      infernalType: number;
      kind: "mobDefinition";
      maxHealth: string;
      mod: string;
      name: string;
      peaceful: boolean;
      representations: string[];
      spawnKnown: boolean;
      spawns: MobSpawn[];
      vial: boolean;
      visible: boolean;
    }
  | {
      additionalInfo: string[];
      amount: string;
      baseChance: number;
      chanceScale: number;
      damages?: MobDamage[] | null;
      drop: number;
      dropType: MobDropType;
      enchantmentPower?: number | null;
      entity: string;
      item: string;
      kind: "mobDrop";
      lootable: boolean;
      modifiers: MobChanceModifier[];
      playerOnly: boolean;
      variableChance: boolean;
    }
  | {
      consuming: boolean;
      entries: number;
      kind: "lexiconDefinition";
      pages: number;
      source: Origin;
    }
  | {
      entry: string;
      items: LexiconItem[];
      kind: "lexiconEntry";
      knowledge: LexiconKnowledge;
      name: LexiconText;
      tagline?: LexiconText | null;
    }
  | {
      consuming: boolean;
      kind: "itemAspectsDefinition";
      ready: boolean;
      scans: number;
      source: Origin;
      visibility: Visibility;
    }
  | {
      aspects?: AspectAmount[] | null;
      item?: string | null;
      kind: "itemAspectObservation";
      scan: string;
      status: ScanStatus;
    }
  | {
      aspects: number;
      consuming: boolean;
      kind: "aspectCombinationsDefinition";
      source: Origin;
      visibility: Visibility;
    }
  | {
      aspect: string;
      components: string[];
      kind: "aspectCombination";
      visible: boolean;
    }
  | {
      kind: "villagerProfession";
      mod: string;
      name: string;
      offers: number;
      profession: number;
      source: Origin;
    }
  | {
      inputs: TradeItem[];
      kind: "villagerOffer";
      nativeFrequency: string;
      offer: number;
      output: TradeItem;
      profession: number;
      source: Origin;
    }
  | {
      consuming: boolean;
      containers: ContainerReference[];
      counts: ReferenceCounts;
      discovery: ReferenceDiscovery;
      explanation: ReferenceExplanation;
      kind: "definition";
      source: Origin;
      tracker: Tracker;
    }
  | {
      definition: ReferenceRoot;
      kind: "root";
    }
  | {
      chromosome: ReferenceChromosome;
      kind: "chromosome";
      root: string;
    }
  | {
      allele: Allele;
      chromosome: number;
      kind: "allele";
      root: string;
    }
  | {
      gene: GeneReference;
      kind: "gene";
    }
  | {
      consuming: boolean;
      craftMetalTools: boolean;
      kind: "toolMaterialsDefinition";
      materials: number;
      nativePages: number;
      parts: number;
      source: Origin;
      version: string;
    }
  | {
      kind: "toolMaterial";
      material: ToolMaterial;
    }
  | {
      display?: string | null;
      displayBinding?: PartDisplayBinding | null;
      index: number;
      kind: "toolMaterialPart";
      materialId: number;
      template: OreTemplate;
    }
  | {
      consuming: boolean;
      enabled: boolean;
      entries: number;
      kind: "forgeRegistryDefinition";
      registry: ForgeRegistryKind;
      source: Origin;
    }
  | {
      items: string[];
      kind: "oreGroup";
      members: ForgeRegistryStack[];
      name: string;
      nativeId: number;
    }
  | {
      containers: ForgeContainerReference[];
      fluid: ForgeFluidReference;
      items: string[];
      kind: "fluid";
    }
  | {
      backend: Backend;
      consuming: boolean;
      counts: NativeInformationCounts;
      kind: "neiInformationDefinition";
      linePolicy: LinePolicy;
      ready: boolean;
      representation: Representation;
      source: Origin;
    }
  | {
      consuming: boolean;
      filterOwner: string | null;
      items: NativeInformationItemLink[];
      kind: "neiInformationPage";
      lines: string[];
      listed: boolean;
      nativeIndex: number;
      page: number;
      precise: boolean | null;
      source: Origin;
      textSources: NativeInformationText[];
    }
  | {
      consuming: boolean;
      counts: NativeDiagramCounts;
      description: string;
      group: string;
      ignoreNbt: boolean;
      kind: "neiDiagramDefinition";
      matcher: NativeDiagramMatcher;
      name: string;
      queryScope: NativeDiagramQueryScope;
      ready: boolean;
      selection: NativeDiagramSelection;
      source: Origin;
    }
  | {
      consuming: boolean;
      content: NativeDiagramContent;
      diagram: number;
      group: string;
      kind: "neiDiagram";
      nativeSlotInsertions: number;
      source: Origin;
    }
  | {
      binding: number;
      component: NativeDiagramComponent;
      consuming: boolean;
      diagrams: number[];
      group: string;
      kind: "neiDiagramBinding";
      query: NativeDiagramQuery;
      source: Origin;
    }
  | {
      bindings: number[];
      consuming: boolean;
      damage: NativeDiagramDamageNormalization;
      group: string;
      kind: "neiDiagramLookup";
      lookup: number;
      original: NativeDiagramComponent;
      source: Origin;
    }
  | {
      action: number;
      consuming: boolean;
      diagrams: number[];
      group: string;
      key: string;
      kind: "neiDiagramAction";
      source: Origin;
    }
  | {
      absentFrequency: EnderStorageAbsentFrequency;
      actions: EnderStorageAction[];
      capturedTick: string;
      configuredSize: number;
      consistency: EnderStorageConsistency;
      consuming: boolean;
      counts: EnderStorageCounts;
      description: string;
      group: string;
      ignoreNbt: boolean;
      kind: "enderStorageDefinition";
      name: string;
      personalLockItem: NativeDiagramComponent;
      personalOwner: string;
      queryRules: EnderStorageQueryRule[];
      ready: boolean;
      scope: EnderStorageSnapshotScope;
      sizes: number[];
      source: Origin;
      storageType: EnderStorageType;
      tankCapacity: number;
    }
  | {
      colours: number[];
      consuming: boolean;
      contents: EnderStorageContents;
      frequency: number;
      group: string;
      kind: "enderStorageState";
      layer: EnderStorageLayer;
      owner: string;
      raw: Nbt | null;
      source: Origin;
      state: number;
    }
  | {
      consuming: boolean;
      diagrams_per_page: number;
      group: string;
      height: number;
      kind: "neiInteractiveGroup";
      page: number;
      source: Origin;
      width: number;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropsNhSection".
 */
export type CropsNhSection = "crops" | "alternateSeeds" | "mutations" | "mutationPools";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropsNhSemantics".
 */
export type CropsNhSemantics = "cropsnhRegistriesV1";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropsNhRequirement".
 */
export type CropsNhRequirement =
  | {
      kind: "minLight";
      level: number;
    }
  | {
      kind: "maxLight";
      level: number;
    }
  | {
      kind: "machineOnly";
    }
  | {
      kind: "worldOnly";
    }
  | {
      kind: "migrated";
    }
  | {
      blocks: CropsNhBlock[];
      kind: "subSoil";
      materials: string[];
      name: string;
      ores: string[];
    }
  | {
      items: CropsNhStack[];
      kind: "machineCatalyst";
      ores: CropsNhOreCatalyst[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropsNhSoil".
 */
export type CropsNhSoil =
  | {
      blocks: CropsNhBlock[];
      kind: "blocks";
      name: string;
    }
  | {
      kind: "anyOf";
      name: string;
      soils: CropsNhSoil[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageBackend".
 */
export type UsageBackend = "aeItemCell" | "aeFluidCell" | "ocManual" | "ocCallbacks";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageScope".
 */
export type UsageScope = "catalogObservations";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageResult".
 */
export type UsageResult =
  | {
      channel: UsageChannel;
      fault: CellFaultKind;
      inventory: string;
      kind: "cellFault";
      nativeException: CellNativeException;
      provider: string;
    }
  | {
      channel: UsageChannel;
      entries: UsageEntry[];
      inventory: string;
      kind: "cell";
      provider: string;
      totalBytes: string;
      totalTypes: string;
      usedBytes: string;
      usedTypes: string;
    }
  | {
      document: string;
      kind: "manual";
      path: string;
    }
  | {
      documents: string[];
      kind: "callbacks";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageChannel".
 */
export type UsageChannel = "items" | "fluids";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CellFaultKind".
 */
export type CellFaultKind = "missingCellInventory";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CellNativeException".
 */
export type CellNativeException = "java.lang.NullPointerException";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageEntryKind".
 */
export type UsageEntryKind = "item" | "fluid";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageDocumentContent".
 */
export type UsageDocumentContent =
  | {
      fallbackAttempted: boolean;
      images: UsageImage[];
      kind: "manual";
      lines: string[];
      links: UsageLink[];
      path: string;
      provider: string | null;
      redirects: string[];
      requestedLocale: string;
      resolvedPath: string;
      status: UsageDocumentStatus;
    }
  | {
      analysis: UsageAnalysis;
      environment: string;
      ignoredAnnotations: number;
      kind: "callbacks";
      methods: UsageCallback[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageImage".
 */
export type UsageImage =
  | {
      href: string;
      kind: "png";
      label: string;
      resource: string;
    }
  | {
      href: string;
      items: string[];
      kind: "items";
      label: string;
    }
  | {
      href: string;
      kind: "missing";
      label: string;
      nativeWarning: UsageMissingWarning;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageMissingWarning".
 */
export type UsageMissingWarning =
  | "oc:gui.Manual.Warning.ImageMissing"
  | "oc:gui.Manual.Warning.ItemMissing"
  | "oc:gui.Manual.Warning.BlockMissing"
  | "oc:gui.Manual.Warning.OreDictMissing";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageLinkKind".
 */
export type UsageLinkKind = "document" | "external";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageDocumentStatus".
 */
export type UsageDocumentStatus = "found" | "missing";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageAnalysis".
 */
export type UsageAnalysis = "nativeStaticAnnotations";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageMime".
 */
export type UsageMime = "image/png";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProjectBluePickup".
 */
export type ProjectBluePickup = "vanillaWorkbenchAllOccupiedCells";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProjectBlueMaterialPolicy".
 */
export type ProjectBlueMaterialPolicy = "defaultWood" | "coverRequiresMaterial" | "numericSprayColor";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProjectBlueMode".
 */
export type ProjectBlueMode = "panel" | "miniature" | "paint";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProjectBlueItemRole".
 */
export type ProjectBlueItemRole = "saw" | "spray" | "lever" | "button" | "fullLever" | "stoneButton" | "woodButton";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BufferLeakCast".
 */
export type BufferLeakCast = "truncateToInt";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BufferLeakPhase".
 */
export type BufferLeakPhase = "beforeIntake";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ByproductStage".
 */
export type ByproductStage = "afterStateChange";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "HardnessGate".
 */
export type HardnessGate = "strictlyGreater";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "IntakeSide".
 */
export type IntakeSide = "opposite";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "IntakeSource".
 */
export type IntakeSource = "firstNonzeroAdvertisedSource";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SpaceCheck".
 */
export type SpaceCheck = "beforeStateChange";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SampleRole".
 */
export type SampleRole = "workpiece" | "tool";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Comparison".
 */
export type Comparison = "lessThan";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ColorOrder".
 */
export type ColorOrder = "gridScan";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EffectKey".
 */
export type EffectKey = "Trail" | "Flicker";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FadePolicy".
 */
export type FadePolicy = "copyTaggedStarModifyExistingCompound";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FlightPolicy".
 */
export type FlightPolicy = "onlyWithChargeCell" | "alwaysWithPowder";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FailureCode".
 */
export type FailureCode = "malformedExplosion";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ContainerPolicy".
 */
export type ContainerPolicy = "exhaustedFuelStack";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Scope2".
 */
export type Scope2 = "nativePanelFuelSamples";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FuelContainerRule".
 */
export type FuelContainerRule =
  | {
      comparison: FuelFirestoneComparison;
      damageStep: number;
      degradationThreshold: string;
      degradedBase: string;
      degradedReturn?: Container | null;
      kind: "railcraftFirestone";
      normalReturn?: Container | null;
      sourceMaxDamage: number;
      stage: FuelFirestoneStage;
    }
  | {
      energyUsed: number;
      kind: "bloodMagicLavaCrystal";
      resourcePolicy: FuelResourcePolicy;
      returned: Container;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FuelFirestoneComparison".
 */
export type FuelFirestoneComparison = "nextDoubleLessThanDamageFractionTimes1e-4";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FuelFirestoneStage".
 */
export type FuelFirestoneStage = "refined" | "cracked";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FuelResourcePolicy".
 */
export type FuelResourcePolicy = "ownerNameServerWorldZeroCreateNetworkDebitIfEnough";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Family".
 */
export type Family = "gt" | "compat";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Matching".
 */
export type Matching = "nativeItemHashDamageSum";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Projection".
 */
export type Projection = "completeNativeMapAndItemPanel";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "StructureChannelDefinitionKind".
 */
export type StructureChannelDefinitionKind = "bounded" | "unbounded" | "custom";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "StructureChannelNormalization".
 */
export type StructureChannelNormalization = "clampToBounds" | "opaqueCustom";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "StructureChannelValidation".
 */
export type StructureChannelValidation = "inclusiveBounds" | "opaqueCustom";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestCaptureScope".
 */
export type QuestCaptureScope = "nativeCurrentPlayerVisibility";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestLogicalOperation".
 */
export type QuestLogicalOperation = "AND" | "NAND" | "OR" | "NOR" | "XOR" | "XNOR";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestTargetState".
 */
export type QuestTargetState = "visible" | "hidden" | "missing";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestPrerequisiteType".
 */
export type QuestPrerequisiteType = "NORMAL" | "IMPLICIT" | "HIDDEN";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestRewardRule".
 */
export type QuestRewardRule =
  | {
      grantPolicy: QuestRewardGrantPolicy;
      ignoreDisabled: boolean;
      items: QuestBigItemStack[];
      type: "item";
    }
  | {
      grantPolicy: QuestRewardGrantPolicy;
      ignoreDisabled: boolean;
      items: QuestBigItemStack[];
      selectionCount: number;
      type: "choice";
    }
  | {
      amount: number;
      ignoreDisabled: boolean;
      levels: boolean;
      type: "xp";
    }
  | {
      command: string;
      hideCommand: boolean;
      ignoreDisabled: boolean;
      type: "command";
      viaPlayer: boolean;
    }
  | {
      criteria: string;
      ignoreDisabled: boolean;
      relative: boolean;
      score: string;
      type: "scoreboard";
      value: number;
    }
  | {
      ignoreDisabled: boolean;
      questUuid?: string | null;
      type: "questCompletion";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestRewardGrantPolicy".
 */
export type QuestRewardGrantPolicy = "baseStackSplit";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestTaskRule".
 */
export type QuestTaskRule =
  | {
      autoConsume: boolean;
      consume: boolean;
      groupDetect: boolean;
      ignoreNBT: boolean;
      items: QuestBigItemStack[];
      optional: boolean;
      partialMatch: boolean;
      type: "retrieval";
    }
  | {
      allowAnvil: boolean;
      allowCraft: boolean;
      allowCraftedFromStatistics: boolean;
      allowSmelt: boolean;
      ignoreNBT: boolean;
      items: QuestBigItemStack[];
      partialMatch: boolean;
      type: "crafting";
    }
  | {
      autoConsume: boolean;
      consume: boolean;
      fluids: QuestFluidRequirement[];
      groupDetect: boolean;
      ignoreNBT: boolean;
      type: "fluid";
    }
  | {
      amount: number;
      entity: string;
      ignoreNBT: boolean;
      range: number;
      subtypes: boolean;
      targetNBT?: Nbt | null;
      type: "meeting";
    }
  | {
      damageType: string;
      entity: string;
      ignoreNBT: boolean;
      required: number;
      subtypes: boolean;
      targetNBT?: Nbt | null;
      type: "hunt";
    }
  | {
      block: QuestBlockRequirement;
      ignoreNBT: boolean;
      item?: QuestBigItemStack | null;
      onHit: boolean;
      onInteract: boolean;
      partialMatch: boolean;
      required: number;
      type: "interactItem";
    }
  | {
      entity: string;
      entitySubtypes: boolean;
      ignoreEntityNBT: boolean;
      ignoreItemNBT: boolean;
      item?: QuestBigItemStack | null;
      onHit: boolean;
      onInteract: boolean;
      partialItemMatch: boolean;
      required: number;
      targetNBT?: Nbt | null;
      type: "interactEntity";
    }
  | {
      type: "checkbox";
    }
  | {
      biome: number;
      dim: number;
      hideInfo: boolean;
      invert: boolean;
      name: string;
      range: number;
      structure: string;
      taxiCab: boolean;
      type: "location";
      visible: boolean;
      x: number;
      y: number;
      z: number;
    }
  | {
      amount: number;
      consume: boolean;
      levels: boolean;
      type: "xp";
    }
  | {
      blocks: QuestBlockRequirement[];
      type: "blockBreak";
    }
  | {
      conversionBits: string;
      criteria: string;
      operation: QuestScoreOperation;
      scoreDisp: string;
      scoreName: string;
      suffix: string;
      target: number;
      type: "scoreboard";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestScoreOperation".
 */
export type QuestScoreOperation = "EQUAL" | "LESS_THAN" | "MORE_THAN" | "LESS_OR_EQUAL" | "MORE_OR_EQUAL" | "NOT";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestVisibilityRule".
 */
export type QuestVisibilityRule = "HIDDEN" | "SECRET" | "UNLOCKED" | "NORMAL" | "COMPLETED" | "CHAIN" | "ALWAYS";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BartworksOreLayerDiscovery".
 */
export type BartworksOreLayerDiscovery = "activeListThenMetadataLookup";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BartworksOreOverlayPolicy".
 */
export type BartworksOreOverlayPolicy = "oreFeatureForBothVariantsThenLayerOnlyDedup";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BartworksOreRoleKind".
 */
export type BartworksOreRoleKind = "primary" | "secondary" | "between" | "sporadic";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BartworksOreVariant".
 */
export type BartworksOreVariant = "ore" | "smallOre";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralFamily".
 */
export type GtMineralFamily = "smallOre" | "vein";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralHostPolicy".
 */
export type GtMineralHostPolicy = "nativeDimensionSevenHosts";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralShareSemantics".
 */
export type GtMineralShareSemantics = "relativeAmount" | "relativeWeight";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralRoleKind".
 */
export type GtMineralRoleKind = "small" | "primary" | "secondary" | "between" | "sporadic";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralNonFinite".
 */
export type GtMineralNonFinite = "nan" | "positiveInfinity" | "negativeInfinity";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AmountGeneration".
 */
export type AmountGeneration = "nativeFifthPowerVeinData";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Selection".
 */
export type Selection = "nativeThousandTicketsInclusive";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AncestorSecrets".
 */
export type AncestorSecrets = "nativeUnfiltered";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AncestorSelection".
 */
export type AncestorSelection = "firstRegisteredMutation";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Specimens".
 */
export type Specimens = "nativeAnalyzedTemplates";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Mode".
 */
export type Mode = "produce" | "breeding";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OutputOrder".
 */
export type OutputOrder = "stableRoleItemAmountFloatBits";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Probability".
 */
export type Probability = "nominalSpeciesValues";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Specimens2".
 */
export type Specimens2 = "nativeAnalyzedTemplates";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Role".
 */
export type Role = "producer" | "parent1" | "parent2" | "offspring";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OutputRole".
 */
export type OutputRole = "product" | "specialty";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AutomationExtraction".
 */
export type AutomationExtraction = "fullShardStrictlyGreater";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FuelMatching".
 */
export type FuelMatching = "orderedOreDictionary";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FuelRole".
 */
export type FuelRole = "block" | "ingot" | "nugget";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ManualAccess".
 */
export type ManualAccess = "offlineOnly";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ManualExtraction".
 */
export type ManualExtraction = "fuelRefundBeforeFragments";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Operation".
 */
export type Operation = "stateDependentEnergyAndConversion";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProductRole".
 */
export type ProductRole = "tiny" | "small" | "large" | "full";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Scope3".
 */
export type Scope3 = "nativeConfiguredReference";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Enchanting".
 */
export type Enchanting = "nativeInfernalPerSelection";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuantitySource".
 */
export type QuantitySource = "loadedPoolStack";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Scope4".
 */
export type Scope4 = "nativeClientConfiguration";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Selection2".
 */
export type Selection2 = "finalModifierCountRepeatedUniformPool";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Tier".
 */
export type Tier = "elite" | "ultra" | "inferno";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnchantmentKind".
 */
export type EnchantmentKind = "nativeRandomEnchantedBook" | "remainingModifierCount";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MobDropType".
 */
export type MobDropType = "Normal" | "Rare" | "Additional" | "Infernal";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Visibility".
 */
export type Visibility = "nativePlayerKnowledge" | "allRegisteredAspects";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ScanStatus".
 */
export type ScanStatus = "available" | "noAspects" | "cacheMiss" | "invalidKey";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Acceptor".
 */
export type Acceptor = "zeroCharges";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GenePolicy".
 */
export type GenePolicy = "replaceSingleGene" | "replaceSameChromosomeAppendNew";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ContainerKind".
 */
export type ContainerKind = "serum" | "genome";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AcceptorRootGate".
 */
export type AcceptorRootGate = "singleSerumAcceptsAnyRoot" | "sameRootForExistingArray";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ReferenceDiscovery".
 */
export type ReferenceDiscovery = "sequencerCompletion";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Tracker".
 */
export type Tracker = "GeneTracker.worldOwner";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PartDisplayBinding".
 */
export type PartDisplayBinding = {
  kind: "tgregInventory";
  material: string;
  version: string;
};
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForgeRegistryKind".
 */
export type ForgeRegistryKind = "oreDictionary" | "fluidRegistry";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Backend".
 */
export type Backend = "nei" | "extraUtilities";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "LinePolicy".
 */
export type LinePolicy = "neiEscapedNewlines" | "plainNativeText";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Representation".
 */
export type Representation = "unwrappedNativeText";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramMatcher".
 */
export type NativeDiagramMatcher = "explicitComponent" | "nativeMaterial";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramQueryScope".
 */
export type NativeDiagramQueryScope = "completeNativeBindings" | "readyItemListRegistryFluidsAndDiagramComponents";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramVisibility".
 */
export type NativeDiagramVisibility =
  "ALWAYS_SHOWN" | "ALWAYS_HIDDEN" | "SHOW_ON_SHIFT" | "HIDE_ON_SHIFT" | "SHOW_ON_CTRL" | "HIDE_ON_CTRL" | "DISABLED";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramEmptyPredicate".
 */
export type NativeDiagramEmptyPredicate =
  | {
      kind: "never";
    }
  | {
      kind: "slotInsertionsBelow";
      minimum: number;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramNavigation".
 */
export type NativeDiagramNavigation = {
  craftingOpens: NativeDiagramQuery;
  group: string;
  kind: "group";
  usageOpens: NativeDiagramQuery;
};
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramQuery".
 */
export type NativeDiagramQuery = "crafting" | "usage";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramLabel".
 */
export type NativeDiagramLabel =
  | {
      kind: "text";
      text: string;
    }
  | {
      component: NativeDiagramComponent;
      kind: "component";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramComponent".
 */
export type NativeDiagramComponent =
  | {
      fact: string;
      kind: "itemFact";
    }
  | {
      fact: string | null;
      kind: "item";
      meta: number;
      nbt: Nbt | null;
      registry: string;
    }
  | {
      fact: string;
      kind: "fluid";
      nbt: Nbt | null;
      registry: string;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramTooltipElement".
 */
export type NativeDiagramTooltipElement =
  | {
      kind: "text";
      text: string;
    }
  | {
      component: NativeDiagramComponent;
      kind: "description";
    }
  | {
      display: NativeDiagramDisplay;
      kind: "display";
    }
  | {
      codes: string[];
      kind: "formatting";
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramRelationKind".
 */
export type NativeDiagramRelationKind = "circuitRecipe" | "lensRecipe" | "oreStage";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramDamageNormalization".
 */
export type NativeDiagramDamageNormalization = "preserved" | "zeroForDamageableItem" | "notApplicable";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageAbsentFrequency".
 */
export type EnderStorageAbsentFrequency = "unallocatedNativeEmpty";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageOwnerScope".
 */
export type EnderStorageOwnerScope = "global" | "personal";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageConsistency".
 */
export type EnderStorageConsistency = "ownedAtServerTick";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageSnapshotScope".
 */
export type EnderStorageSnapshotScope = "integratedServerGlobalAndCurrentPersonal";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageType".
 */
export type EnderStorageType = "item" | "liquid";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageContents".
 */
export type EnderStorageContents =
  | {
      kind: "item";
      nonEmpty: boolean;
      sizeIndex: number;
      slots: (EnderStorageSlot | null)[];
    }
  | {
      amount: string;
      capacity: number;
      fluid: NativeDiagramComponent | null;
      kind: "liquid";
      nonEmpty: boolean;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageLayer".
 */
export type EnderStorageLayer = "loadedServer" | "persistedServer";
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
export type SpeciesKind = "bee" | "tree" | "butterfly" | "flower";
/**
 * A StructureLib definition, not a claim that one assembled machine passes its world checks.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DefinitionKind".
 */
export type DefinitionKind = "structurelib" | "construction" | "gtconstruction";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RuleKind".
 */
export type RuleKind = "air" | "solid" | "element";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "VisualData".
 */
export type VisualData =
  | {
      bits: string;
      kind: "mathHelper";
    }
  | {
      gaussian: string;
      haveGaussian: boolean;
      kind: "javaRandom";
      seed: string;
      stream: number;
    }
  | {
      fragment: string;
      kind: "cosmicShader";
      profile: CosmicShaderProfile;
      version: string;
      vertex: string;
    }
  | {
      kind: "mesh";
      parts: InventoryPart[];
    }
  | {
      anchor?: InventoryStep | null;
      /**
       * @minItems 2
       * @maxItems 2
       */
      atlas: [number, number];
      columns: number;
      filter: InventorySampler;
      frames: number;
      kind: "nativeSampler";
      /**
       * One lossless frame sheet for each actually uploaded native mip.
       */
      levels: string[];
      lod: InventoryLod;
      /**
       * @minItems 2
       * @maxItems 2
       */
      origin: [number, number];
      phase: number;
      /**
       * @minItems 2
       * @maxItems 2
       */
      size: [number, number];
      timeline: InventoryStep[];
    }
  | {
      draws: string[];
      kind: "cosmicInventory";
      renderer: string;
      version: string;
    }
  | {
      clock: ForestryClock;
      draws: string[];
      kind: "forestryInventory";
      profile: ForestryProfile;
      trig: string;
      version: string;
    }
  | {
      kind: "forestryPart";
      lightmap?: string | null;
      mesh: string;
      sampler: string;
      state: InventoryState;
      wing: number;
    }
  | {
      /**
       * @minItems 4
       * @maxItems 4
       */
      border: [string, string, string, string];
      /**
       * @minItems 4
       * @maxItems 4
       */
      coordinate: [string, string, string, string];
      kind: "inventoryLightmap";
      /**
       * @minItems 16
       * @maxItems 16
       */
      matrix: [
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string,
        string
      ];
      pixels: string;
      /**
       * @minItems 2
       * @maxItems 2
       */
      wrap: [number, number];
    }
  | {
      kind: "inventoryFlat";
      /**
       * @minItems 16
       * @maxItems 16
       */
      pulseBasis?:
        | [
            string,
            string,
            string,
            string,
            string,
            string,
            string,
            string,
            string,
            string,
            string,
            string,
            string,
            string,
            string,
            string
          ]
        | null;
      random?: string | null;
      sampler: string;
      state: InventoryState;
      vertices: [string, string, string, string, string, string, string, string, string, string, string, string][];
    }
  | {
      kind: "inventoryCosmic";
      samplers: string[];
      shader: string;
      state: InventoryState;
      uniforms: CosmicBindings;
      vertices: [string, string, string, string, string, string, string, string, string, string, string, string][];
    }
  | {
      kind: "galacticraftKey";
      mesh: string;
      /**
       * Native LWJGL Sys units per second, never assumed to be milliseconds.
       */
      resolution: string;
      sampler: string;
      state: InventoryState;
      trig: string;
      version: string;
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CosmicShaderProfile".
 */
export type CosmicShaderProfile = "avaritia" | "eternal";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ButterflyState".
 */
export type ButterflyState = "flying" | "gliding" | "rising" | "resting" | "hover";
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForestryProfile".
 */
export type ForestryProfile = "bee" | "butterfly";
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
      records: Group2[];
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
      kind: "ore-groups";
      records: OreGroup[];
    }
  | {
      kind: "ore-members";
      records: OreMember[];
    }
  | {
      kind: "programs";
      records: ProgramChunk[];
    }
  | {
      kind: "recipes";
      records: Recipe[];
    }
  | {
      kind: "references";
      records: GeneticReferenceRow[];
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
      kind: "visuals";
      records: Visual[];
    };
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "TopicKind".
 */
export type TopicKind =
  "material" | "circuit" | "bee" | "tree" | "butterfly" | "flower" | "ore" | "structure" | "aspect" | "research";

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
  /**
   * Actual game capture language, shared by all display text and search projections.
   */
  locale: string;
  revision: 41;
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
  groups: Group2[];
  items: Item[];
  materials: Material[];
  models: Model[];
  mutations: Mutation[];
  oreGroups: OreGroup[];
  oreMembers: OreMember[];
  programs: ProgramChunk[];
  recipes: Recipe[];
  references: GeneticReferenceRow[];
  research: Research[];
  shapes: Shape[];
  species: Species[];
  strings: Text[];
  structures: Structure[];
  visuals: Visual[];
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
  /**
   * Still preview plus a reviewed inventory program; never a recipe GUI.
   */
  visual?: string | null;
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
   * Present only for the pinned Botania enchanter's StructureLib display provider.
   */
  completion?: BuildCompletion | null;
  /**
   * @minItems 3
   * @maxItems 3
   */
  controller: [number, number, number];
  /**
   * GT uses south; the pinned compatibility wrapper uses ExtendedFacing.DEFAULT (north). Historical GT builds omit this field.
   */
  facing?: ConstructionFacing | null;
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
   * Coordinates are east, down, north, independent of the native construction orientation.
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
  notes?: Note[];
  order: number;
  program?: string | null;
  /**
   * Independently transported native reference domain, not consuming recipes.
   */
  reference?: string | null;
  source: Origin;
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
 * Native explanatory text, bound to a stable field key and its reader source. Display text is not part of a recipe's semantic identity.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Note".
 */
export interface Note {
  key: string;
  source: string;
  text: string;
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
  tier?: Tier2 | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Tier2".
 */
export interface Tier2 {
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
 * via the `definition` "Group2".
 */
export interface Group2 {
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
  /**
   * Raw ItemStack.getMaxDamage() result; negative values retain mod-specific meaning.
   */
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
 * via the `definition` "OreGroup".
 */
export interface OreGroup {
  id: string;
  /**
   * Number of original registration-list positions, including duplicates.
   */
  members: number;
  /**
   * Exact native ore dictionary name, not a translated item label.
   */
  name: string;
  /**
   * Native Java string ordering of all registered non-null names.
   */
  order: number;
  source: Origin;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OreMember".
 */
export interface OreMember {
  /**
   * A verified concrete NEI example; null when none is known safe to display.
   */
  display?: string | null;
  group: string;
  id: string;
  index: number;
  template: OreTemplate;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OreTemplate".
 */
export interface OreTemplate {
  /**
   * Signed native stackSize. Registry metadata, not a consumption quantity.
   */
  amount: string;
  /**
   * Raw stored metadata; 32767 remains a wildcard template, never a display stack.
   */
  meta: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProgramChunk".
 */
export interface ProgramChunk {
  data: ProgramData;
  id: string;
  offset: number;
  program: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolSettings".
 */
export interface BeePoolSettings {
  clearInvalidChromosomes: boolean;
  forestry: string;
  magicbees: string;
  mask: number;
  revision: number;
  root: string;
  runtime: BeePoolRuntime;
  shift: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolRuntime".
 */
export interface BeePoolRuntime {
  afterNanos: string;
  artifactSha256: string;
  beforeNanos: string;
  clock: string;
  runtime: string;
  scope: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolDefinition".
 */
export interface BeePoolDefinition {
  allele: BeePoolAllele;
  key: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolAllele".
 */
export interface BeePoolAllele {
  dominant: boolean;
  slots: number[];
  uid: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolRegistration".
 */
export interface BeePoolRegistration {
  aliasOnly?: boolean;
  dominant: boolean;
  key: string;
  slots: number[];
  uid: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolAlias".
 */
export interface BeePoolAlias {
  key: string;
  uid: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolTemplate".
 */
export interface BeePoolTemplate {
  alleles: (string | null)[];
  key: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolPair".
 */
export interface BeePoolPair {
  primary: string | null;
  secondary: string | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolForm".
 */
export interface BeePoolForm {
  key: string;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolStack".
 */
export interface BeePoolStack {
  amount: string;
  meta: number;
  nbt: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolItemPredicate".
 */
export interface BeePoolItemPredicate {
  metadata: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolSubtileObservation".
 */
export interface BeePoolSubtileObservation {
  input: BeePoolStack;
  value: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolCacheEntry".
 */
export interface BeePoolCacheEntry {
  accessNanos: string;
  chromosome: BeePoolCachedChromosome | null;
  hash: number;
  key: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolCachedChromosome".
 */
export interface BeePoolCachedChromosome {
  primary: BeePoolCachedAllele | null;
  secondary: BeePoolCachedAllele | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolCachedAllele".
 */
export interface BeePoolCachedAllele {
  beeSpecies: boolean;
  dominant: boolean;
  sourceIdentity: boolean;
  uid: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BeePoolCacheSegment".
 */
export interface BeePoolCacheSegment {
  access: number[];
  buckets: number[][];
  count: number;
  maxWeight: string;
  readCount: number;
  recency: number[];
  threshold: number;
  totalWeight: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "XuNativeMaterial".
 */
export interface XuNativeMaterial {
  block: string;
  id: number;
  name: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FmpNativeName".
 */
export interface FmpNativeName {
  id: number;
  name: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FmpNativeAlias".
 */
export interface FmpNativeAlias {
  name: string;
  target: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "XuNativeSettings".
 */
export interface XuNativeSettings {
  aliasPolicy: AliasPolicy;
  fmpVersion: string;
  lookup: XuLookupState;
  microRegistry: string;
  missingPolicy: MissingPolicy;
  nativeRecipe: string;
  version: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FmpNativeMaterial".
 */
export interface FmpNativeMaterial {
  block: string;
  cutterStrength: number;
  id: number;
  name: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Saw".
 */
export interface Saw {
  damageable: boolean;
  durability: number;
  maxStrength: number;
  nbt: SawNbt;
  owner: string;
  registry: string;
  strength: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FmpNativeSettings".
 */
export interface FmpNativeSettings {
  aliasPolicy: AliasPolicy;
  maxSawStrength: number;
  microRegistry: string;
  missingCompoundPolicy: CompoundPolicy;
  missingPolicy: MissingPolicy;
  nativeRecipe: string;
  version: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FermenterRecipe".
 */
export interface FermenterRecipe {
  difficulty: string;
  fermentationValue: number;
  fluid: NativeFluid;
  modifier: string;
  output: string;
  resource: NativeStack;
  resourceModifier: ModifierPolicy;
  variations: FermentationVariation[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeFluid".
 */
export interface NativeFluid {
  amount: number;
  nbt?: Nbt | null;
  registry: string;
  viscosity: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeStack".
 */
export interface NativeStack {
  amount: number;
  container: ContainerPolicy2;
  containerOwner: string;
  maxDamage: number;
  meta: number;
  nbt?: Nbt | null;
  ores: string[];
  registry: string;
  subtypes: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ContainerResult".
 */
export interface ContainerResult {
  amount: number;
  maxDamage: number;
  meta: number;
  nbt?: Nbt | null;
  ores: string[];
  registry: string;
  subtypes: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FermentationVariation".
 */
export interface FermentationVariation {
  modifier: string;
  resource: NativeStack;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FermenterFuel".
 */
export interface FermenterFuel {
  burnDuration: number;
  fermentPerCycle: number;
  item: NativeStack;
  key: FuelKey;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MoistenerRecipe".
 */
export interface MoistenerRecipe {
  product: NativeStack;
  resource: NativeStack;
  timePerItem: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MoistenerFuel".
 */
export interface MoistenerFuel {
  item: NativeStack;
  key: FuelKey;
  moistenerValue: number;
  product?: NativeStack | null;
  stage: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CarpenterRecipe".
 */
export interface CarpenterRecipe {
  box?: NativeStack | null;
  difficulty: string;
  fluid?: NativeFluid | null;
  height: number;
  ingredients: (Ingredient | null)[];
  packagingTime: number;
  result: NativeStack;
  stockItems: NativeStack[];
  width: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FabricatorRecipe".
 */
export interface FabricatorRecipe {
  fluid?: NativeFluid | null;
  height: number;
  ingredients: (Ingredient | null)[];
  plan?: NativeStack | null;
  planUse: PlanUse;
  result: NativeStack;
  stockItems: NativeStack[];
  width: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "WaxPlan".
 */
export interface WaxPlan {
  maxDamage: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FabricatorSmeltingRecipe".
 */
export interface FabricatorSmeltingRecipe {
  meltingPoint: number;
  product: NativeFluid;
  resource: NativeStack;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SqueezerRecipe".
 */
export interface SqueezerRecipe {
  /**
   * Raw IEEE-754 float bits. NaN must not turn into a guaranteed remnant.
   */
  chance: string;
  fluid?: ForestryFluid | null;
  remnant?: ForestryStack | null;
  requirements: (ForestryStack | null)[];
  time: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForestryFluid".
 */
export interface ForestryFluid {
  amount: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForestryStack".
 */
export interface ForestryStack {
  amount: number;
  meta: number;
  nbt?: Nbt | null;
  ores: string[];
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SqueezerContainer".
 */
export interface SqueezerContainer {
  chance: string;
  empty: ForestryStack;
  key: ContainerKey;
  remnant?: ForestryStack | null;
  time: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FilledContainer".
 */
export interface FilledContainer {
  empty: ForestryStack;
  filled: ForestryStack;
  fluid: ForestryFluid;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SqueezerGtFluid".
 */
export interface SqueezerGtFluid {
  emptyMeta: number;
  emptyRegistry: string | null;
  fluid: ForestryFluid;
  meta: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Recipe".
 */
export interface Recipe {
  category: string;
  /**
   * Raw native tick value, when defined. Negative values are upstream anomalies, not elapsed time; preserve them and do not infer an effective execution duration.
   */
  duration?: string | null;
  /**
   * Signed EU/t; negative values denote generation.
   */
  energy?: string | null;
  grid?: Grid | null;
  /**
   * GT native arrays and unresolved execution policy, separate from display projections.
   */
  gtNative?: GtNative | null;
  id: string;
  inputs: Input[];
  magic?: MagicRecipe | null;
  notes?: Note[];
  /**
   * Native category-local position; branches can tie. Sort tied records by id.
   */
  order: number;
  outputs: Output[];
  process?: Process | null;
  /**
   * Namespaced properties with an explicit value type and unit.
   */
  properties: {
    [k: string]: Property;
  };
  source: Origin;
}
/**
 * Native crafting arrangement, independent of presentation.
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
 * via the `definition` "GtNative".
 */
export interface GtNative {
  archiveSha256: string;
  enabled: boolean;
  fake: boolean;
  /**
   * Null falls back to the representative; nonnull empty does not.
   */
  fluidAlternatives?: ((GtStack | null)[] | null)[] | null;
  fluidInputChances?: number[] | null;
  fluidInputs: (GtStack | null)[];
  fluidOutputChances?: number[] | null;
  fluidOutputs: (GtStack | null)[];
  /**
   * Captured raw values, not reconstructed getter results. Null/empty/short differ.
   */
  inputChances?: number[] | null;
  itemAlternatives?: ((GtStack | null)[] | null)[] | null;
  /**
   * Array position is the native slot, including null holes.
   */
  itemInputs: (GtStack | null)[];
  itemOutputs: (GtStack | null)[];
  nativeRecipeClass: string;
  nbtSensitive: boolean;
  outputChances?: number[] | null;
  policy: GtPolicy;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtStack".
 */
export interface GtStack {
  /**
   * Exact signed native int; zero and invalid negative evidence stay distinct.
   */
  amount: string;
  id: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtPolicy".
 */
export interface GtPolicy {
  execution: GtExecution;
  fluidConsumption: GtFluidConsumption;
  itemConsumption: GtItemConsumption;
  projection: GtProjection;
  /**
   * The default helper's sampler, not a promise that every machine uses it.
   */
  sampler: GtSampler;
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
   * Signed-64-bit count/millibuckets. Zero requires a native fluid policy; Forestry has native signed values.
   */
  amount: string;
  consume: Consumption;
  id: string;
  returns: Remainder[];
  rule: Match;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtToolSample".
 */
export interface GtToolSample {
  damage: string;
  electric: boolean;
  limit: string;
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
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SoulFilter".
 */
export interface SoulFilter {
  /**
   * Complement for spawner blacklist; includes null in names.
   */
  exclude: boolean;
  /**
   * Sorted unique native mob strings, with explicit null extraction result.
   */
  names: (string | null)[];
  /**
   * Empty native vessel item reference; metadata does not constrain input.
   */
  vessel: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MatchCase".
 */
export interface MatchCase {
  id: string;
  rule: Match;
}
/**
 * Registered recipe costs before equipment discounts and observed research prerequisites.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MagicRecipe".
 */
export interface MagicRecipe {
  /**
   * Vis for arcane crafting; essentia for crucible and infusion recipes. With Process::Runic these are sample costs, validated against sample charge.
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
   * Null preserves an unregistered recipe knowledge key. Research-tree links require a definition except for Thaumcraft's @ knowledge flags.
   */
  id?: string | null;
  key: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Output".
 */
export interface Output {
  amount?: string | null;
  chance: Chance;
  /**
   * With a change, id/amount are the first input choice's example, not a fixed result.
   */
  change?: Change | null;
  id: string;
  kind: Kind;
  quantity?: Quantity | null;
  role: OutputRole2;
  slot: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Change".
 */
export interface Change {
  action: Edit;
  /**
   * One choice index (or an empty optional slot) per recipe input, per sample. Native correlated processes use tuples. Excluded from recipe identity with samples.
   */
  bindings?: (number | null)[][] | null;
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
 * via the `definition` "Invocation".
 */
export interface Invocation {
  input: Query;
  output?: Query | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Battery".
 */
export interface Battery {
  capacity: Capacity;
  defaulted: boolean;
  offeredId: number;
  offeredRegistry?: string | null;
  selectedId: number;
  selectedRegistry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PoolSelection".
 */
export interface PoolSelection {
  coercions: Coercion[];
  current: PoolPredicate[];
  earlier: PoolPredicate[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Coercion".
 */
export interface Coercion {
  input: string;
  value: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Selector".
 */
export interface Selector {
  metadata: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Head".
 */
export interface Head {
  defaultNameKey: string;
  initialName: string;
  matching: HeadMatching;
  nameTag: string;
  output: HeadOutput;
  sampleName: string;
  visibility: HeadVisibility;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BotaniaWorldSelector".
 */
export interface BotaniaWorldSelector {
  block?: string | null;
  metadata: number;
  registry?: string | null;
  species?: string | null;
  wildcard: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BotaniaWorldBlock".
 */
export interface BotaniaWorldBlock {
  metadata: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Selector2".
 */
export interface Selector2 {
  metadata: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProjectRedPredicate".
 */
export interface ProjectRedPredicate {
  checkNbt: boolean;
  kind: ProjectRedPredicateKind;
  metadata?: number | null;
  nbt?: Nbt | null;
  ores: string[];
  registry?: string | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InventoryItemPrototype".
 */
export interface InventoryItemPrototype {
  amount: string;
  id: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BloodAlchemyPriority".
 */
export interface BloodAlchemyPriority {
  minimumOrb: number;
  /**
   * @minItems 5
   * @maxItems 5
   */
  templates: [
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null
  ];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InventoryItemSelector".
 */
export interface InventoryItemSelector {
  metadata: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BloodAlchemyOrb".
 */
export interface BloodAlchemyOrb {
  level: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BloodOrbCraftingPredicate".
 */
export interface BloodOrbCraftingPredicate {
  alternatives: InventoryItemSelector[];
  minimumOrb?: number | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ImbuingDefinition".
 */
export interface ImbuingDefinition {
  base: ImbuingPredicate;
  ingredients: ImbuingPredicate[];
  outputAlias: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ImbuingPredicate".
 */
export interface ImbuingPredicate {
  metadata: number;
  nbt?: Nbt | null;
  ores: string[];
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SolderingDefinition".
 */
export interface SolderingDefinition {
  /**
   * @minItems 9
   * @maxItems 9
   */
  inputs: [
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null,
    InventoryItemSelector | null
  ];
  result: InventoryItemSelector;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Selector3".
 */
export interface Selector3 {
  code: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Ic2AdvancedLayout".
 */
export interface Ic2AdvancedLayout {
  class: string;
  height: number;
  inputCount: number;
  masks?: number[] | null;
  mirroredMasks?: number[] | null;
  mirroredOrder?: number[] | null;
  shapeless: boolean;
  width: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtCraftingWitness".
 */
export interface GtCraftingWitness {
  choices: number[];
  /**
   * Physical 3x3 inventory cell for each logical input, not GUI coordinates.
   */
  positions: number[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Ic2AdvancedVisibility".
 */
export interface Ic2AdvancedVisibility {
  configValue?: string | null;
  hidden: boolean;
  hideSecretRecipes?: boolean | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtCraftingRules".
 */
export interface GtCraftingRules {
  declaredLevels: number[];
  enchantments: GtCraftingEnchantment[];
  keepingNbt: boolean;
  overwriteNbt: boolean;
  removable: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtCraftingEnchantment".
 */
export interface GtCraftingEnchantment {
  class: string;
  id: number;
  level: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropKey".
 */
export interface CropKey {
  name: string;
  owner: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Module".
 */
export interface Module {
  motorTier: number;
  parallels: number;
  slots: number;
  tier: number;
  voltageTier: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "XuNativeLiteralStack".
 */
export interface XuNativeLiteralStack {
  amount: string;
  meta: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "XuNativeState".
 */
export interface XuNativeState {
  firstPart?: number | null;
  matchedMirrored: boolean;
  offeredMirrored: boolean;
  output: Stack;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CachedGroup".
 */
export interface CachedGroup {
  group: Group;
  key: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Group".
 */
export interface Group {
  cachedWeight: number;
  combine: boolean;
  drops: Drop[];
  id: number;
  maxItems: number;
  minItems: number;
  name?: string | null;
  rarity: number;
  trash: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Drop".
 */
export interface Drop {
  amount: number;
  descriptor?: string | null;
  identifier?: string | null;
  itemGroup: string;
  limit: number;
  nbt?: string | null;
  random: boolean;
  resolved?: string | null;
  resolvedNbt?: Nbt | null;
  stackLimit?: number | null;
  weight: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ColorOre".
 */
export interface ColorOre {
  color: number;
  name: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Dye".
 */
export interface Dye {
  color: number;
  id: string;
  ores: string[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PatternRule".
 */
export interface PatternRule {
  catalyst?: string | null;
  id: string;
  mask?: string[] | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DistilleryAdmission".
 */
export interface DistilleryAdmission {
  jars: number;
  /**
   * @minItems 2
   * @maxItems 2
   */
  reagents: [DistilleryReagent | null, DistilleryReagent | null];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DistilleryReagent".
 */
export interface DistilleryReagent {
  item: string;
  metadata: DistilleryMetadata;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "DistilleryPrototype".
 */
export interface DistilleryPrototype {
  amount: string;
  id: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeIdentity".
 */
export interface NativeIdentity {
  meta: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "OvenPrior".
 */
export interface OvenPrior {
  jarThreshold: number;
  meta: number;
  oreKey?: string | null;
  registry?: string | null;
  subtypes: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeOvenDefinition".
 */
export interface NativeOvenDefinition {
  gate?: NativeOvenPrototype | null;
  input?: NativeOvenSelector | null;
  jars: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeOvenPrototype".
 */
export interface NativeOvenPrototype {
  count: number;
  metadata: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeOvenSelector".
 */
export interface NativeOvenSelector {
  metadata: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeOvenSmelting".
 */
export interface NativeOvenSmelting {
  metadata: number;
  output: NativeOvenPrototype;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeRitualDispatch".
 */
export interface NativeRitualDispatch {
  action: NativeRitualActionKey;
  bucket: number;
  hash: number;
  metadata: number;
  power: number;
  registry: string;
  target: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeRitualActionKey".
 */
export interface NativeRitualActionKey {
  metadata: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InfernalPrior".
 */
export interface InfernalPrior {
  input: InfernalInput;
  inputCount: number;
  meta: number;
  metaWildcard: boolean;
  oreKey?: string | null;
  registry?: string | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SourceSelection".
 */
export interface SourceSelection {
  item: string;
  nativeId: number;
  nativeWildcard: boolean;
  targets: TargetSelection[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "TargetSelection".
 */
export interface TargetSelection {
  count: number;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Tool".
 */
export interface Tool {
  damageEnabled: boolean;
  item: string;
  maxDamage: number;
  owner: ToolOwner;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PriorPredicate".
 */
export interface PriorPredicate {
  keys: string[];
  meta: number;
  metadataWildcard: boolean;
  registry: string;
  tagged: boolean;
  tier: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AltarOrb".
 */
export interface AltarOrb {
  capacityRule: OrbCapacity;
  debit: OrbDebit;
  free: boolean;
  input: KeptOrb;
  maxEssence: number;
  ownerTag: OrbOwnerTag;
  requireOwner: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "AltarUpgrades".
 */
export interface AltarUpgrades {
  acceleration: AltarAcceleration;
  baseCapacity: number;
  betterCapacityBase: number;
  bufferBaseCapacity: number;
  capacityPerRune: number;
  capacityRule: AltarCapacity;
  decayPerSpeedRune: number;
  displacementBase: number;
  drainRule: AltarDrain;
  orbCapacityPerRune: number;
  quicknessBaseDelay: number;
  quicknessDecayPerRune: number;
  quicknessMinimumDelay: number;
  quicknessRule: AltarQuickness;
  rateRule: AltarRate;
  speedPerRune: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RitualComponent".
 */
export interface RitualComponent {
  stone: number;
  x: number;
  y: number;
  z: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorComponent".
 */
export interface MeteorComponent {
  default: boolean;
  id?: string | null;
  isBlock: boolean;
  placementCallback?: PlacementCallback | null;
  placementOutcome: NativePlacementOutcome;
  required: string[];
  weight: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorGeneration".
 */
export interface MeteorGeneration {
  componentChoice: MeteorComponentChoice;
  destroyRule: MeteorDestroy;
  eligibility: MeteorEligibility;
  emptyPool: MeteorEmptyPool;
  explosion: MeteorExplosion;
  fillerOverride: MeteorFillerOverride;
  fillerRule: MeteorFillerChance;
  jointCounts: MeteorJointCounts;
  placement: MeteorPlacement;
  placementSuccess: MeteorPlacementSuccess;
  poolChoice: MeteorPoolChoice;
  radiusRule: MeteorRadius;
  reagentPredicate: MeteorReagentPredicate;
  shape: MeteorShape;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ReagentIdentity".
 */
export interface ReagentIdentity {
  id: string;
  name: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MeteorModifier".
 */
export interface MeteorModifier {
  disableExplosions: boolean;
  filler: MeteorComponent[];
  fillerChanceChange: number;
  invertExplosionBlockDamage: boolean;
  radiusChange: number;
  rawFillerChanceChange: number;
  reagent: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BottleCallback".
 */
export interface BottleCallback {
  capacity: number;
  kind: CallbackKind;
  nbtKey: string;
  owner: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Domain2".
 */
export interface Domain2 {
  chromosomes: Chromosome[];
  conversions: NativeConversion[];
  memberTypes: number[];
  members: NativeMember[];
  resources: Resources;
  root: string;
  rootClass: string;
  systemClass: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Chromosome".
 */
export interface Chromosome {
  alleleClass: string;
  alleles: Allele[];
  invalid: boolean;
  name: string;
  ordinal: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Allele".
 */
export interface Allele {
  class: string;
  compatible: boolean;
  /**
   * @minItems 2
   * @maxItems 2
   */
  toleranceBounds?: [number, number] | null;
  uid: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeConversion".
 */
export interface NativeConversion {
  analysed: NativeStack2;
  input: NativeStack2;
  output: NativeStack2;
  registered: NativeStack2;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeStack2".
 */
export interface NativeStack2 {
  amount: number;
  item: string;
  meta: number;
  nbt?: Nbt | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeMember".
 */
export interface NativeMember {
  analysed: NativeStack2;
  dnaYield: number;
  manipulable: boolean;
  stack: NativeStack2;
  type: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Resources".
 */
export interface Resources {
  dnaDye: NativeStack2;
  emptySequence: NativeStack2;
  enzyme: NativeStack2;
  ethanol: NativeFluid2;
  fluorescentDye: NativeStack2;
  genome: string;
  polymerBacteria: NativeFluid2;
  rawDna: NativeFluid2;
  sequence: string;
  serum: string;
  vectorBacteria: NativeFluid2;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeFluid2".
 */
export interface NativeFluid2 {
  amount: number;
  fluid: string;
  nbt?: Nbt | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Effect".
 */
export interface Effect {
  effect: number;
  item: NativeStack2;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ToleranceSystem".
 */
export interface ToleranceSystem {
  chromosome: number;
  root: string;
  type: ToleranceKind;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "IncubatorRecipe".
 */
export interface IncubatorRecipe {
  callback: IncubatorCallback;
  chance: number;
  fluidInput: NativeFluid2;
  fluidOutput?: NativeFluid2 | null;
  input: NativeStack2;
  lossChance: number;
  output?: NativeStack2 | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryConfig".
 */
export interface GendustryConfig {
  activationEnergy: number;
  deathChanceArtificial?: number | null;
  deathChanceNatural?: number | null;
  degradeChanceNatural?: number | null;
  dnaPerItem?: number | null;
  dnaTankSize?: number | null;
  labwareConsumeChance?: number | null;
  makePristineBees?: boolean | null;
  maxReceivedEnergy: number;
  maxStoredEnergy: number;
  mjPerItem: number;
  mutagenPerItem?: number | null;
  powerUseRate: number;
  proteinPerItem?: number | null;
  proteinTankSize?: number | null;
  secretChance?: number | null;
  tankSize?: number | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryRoot".
 */
export interface GendustryRoot {
  chromosomes: GendustryChromosome[];
  class: string;
  defaultTemplate: (string | null)[];
  forms: GendustryMemberForm[];
  kind: GendustryRootKind;
  lifespan?: GendustryLifespan | null;
  mutations: GendustryMutationProcess[];
  required: number[];
  speciesChromosome: number;
  templates: GendustryTemplate[];
  uid: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryChromosome".
 */
export interface GendustryChromosome {
  name: string;
  ordinal: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryMemberForm".
 */
export interface GendustryMemberForm {
  meta: number;
  nbt?: Nbt | null;
  ordinal: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryLifespan".
 */
export interface GendustryLifespan {
  chromosome: number;
  values: {
    [k: string]: number;
  };
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryMutationProcess".
 */
export interface GendustryMutationProcess {
  beeMutation: boolean;
  class: string;
  conditions: string[];
  parent0: string;
  parent1: string;
  secret: boolean;
  setting: GendustryMutationSetting;
  template: (string | null)[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryTemplate".
 */
export interface GendustryTemplate {
  alleles: (string | null)[];
  species: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryDnaSource".
 */
export interface GendustryDnaSource {
  amount: number;
  meta: number;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GendustryFluidConfig".
 */
export interface GendustryFluidConfig {
  activationEnergy: number;
  maxReceivedEnergy: number;
  maxStoredEnergy: number;
  mjPerItem: number;
  powerUseRate: number;
  tankSize: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GateExpansion".
 */
export interface GateExpansion {
  chip: Chip;
  id: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Chip".
 */
export interface Chip {
  amount: number;
  meta: number;
  nbt?: Nbt | null;
  registry: string;
  subtypes: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RailBlastFuelSupply".
 */
export interface RailBlastFuelSupply {
  allowed: RailBlastFuelTemplate[];
  burn: RailFuelBurn;
  cache: RailFuelCache;
  depletion: RailFuelDepletion;
  filter: RailFuelFilter;
  reload: RailFuelReload;
  tick: RailFuelTick;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RailBlastFuelTemplate".
 */
export interface RailBlastFuelTemplate {
  item: string;
  metadata: number;
  subtypes: boolean;
  templateAmount: string;
}
/**
 * Ordered requirements. None grid means native greedy shapeless matching in offered row-major cell order; a grid permits translations and its mirror flag.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CraftingSelector".
 */
export interface CraftingSelector {
  grid?: Grid | null;
  inputs: MatchCase[][];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Repair".
 */
export interface Repair {
  enabled: boolean;
  item: string;
  maximum: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Coercion2".
 */
export interface Coercion2 {
  input: string;
  value: string;
}
/**
 * A registered predicate, including wildcard metadata, has no display name/icon.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Template".
 */
export interface Template {
  metadata: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "RevolverUpgrade".
 */
export interface RevolverUpgrade {
  /**
   * Object identities encoded as indices, independently of the public numeric id.
   */
  conflicts: number[];
  id: number;
  instability: number;
  maxLevel: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FamiliarAugment".
 */
export interface FamiliarAugment {
  conditions: FamiliarCondition[];
  conflicts: (number | null)[];
  name: string;
  requiresPrevious: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "LegacyPattern".
 */
export interface LegacyPattern {
  allowExtra: boolean;
  central?: Requirement | null;
  components: Requirement[];
  mode: OreMode;
  predicates?: Predicate[] | null;
  startup: StartupMatcher;
  states: ItemState[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Requirement".
 */
export interface Requirement {
  /**
   * Central input is copied once, a pedestal input twice, before comparison.
   */
  copies: number;
  /**
   * craftingStart copies each component once before the later pedestal comparison.
   */
  cycleTagStrings: {
    [k: string]: string;
  };
  /**
   * Ore groups whose actual member list accepts the original template.
   */
  defaultOres: string[];
  ores: string[];
  tagStrings: {
    [k: string]: string;
  };
  template: Template;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ItemState".
 */
export interface ItemState {
  augmentKeys: string[];
  copiedAugmentKeys: string[];
  copiedTagStrings: {
    [k: string]: string;
  };
  id: string;
  lowercaseKeys: {
    [k: string]: string;
  };
  /**
   * Actual OreDictionary.getOreIDs order, before sorting for catalog Item.tags.
   */
  ores: string[];
  pedestalTagStrings: {
    [k: string]: string;
  };
  /**
   * Native NBTBase.toString observations; compound order is not reconstructed from JSON.
   */
  tagStrings: {
    [k: string]: string;
  };
  wildcardOres: string[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "PedestalReplacement".
 */
export interface PedestalReplacement {
  input?: Template | null;
  output?: ReplacementStack | null;
  strict: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ReplacementStack".
 */
export interface ReplacementStack {
  amount: number;
  item: Template;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Comparison2".
 */
export interface Comparison2 {
  amount: number;
  defaultOres: string[];
  item: Template;
  limit: number;
  ores: string[];
  tags: {
    [k: string]: string;
  };
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "State".
 */
export interface State {
  armorPosition: number;
  comparison: Comparison2;
  id: string;
  invisible?: boolean | null;
  onceTags: {
    [k: string]: string;
  };
  potion: boolean;
  serialized: Nbt;
  twiceTags: {
    [k: string]: string;
  };
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Definition".
 */
export interface Definition {
  count: number;
  fibre: InventoryItemSelector;
  modifiers: InventoryItemSelector[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Definition2".
 */
export interface Definition2 {
  inputs?: Need[] | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "SoulSelector".
 */
export interface SoulSelector {
  material: MatchCase;
  soul: SoulFilter;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GrindingBall".
 */
export interface GrindingBall {
  chance: string;
  choices: MatchCase[];
  duration: number;
  grinding: string;
  power: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GrindingRequirement".
 */
export interface GrindingRequirement {
  amount: string;
  choices: MatchCase[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "VatConsumption".
 */
export interface VatConsumption {
  amount: number;
  id: string;
  rule: Match;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "WandMaterial".
 */
export interface WandMaterial {
  conductor: Template;
  screw: string;
  screws: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Upgrade".
 */
export interface Upgrade {
  bit: number;
  id: number;
  name: string;
  ordinal: number;
  providesCaps: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Mapping".
 */
export interface Mapping {
  metadata: number;
  upgrade?: Upgrade | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "State2".
 */
export interface State2 {
  id: string;
  normalized: Nbt;
  origin: Origin2;
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
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GeneticReferenceRow".
 */
export interface GeneticReferenceRow {
  data: ReferenceData;
  domain: string;
  id: string;
  order: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropsNhStack".
 */
export interface CropsNhStack {
  amount: string;
  item?: string | null;
  meta: number;
  nbt?: Nbt | null;
  nbtWildcard: boolean;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropsNhCropKey".
 */
export interface CropsNhCropKey {
  crop: string;
  numericId: number;
  seed: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropsNhDrop".
 */
export interface CropsNhDrop {
  stack: CropsNhStack;
  weight: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropsNhBlock".
 */
export interface CropsNhBlock {
  meta?: number | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CropsNhOreCatalyst".
 */
export interface CropsNhOreCatalyst {
  amount: string;
  ore: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageCounts".
 */
export interface UsageCounts {
  documents: number;
  duplicates: number;
  matched: number;
  queries: number;
  resources: number;
  scanned: number;
  unmatched: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageInput".
 */
export interface UsageInput {
  amount: string;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageEntry".
 */
export interface UsageEntry {
  amount: string;
  id: string;
  kind: UsageEntryKind;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageLink".
 */
export interface UsageLink {
  anchor: string;
  href: string;
  kind: UsageLinkKind;
  label: string;
  target: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "UsageCallback".
 */
export interface UsageCallback {
  declaringClass: string;
  direct: boolean;
  doc: string;
  getter: boolean;
  javaName: string;
  limit: number;
  name: string;
  setter: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProjectBlueCounts".
 */
export interface ProjectBlueCounts {
  examples: number;
  items: number;
  materials: number;
  roles: number;
  rules: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ProjectBlueStack".
 */
export interface ProjectBlueStack {
  amount: string;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "LatheCounts".
 */
export interface LatheCounts {
  samples: number;
  states: number;
  toolStates: number;
  tools: number;
  transitions: number;
  workpieces: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "LatheTransition".
 */
export interface LatheTransition {
  after: string;
  byproduct: LatheOutput;
  lane: number;
  preflightOutput: LatheOutput;
  roll: LatheRoll;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "LatheOutput".
 */
export interface LatheOutput {
  amount: string;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "LatheRoll".
 */
export interface LatheRoll {
  comparison: Comparison;
  samples: number;
  thresholdBits: string;
  tickets: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FireworkDye".
 */
export interface FireworkDye {
  item: string;
  meta: number;
  rgb: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FireworkEffect".
 */
export interface FireworkEffect {
  item: string;
  nbtKey: EffectKey;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FireworkShape".
 */
export interface FireworkShape {
  item?: string | null;
  type: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FireworkFailure".
 */
export interface FireworkFailure {
  cause: string;
  code: FailureCode;
  exception: string;
  inputs: FireworkFailureInput[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FireworkFailureInput".
 */
export interface FireworkFailureInput {
  slot: number;
  template: OreTemplate;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "FireworkOutput".
 */
export interface FireworkOutput {
  amount: string;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Candidate".
 */
export interface Candidate {
  container?: Container | null;
  containerRule?: FuelContainerRule | null;
  item: string;
  templateAmount: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Container".
 */
export interface Container {
  amount: string;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "StructureCatalogCounts".
 */
export interface StructureCatalogCounts {
  controllers: number;
  lookups: number;
  panelItems: number;
  tierLookups: number;
  unresolvedLookups: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "StructureChannelDefinition".
 */
export interface StructureChannelDefinition {
  kind: StructureChannelDefinitionKind;
  maximum?: number | null;
  minimum?: number | null;
  normalization: StructureChannelNormalization;
  validation: StructureChannelValidation;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "StructureTierItem".
 */
export interface StructureTierItem {
  channels: string[];
  items: string[];
  nativeHash: string;
  tier: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestDeclarationCounts".
 */
export interface QuestDeclarationCounts {
  fluids: number;
  items: number;
  quests: number;
  rewards: number;
  tasks: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestWorldSettings".
 */
export interface QuestWorldSettings {
  editMode: boolean;
  fullySyncQuests: boolean;
  hardcore: boolean;
  livesDefault: number;
  livesMax: number;
  noRewards: boolean;
  partyEnable: boolean;
  viewMode: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestDeclaredText".
 */
export interface QuestDeclaredText {
  key: string;
  raw: string;
  value: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestBigItemStack".
 */
export interface QuestBigItemStack {
  amount: number;
  base: QuestItemPattern;
  candidates: QuestItemPattern[];
  oreActive: boolean;
  oreKey: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestItemPattern".
 */
export interface QuestItemPattern {
  damageable: boolean;
  examples: string[];
  meta: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestDeclaredPolicy".
 */
export interface QuestDeclaredPolicy {
  autoClaim: boolean;
  global: boolean;
  globalShare: boolean;
  lockedProgress: boolean;
  main: boolean;
  questLogic: QuestLogicalOperation;
  repeatRelative: boolean;
  repeatTicks: number;
  silent: boolean;
  simultaneous: boolean;
  taskLogic: QuestLogicalOperation;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestPrerequisite".
 */
export interface QuestPrerequisite {
  questUuid: string;
  state: QuestTargetState;
  type: QuestPrerequisiteType;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestRewardDeclaration".
 */
export interface QuestRewardDeclaration {
  factory: string;
  labelKey: string;
  nativeClass: string;
  nativeId: number;
  rule: QuestRewardRule;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestTaskDeclaration".
 */
export interface QuestTaskDeclaration {
  factory: string;
  labelKey: string;
  nativeClass: string;
  nativeId: number;
  rule: QuestTaskRule;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestFluidRequirement".
 */
export interface QuestFluidRequirement {
  amount: number;
  fluid: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "QuestBlockRequirement".
 */
export interface QuestBlockRequirement {
  amount: number;
  meta: number;
  nbt?: Nbt | null;
  oreKey?: string | null;
  registry?: string | null;
  specimen?: QuestItemPattern | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BartworksOreCounts".
 */
export interface BartworksOreCounts {
  layers: number;
  lookups: number;
  materials: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BartworksOreDimension".
 */
export interface BartworksOreDimension {
  id: number;
  key: string;
  name: string;
  provider: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BartworksOreReplacement".
 */
export interface BartworksOreReplacement {
  metas: number[];
  registry?: string | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BartworksOreRole".
 */
export interface BartworksOreRole {
  bartworks: boolean;
  item: string;
  meta: number;
  role: BartworksOreRoleKind;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BartworksOreSample".
 */
export interface BartworksOreSample {
  amount?: number | null;
  item?: string | null;
  nativeOverlayEligible: boolean;
  variant: BartworksOreVariant;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralCounts".
 */
export interface GtMineralCounts {
  dimensionShares: number;
  entries: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralDropLookup".
 */
export interface GtMineralDropLookup {
  meta: number;
  name: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralMaterial".
 */
export interface GtMineralMaterial {
  key: string;
  meta: number;
  name: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralSettings".
 */
export interface GtMineralSettings {
  amount?: number | null;
  density?: number | null;
  maxY: number;
  minY: number;
  size?: number | null;
  weight?: number | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralDimension".
 */
export interface GtMineralDimension {
  enabled: boolean;
  name: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralDrop".
 */
export interface GtMineralDrop {
  amount: number;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralGenerator".
 */
export interface GtMineralGenerator {
  amount?: number | null;
  biome: string;
  blacklistedProviders: string[];
  density?: number | null;
  flags: GtMineralFlags;
  maxY: number;
  metas: number[];
  minY: number;
  providers: string[];
  size?: number | null;
  weight?: number | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralFlags".
 */
export interface GtMineralFlags {
  end: boolean;
  endAsteroid?: boolean | null;
  nether: boolean;
  overworld: boolean;
  twilightForest: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralRole".
 */
export interface GtMineralRole {
  hosts: GtMineralHost[];
  material: GtMineralMaterial;
  meta: number;
  role: GtMineralRoleKind;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GtMineralHost".
 */
export interface GtMineralHost {
  host: number;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Counts".
 */
export interface Counts {
  dimensions: number;
  displays: number;
  entries: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Counts2".
 */
export interface Counts2 {
  eligible: number;
  mutations: number;
  species: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Counts3".
 */
export interface Counts3 {
  eligible: number;
  mutations: number;
  nativeEntries: number;
  outputs: number;
  species: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Specimen".
 */
export interface Specimen {
  amount: string;
  item: string;
  nativeType: number;
  role: Role;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Scalar".
 */
export interface Scalar {
  bits: string;
  decimal: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Config".
 */
export interface Config {
  automation: boolean;
  configuredFuelStorage: number;
  configuredNuggetUnits: number;
  configuredTinyChaosUnits: number;
  fuelUsageMultiplier: string;
  largeExplosion: boolean;
  linearFuelUsage: boolean;
  maximumFuelStorage: number;
  outputMultiplier: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Fuel".
 */
export interface Fuel {
  item: string;
  ore: string;
  role: FuelRole;
  units: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Product".
 */
export interface Product {
  item: string;
  role: ProductRole;
  units: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MobSpawn".
 */
export interface MobSpawn {
  description: string;
  nativeClass: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MobDamage".
 */
export interface MobDamage {
  meta: number;
  weight: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "MobChanceModifier".
 */
export interface MobChanceModifier {
  description: string[];
  nativeClass: string;
  payload: string;
  priority: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "LexiconItem".
 */
export interface LexiconItem {
  hasQuickInfo: boolean;
  item: string;
  nativeKey: string;
  quickInfo: LexiconText;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "LexiconText".
 */
export interface LexiconText {
  key: string;
  value: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "LexiconKnowledge".
 */
export interface LexiconKnowledge {
  autoUnlock: boolean;
  color: string;
  name: LexiconText;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "TradeItem".
 */
export interface TradeItem {
  enchantmentPower?: number | null;
  item: string;
  /**
   * Signed Java-int observations from Mobs Info, never executable consumption amounts.
   */
  sizes: number[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ContainerReference".
 */
export interface ContainerReference {
  acceptor: Acceptor;
  empty: string;
  genePolicy: GenePolicy;
  item: string;
  kind: ContainerKind;
  rootGate: AcceptorRootGate;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ReferenceCounts".
 */
export interface ReferenceCounts {
  alleles: number;
  chromosomes: number;
  containers: number;
  genes: number;
  roots: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ReferenceExplanation".
 */
export interface ReferenceExplanation {
  key: string;
  text: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ReferenceRoot".
 */
export interface ReferenceRoot {
  genome: string;
  memberTypes: number[];
  root: string;
  rootClass: string;
  sequence: string;
  serum: string;
  systemClass: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ReferenceChromosome".
 */
export interface ReferenceChromosome {
  alleleClass: string;
  invalid: boolean;
  name: string;
  ordinal: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "GeneReference".
 */
export interface GeneReference {
  allele: string;
  chromosome: number;
  genome: string;
  root: string;
  sequence: string;
  serum: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ToolMaterial".
 */
export interface ToolMaterial {
  ability: string;
  arrow?: ArrowMaterial | null;
  attack: number;
  bow?: BowMaterial | null;
  durability: number;
  /**
   * Exact native IEEE754 float bits, including negative native modifiers.
   */
  handleModifier: string;
  harvestLevel: number;
  localizationString: string;
  localizedName: string;
  materialId: number;
  materialName: string;
  miningSpeed: number;
  parts: number;
  prefixName: string;
  primaryColor: number;
  reinforced: number;
  stonebound: string;
  tipStyle: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ArrowMaterial".
 */
export interface ArrowMaterial {
  breakChance: string;
  mass: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "BowMaterial".
 */
export interface BowMaterial {
  drawSpeed: number;
  flightSpeedMax: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForgeRegistryStack".
 */
export interface ForgeRegistryStack {
  amount: number;
  displays: string[];
  meta: number;
  nbt?: Nbt | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForgeContainerReference".
 */
export interface ForgeContainerReference {
  empty: ForgeRegistryStack;
  filled: ForgeRegistryStack;
  fluid: ForgeContainerFluid;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForgeContainerFluid".
 */
export interface ForgeContainerFluid {
  amount: number;
  id: string;
  nbt?: Nbt | null;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForgeFluidReference".
 */
export interface ForgeFluidReference {
  block?: ForgeRegisteredBlock | null;
  density: number;
  gaseous: boolean;
  id: string;
  luminosity: number;
  nativeId: number;
  placeable: boolean;
  registry: string;
  temperature: number;
  viscosity: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForgeRegisteredBlock".
 */
export interface ForgeRegisteredBlock {
  display?: string | null;
  registry: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeInformationCounts".
 */
export interface NativeInformationCounts {
  itemLinks: number;
  listed: number;
  nativeEntries: number;
  pages: number;
  scannedItems: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeInformationItemLink".
 */
export interface NativeInformationItemLink {
  amount: string;
  item: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeInformationText".
 */
export interface NativeInformationText {
  key: string;
  text: string;
  translated: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramCounts".
 */
export interface NativeDiagramCounts {
  actions: number;
  bindings: number;
  diagrams: number;
  lookups: number;
  scannedFluids: number;
  scannedItems: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramSelection".
 */
export interface NativeDiagramSelection {
  configuredVisibility: string | null;
  defaultVisibility: NativeDiagramVisibility;
  empty: NativeDiagramEmptyPredicate;
  showEmptyDiagrams: boolean;
  visibility: NativeDiagramVisibility;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramContent".
 */
export interface NativeDiagramContent {
  annotations: NativeDiagramAnnotation[];
  cells: NativeDiagramCell[];
  groups: NativeDiagramLogicalGroup[];
  labels: NativeDiagramLabel[];
  relations: NativeDiagramRelation[];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramAnnotation".
 */
export interface NativeDiagramAnnotation {
  action: NativeDiagramNavigation | null;
  label: NativeDiagramLabel;
  tooltip: NativeDiagramTooltipElement[][];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramDisplay".
 */
export interface NativeDiagramDisplay {
  amount: string | null;
  component: NativeDiagramComponent;
  note: string | null;
  tooltip: NativeDiagramTooltipElement[][];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramCell".
 */
export interface NativeDiagramCell {
  alternatives: NativeDiagramDisplay[];
  group: string;
  index: number;
  tooltip: NativeDiagramTooltipElement[][];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramLogicalGroup".
 */
export interface NativeDiagramLogicalGroup {
  columns: number;
  key: string;
  rows: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "NativeDiagramRelation".
 */
export interface NativeDiagramRelation {
  from: string;
  kind: NativeDiagramRelationKind;
  to: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageAction".
 */
export interface EnderStorageAction {
  key: string;
  owner: EnderStorageOwnerScope;
  requiresNewSnapshot: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageCounts".
 */
export interface EnderStorageCounts {
  loaded: number;
  persisted: number;
  states: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageQueryRule".
 */
export interface EnderStorageQueryRule {
  highMetadata: number | null;
  registry: string;
  role: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "EnderStorageSlot".
 */
export interface EnderStorageSlot {
  amount: string;
  component: NativeDiagramComponent;
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
  itemTriggers: Clue[];
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
 * A declared TC item trigger, independent of whether NEI lists a concrete example.
 *
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Clue".
 */
export interface Clue {
  /**
   * Sorted unique IDs of observed catalog items accepted by the native matcher. Empty preserves the condition without inventing an obtainable item.
   */
  matches: string[];
  /**
   * Raw metadata. 32767 is TC's wildcard, not a concrete item subtype.
   */
  meta: number;
  nbt?: Nbt | null;
  /**
   * The first native ore group, checked before item/meta/NBT matching.
   */
  ore?: string | null;
  registry: string;
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
  flower?: FlowerTraits | null;
  /**
   * Whether the tree species supports its default template's fruit family.
   */
  fruitCompatible?: boolean | null;
  genes: Gene[];
  humidity: string;
  id: string;
  /**
   * Native bee specialty-condition description; not a live housing eligibility result.
   */
  jubilance?: string | null;
  kind: SpeciesKind;
  members: Member[];
  name: string;
  /**
   * Natural bee or butterfly activity; distinct from nocturnal tolerance chromosomes.
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
 * via the `definition` "FlowerTraits".
 */
export interface FlowerTraits {
  /**
   * Native Botany soil symbols, distinct from Forestry air humidity.
   */
  acidity: string;
  moisture: string;
  /**
   * Native IFlowerType identifier; colors remain separate chromosome values.
   */
  type: number;
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
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Structure".
 */
export interface Structure {
  controller: string;
  /**
   * Older structure records omit this field and use enumerable StructureLib definitions. Construction providers supply actual builds, not invented definition pieces.
   */
  definitionKind?: DefinitionKind | null;
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
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "Visual".
 */
export interface Visual {
  data: VisualData;
  id: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InventoryPart".
 */
export interface InventoryPart {
  /**
   * @minItems 3
   * @maxItems 3
   */
  offset: [string, string, string];
  /**
   * @minItems 3
   * @maxItems 3
   */
  pivot: [string, string, string];
  /**
   * Native Z, Y, X radians, applied with ModelRenderer's float degree conversion.
   *
   * @minItems 3
   * @maxItems 3
   */
  rotation: [string, string, string];
  /**
   * Position XYZ, packed-normal XYZ and UV; two ordered triangles per native quad.
   */
  vertices: [string, string, string, string, string, string, string, string][];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InventoryStep".
 */
export interface InventoryStep {
  frame: number;
  ticks: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InventorySampler".
 */
export interface InventorySampler {
  mag: number;
  min: number;
  s: number;
  t: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InventoryLod".
 */
export interface InventoryLod {
  bias: string;
  max: string;
  min: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "ForestryClock".
 */
export interface ForestryClock {
  alive: boolean;
  itemHash: number;
  state?: ButterflyState | null;
  stateHash?: number | null;
  ticks: number;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InventoryState".
 */
export interface InventoryState {
  alphaFunc: number;
  alphaRef: string;
  alphaTest: boolean;
  /**
   * @minItems 4
   * @maxItems 4
   */
  ambient: [string, string, string, string];
  blend: boolean;
  blendDstAlpha: number;
  blendDstRgb: number;
  blendSrcAlpha: number;
  blendSrcRgb: number;
  /**
   * @minItems 4
   * @maxItems 4
   */
  color: [string, string, string, string];
  colorMaterial: boolean;
  cull: boolean;
  cullFace: number;
  depth: boolean;
  depthFunc: number;
  depthWrite: boolean;
  /**
   * @minItems 4
   * @maxItems 4
   */
  emission: [string, string, string, string];
  frontFace: number;
  lighting: boolean;
  lights: InventoryLight[];
  /**
   * @minItems 4
   * @maxItems 4
   */
  materialAmbient: [string, string, string, string];
  /**
   * @minItems 4
   * @maxItems 4
   */
  materialDiffuse: [string, string, string, string];
  /**
   * @minItems 4
   * @maxItems 4
   */
  materialSpecular: [string, string, string, string];
  /**
   * @minItems 16
   * @maxItems 16
   */
  modelView: [
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string
  ];
  normalize: boolean;
  /**
   * @minItems 16
   * @maxItems 16
   */
  projection: [
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string,
    string
  ];
  rescaleNormal: boolean;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "InventoryLight".
 */
export interface InventoryLight {
  /**
   * @minItems 4
   * @maxItems 4
   */
  ambient: [string, string, string, string];
  /**
   * @minItems 4
   * @maxItems 4
   */
  diffuse: [string, string, string, string];
  /**
   * @minItems 4
   * @maxItems 4
   */
  position: [string, string, string, string];
  /**
   * @minItems 4
   * @maxItems 4
   */
  specular: [string, string, string, string];
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CosmicBindings".
 */
export interface CosmicBindings {
  activeLights: string;
  cosmicuvs: string[];
  externalScale: string;
  /**
   * @minItems 3
   * @maxItems 3
   */
  lightlevel: [string, string, string];
  lightmix: string;
  /**
   * @minItems 2
   * @maxItems 2
   */
  lights: [CosmicLight, CosmicLight];
  opacity: string;
  pitch: string;
  /**
   * @minItems 4
   * @maxItems 4
   */
  sceneColor: [string, string, string, string];
  textureUnit: number;
  time: string;
  yaw: string;
}
/**
 * This interface was referenced by `Contract`'s JSON-Schema
 * via the `definition` "CosmicLight".
 */
export interface CosmicLight {
  /**
   * @minItems 4
   * @maxItems 4
   */
  ambient: [string, string, string, string];
  /**
   * @minItems 3
   * @maxItems 3
   */
  attenuation: [string, string, string];
  /**
   * @minItems 4
   * @maxItems 4
   */
  diffuse: [string, string, string, string];
  /**
   * @minItems 4
   * @maxItems 4
   */
  position: [string, string, string, string];
  spotCosCutoff: string;
  spotCutoff: string;
  /**
   * @minItems 3
   * @maxItems 3
   */
  spotDirection: [string, string, string];
  spotExponent: string;
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
  revision: 41;
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
  revision: 41;
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
  visual?: string | null;
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
