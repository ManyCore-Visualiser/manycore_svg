use const_format::concatcp;
use serde::Serialize;

use crate::{CoordinateT, ROUTER_OFFSET, SIDE_LENGTH};

pub(crate) const FREEFORM_CLIP_PATH_ID: &'static str = "crop";

pub(crate) static USE_FREEFORM_CLIP_PATH: &'static str = concatcp!("url(#", FREEFORM_CLIP_PATH_ID, ")");

/// Object representation of an SVG `<polygon>`.
#[derive(Serialize)]
struct Polygon {
    #[serde(rename = "@points")]
    points: String,
}

/// Object representation of an SVG `<clipPath>`.
#[derive(Serialize)]
pub struct ClipPath {
    #[serde(rename = "@id")]
    id: String,
    polygon: Polygon,
}

impl ClipPath {
    /// Creates a new [`ClipPath`] instance given the `points` of a polygon (as [`String`], i.e. already formatted as to be expected in an SVG).
    pub fn new(polygon_points: String) -> Self {
        Self {
            id: FREEFORM_CLIP_PATH_ID.to_string(),
            polygon: Polygon {
                points: polygon_points,
            },
        }
    }

    /// Calculates the id for a core's clip path
    pub(crate) fn make_core_id(id: &u8) -> String {
        format!("clip-c-{id}")
    }

    /// Calculates the id for a router's clip path
    pub(crate) fn make_router_id(id: &u8) -> String {
        format!("clip-r-{id}")
    }

    /// Creates a new [`ClipPath`] instance from a core's position and id. Used to clip information layer.
    pub(crate) fn for_core(id: u8, x: CoordinateT, y: CoordinateT) -> Self {
        let full_y = y.saturating_add(SIDE_LENGTH);
        let full_x = x.saturating_add(SIDE_LENGTH);

        Self {
            id: ClipPath::make_core_id(&id),
            polygon: Polygon {
                points: format!(
                    "{x} {y}, {x} {full_y}, {full_x} {full_y}, {full_x} {}, {} {y}",
                    full_y.saturating_sub(ROUTER_OFFSET),
                    x.saturating_add(ROUTER_OFFSET)
                ),
            },
        }
    }

    /// Creates a new [`ClipPath`] instance from a router's position and id. Used to clip information layer.
    pub(crate) fn for_router(id: u8, x: CoordinateT, y: CoordinateT) -> Self {
        let min_y = y.saturating_sub(ROUTER_OFFSET);
        let full_x = x.saturating_add(SIDE_LENGTH);
        let full_y = min_y.saturating_add(SIDE_LENGTH);

        Self {
            id: ClipPath::make_router_id(&id),
            polygon: Polygon {
                points: format!(
                    "{x} {y}, {x} {min_y}, {full_x} {min_y}, {full_x} {full_y}, {} {full_y}",
                    full_x.saturating_sub(ROUTER_OFFSET),
                ),
            },
        }
    }
}
