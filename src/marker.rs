/// SVG marker useed in connection lines.
/// This just gets deserialiseed as-is. No logic here.
use const_format::concatcp;
use serde::Serialize;

use crate::{CommonAttributes, CoordinateT};

pub(crate) static MARKER_PATH: &str = "M0,0 M0,0 V14 L14,7 Z";
pub(crate) static MARKER_REFERENCE: &str = "url(#arrowHead)";
pub(crate) const MARKER_HEIGHT: CoordinateT = 14;
static MARKER_DIMEN: &'static str = concatcp!(MARKER_HEIGHT);
static MARKER_REF_Y: &'static str = concatcp!(MARKER_HEIGHT.saturating_div(2));

/// Object representation of the marker path (arow).
#[derive(Serialize)]
struct MarkerPath {
    #[serde(rename = "@d")]
    d: &'static str,
    #[serde(rename = "@fill")]
    fill: &'static str,
    #[serde(flatten)]
    attributes: CommonAttributes,
}

impl Default for MarkerPath {
    fn default() -> Self {
        Self {
            d: MARKER_PATH,
            fill: "black",
            attributes: CommonAttributes::with_no_class(),
        }
    }
}

/// Object repersentation of an SVG maker element.
#[derive(Serialize)]
pub(crate) struct Marker {
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
            marker_width: MARKER_DIMEN,
            marker_height: MARKER_DIMEN,
            ref_y: MARKER_REF_Y,
            path: MarkerPath::default(),
        }
    }
}
