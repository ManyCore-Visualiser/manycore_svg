use const_format::concatcp;
use serde::Serialize;

pub(crate) const CLIP_PATH_ID: &'static str = "crop";

pub(crate) static USE_CLIP_PATH: &'static str = concatcp!("url(#", CLIP_PATH_ID, ")");

/// Object representation of an SVG `<polygon>`.
#[derive(Serialize)]
struct Polygon {
    #[serde(rename = "@points")]
    points: String,
}

/// Object representation of an SVG `<clip-path>`.
#[derive(Serialize)]
pub struct ClipPath {
    #[serde(rename = "@id")]
    id: &'static str,
    polygon: Polygon,
}

impl ClipPath {
    /// Creates a new [`ClipPath`] instance given the `points` of a polygon (as [`String`], i.e. already formatted as to be expected in an SVG).
    pub fn new(polygon_points: String) -> Self {
        Self {
            id: CLIP_PATH_ID,
            polygon: Polygon {
                points: polygon_points,
            },
        }
    }
}
