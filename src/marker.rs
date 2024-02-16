use serde::Serialize;

use crate::CoreRouterCommon;

#[derive(Serialize)]
struct MarkerPath {
    #[serde(rename = "@d")]
    d: &'static str,
    #[serde(flatten)]
    attributes: CoreRouterCommon,
}

impl Default for MarkerPath {
    fn default() -> Self {
        let mut attributes = CoreRouterCommon::default();
        attributes.set_fill("black".to_string());

        Self {
            d: crate::MARKER_PATH,
            attributes,
        }
    }
}

#[derive(Serialize)]
pub struct Marker {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "@orient")]
    orient: &'static str,
    #[serde(rename = "@markerWidth")]
    marker_width: &'static str,
    #[serde(rename = "@markerHeight")]
    marker_height: &'static str,
    #[serde(rename = "@refY")]
    ref_y: &'static str,
    path: MarkerPath,
}

impl Default for Marker {
    fn default() -> Self {
        Self {
            id: "arrowHead",
            orient: "auto",
            marker_width: "8",
            marker_height: "8",
            ref_y: "4",
            path: MarkerPath::default(),
        }
    }
}
