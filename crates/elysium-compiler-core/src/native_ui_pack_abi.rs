//! Native UI runtime pack ABI catalog.
//!
//! This module is the single source of truth for the compact Native UI pack
//! wire format emitted by the compiler and consumed by NeoNEI. Keep version,
//! magic, stride, and public report field catalogs together so ABI changes are
//! explicit and reviewable instead of being scattered through emitters,
//! validators, and gates.

use serde_json::{json, Value};

pub const UI_TEMPLATE_SCHEMA: &str = "neonei/ui-template-pack/current";
pub const UI_BINDING_SCHEMA: &str = "neonei/ui-binding-pack/current";
pub const UI_STRING_SCHEMA: &str = "neonei/ui-string-pack/current";

pub const UI_TEMPLATE_MAGIC: &[u8; 8] = b"NEIUIT1\0";
pub const UI_BINDING_MAGIC: &[u8; 8] = b"NEIUIB1\0";
pub const UI_STRING_MAGIC: &[u8; 8] = b"NEIUIS1\0";

pub const UI_TEMPLATE_MAGIC_REPORT: &str = "NEIUIT1_NUL";
pub const UI_BINDING_MAGIC_REPORT: &str = "NEIUIB1_NUL";
pub const UI_STRING_MAGIC_REPORT: &str = "NEIUIS1_NUL";

pub const UI_TEMPLATE_PAYLOAD_VERSION: u32 = 9;
pub const UI_BINDING_PAYLOAD_VERSION: u32 = 1;
pub const UI_STRING_PAYLOAD_VERSION: u32 = 1;

pub const UI_TEMPLATE_ROW_STRIDE_U32: u32 = 25;
pub const UI_SLOT_ROW_STRIDE_U32: u32 = 12;
pub const UI_TEXT_ROW_STRIDE_U32: u32 = 7;
pub const UI_PRIMITIVE_ROW_STRIDE_U32: u32 = 13;
pub const UI_RECT_ROW_STRIDE_U32: u32 = 15;
pub const UI_BINDING_ROW_STRIDE_U32: u32 = 11;

pub const NATIVE_UI_COORDINATE_SPACE: &str = "nei_pixels";
pub const NATIVE_UI_ANCHOR: &str = "top-left";
pub const NATIVE_UI_SCALE_MODE: &str = "uniform-scale";
pub const NATIVE_UI_GT_BACKGROUND_KIND: &str = "gt-modular-ui";
pub const NATIVE_UI_BACKGROUND_SCALING_NINE_SLICE: &str = "nine-slice";
pub const NATIVE_UI_INTERACTION_KIND_NONE: &str = "none";
pub const NATIVE_UI_INTERACTION_KIND_ITEM_CLICK: &str = "item-click";
pub const NATIVE_UI_INTERACTION_TARGET_NONE: &str = "none";
pub const NATIVE_UI_INTERACTION_TARGET_ITEM: &str = "item";
pub const NATIVE_UI_INTERACTION_PAYLOAD_SCHEMA: &str = "neonei/native-ui-interaction/v1";

pub const SURFACE_CONTRACT_FIELDS: &[&str] = &["coordinateSpace", "scaleMode", "anchor"];
pub const SLOT_GEOMETRY_FIELDS: &[&str] = &[
    "coordinateSpace",
    "anchor",
    "slotWidth",
    "slotHeight",
    "pitchX",
    "pitchY",
];
pub const TEMPLATE_DYNAMIC_PRIMITIVE_FIELDS: &[&str] = &["dynamicPrimitives"];
pub const DYNAMIC_PRIMITIVE_GEOMETRY_FIELDS: &[&str] = &[
    "kind",
    "role",
    "x",
    "y",
    "width",
    "height",
    "coordinateSpace",
    "anchor",
    "orientation",
    "source",
    "trackColor",
    "fillColor",
    "borderColor",
];
pub const RECT_GEOMETRY_FIELDS: &[&str] = &["coordinateSpace", "anchor"];
pub const INTERACTION_CONTRACT_FIELDS: &[&str] = &[
    "interactionKind",
    "interactionTargetKind",
    "interactionTargetId",
    "interactionPayloadSchema",
];
pub const BACKGROUND_CONTRACT_FIELDS: &[&str] = &[
    "coordinateSpace",
    "scaleMode",
    "anchor",
    "status",
    "kind",
    "scaling",
    "texture",
    "recipeBackgroundOffset",
    "recipeBackgroundSize",
];
pub const TEMPLATE_BACKGROUND_FIELD: &str = "nativeBackground";

pub const UI_TEMPLATE_STRING_REF_COLUMNS: &[u32] = &[0, 1, 2, 3, 4, 9, 19, 20, 21, 22];
pub const UI_SLOT_STRING_REF_COLUMNS: &[u32] = &[0, 6, 7];
pub const UI_TEXT_STRING_REF_COLUMNS: &[u32] = &[0, 5, 6];
pub const UI_PRIMITIVE_STRING_REF_COLUMNS: &[u32] = &[0, 1, 6, 7, 8, 9, 10, 11, 12];
pub const UI_RECT_STRING_REF_COLUMNS: &[u32] = &[0, 1, 2, 3, 4, 9, 10, 11, 12, 13, 14];
pub const UI_BINDING_STRING_REF_COLUMNS: &[u32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

/// Public ABI catalog emitted into `ui_pack_report.json`.
pub fn ui_pack_format_report() -> Value {
    json!({
        "templatePackMagic": UI_TEMPLATE_MAGIC_REPORT,
        "templatePackVersion": UI_TEMPLATE_PAYLOAD_VERSION,
        "templateStride": UI_TEMPLATE_ROW_STRIDE_U32,
        "slotStride": UI_SLOT_ROW_STRIDE_U32,
        "textStride": UI_TEXT_ROW_STRIDE_U32,
        "primitiveStride": UI_PRIMITIVE_ROW_STRIDE_U32,
        "rectStride": UI_RECT_ROW_STRIDE_U32,
        "surfaceContractFields": SURFACE_CONTRACT_FIELDS,
        "legacyRectActionFields": false,
        "legacyRectActionFieldNames": [],
        "slotGeometryFields": SLOT_GEOMETRY_FIELDS,
        "templateDynamicPrimitiveFields": TEMPLATE_DYNAMIC_PRIMITIVE_FIELDS,
        "dynamicPrimitiveGeometryFields": DYNAMIC_PRIMITIVE_GEOMETRY_FIELDS,
        "rectGeometryFields": RECT_GEOMETRY_FIELDS,
        "interactionContractFields": INTERACTION_CONTRACT_FIELDS,
        "backgroundContractFields": BACKGROUND_CONTRACT_FIELDS,
        "templateBackgroundField": TEMPLATE_BACKGROUND_FIELD,
    })
}
