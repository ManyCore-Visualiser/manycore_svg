use const_format::concatcp;
use serde::Serialize;

pub const CLIP_PATH_ID: &'static str = "crop";

pub const USE_CLIP_PATH: &'static str = concatcp!("url(#", CLIP_PATH_ID, ")");

#[derive(Serialize)]
struct Polygon {
    #[serde(rename = "@points")]
    points: String,
}

#[derive(Serialize)]
pub struct ClipPath {
    #[serde(rename = "@id")]
    id: &'static str,
    polygon: Polygon,
}

impl ClipPath {
    pub fn new(polygon_points: String) -> Self {
        Self {
            id: CLIP_PATH_ID,
            polygon: Polygon {
                points: polygon_points,
            },
        }
    }
}
