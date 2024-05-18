use const_format::concatcp;
use getset::{Getters, MutGetters, Setters};
use manycore_parser::ElementIDT;
use serde::Serialize;

use crate::{
    style::BASE_FILL_CLASS_NAME, ClipPath, CoordinateT, SVGError, TopLeft, CONNECTION_LENGTH,
    MARKER_HEIGHT, USE_FREEFORM_CLIP_PATH,
};

pub(crate) const SIDE_LENGTH: CoordinateT = 100;
pub(crate) const ROUTER_OFFSET: CoordinateT = SIDE_LENGTH.saturating_div(4).saturating_mul(3);
pub(crate) static BLOCK_LENGTH: CoordinateT = SIDE_LENGTH + ROUTER_OFFSET;
pub(crate) static HALF_SIDE_LENGTH: CoordinateT = SIDE_LENGTH.saturating_div(2);
pub(crate) static HALF_ROUTER_OFFSET: CoordinateT = ROUTER_OFFSET.saturating_div(2);
pub(crate) static BLOCK_DISTANCE: CoordinateT = CONNECTION_LENGTH
    .saturating_sub(ROUTER_OFFSET)
    .saturating_add(MARKER_HEIGHT);

// Example after concatenation with SIDE_LENGTH = 100 -> ROUTER_OFFSET = 75
// l0,100 l100,0 l0,-75 l-25,-25 l-75,0 Z
static PROCESSOR_PATH: &'static str = concatcp!(
    "l0,",
    SIDE_LENGTH,
    " l",
    SIDE_LENGTH,
    ",0 l0,-",
    ROUTER_OFFSET,
    " l-",
    SIDE_LENGTH - ROUTER_OFFSET,
    ",-",
    SIDE_LENGTH - ROUTER_OFFSET,
    " l-",
    ROUTER_OFFSET,
    ",0 Z"
);

// Example after concatenation with SIDE_LENGTH = 100 -> ROUTER_OFFSET = 75
// l0,-75 l100,0 l0,100 l-75,0 Z
static ROUTER_PATH: &'static str = concatcp!(
    "l0,-",
    ROUTER_OFFSET,
    " l",
    SIDE_LENGTH,
    ",0 l0,",
    SIDE_LENGTH,
    " l-",
    ROUTER_OFFSET,
    ",0 Z"
);

pub(crate) const CORE_ROUTER_STROKE_WIDTH: CoordinateT = 1;
pub(crate) static CORE_ROUTER_STROKE_WIDTH_STR: &'static str = concatcp!(CORE_ROUTER_STROKE_WIDTH);

/// Wrapper around attributes shared by different elements.
#[derive(Serialize, Setters, Debug)]
pub(crate) struct CommonAttributes {
    #[serde(rename = "@class", skip_serializing_if = "Option::is_none")]
    class: Option<&'static str>,
    #[serde(rename = "@fill-rule")]
    fill_rule: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-linecap")]
    stroke_linecap: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl Default for CommonAttributes {
    fn default() -> Self {
        Self {
            class: Some(BASE_FILL_CLASS_NAME),
            fill_rule: "evenodd",
            stroke: "black",
            stroke_linecap: "butt",
            stroke_width: CORE_ROUTER_STROKE_WIDTH_STR,
        }
    }
}

impl CommonAttributes {
    /// Generates  a [`CommonAttributes`] instance with no class.
    pub(crate) fn with_no_class() -> Self {
        Self {
            class: None,
            fill_rule: "evenodd",
            stroke: "black",
            stroke_linecap: "butt",
            stroke_width: CORE_ROUTER_STROKE_WIDTH_STR,
        }
    }
}

/// Object representattion of the SVG `<path>` that makes up a router.
#[derive(Serialize, MutGetters, Getters)]
pub(crate) struct Router {
    /// Router coordinates, (x, y)
    #[serde(skip)]
    #[getset(get = "pub")]
    move_coordinates: (CoordinateT, CoordinateT),
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CommonAttributes,
}

impl Router {
    /// Generates a new [`Router`] instance from the given parameters.
    pub(crate) fn new(
        row: &CoordinateT,
        column: &CoordinateT,
        id: &ElementIDT,
        top_left: &TopLeft,
    ) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(row, column, top_left);

        Self {
            move_coordinates: (move_x, move_y),
            id: format!("r{}", id),
            d: format!("M{},{} {}", move_x, move_y, ROUTER_PATH),
            attributes: CommonAttributes::default(),
        }
    }

    /// Calculates the move coordinates for a [`Router`] path given current row, column and the viewBox [`TopLeft`].
    pub(crate) fn get_move_coordinates(
        row: &CoordinateT,
        column: &CoordinateT,
        top_left: &TopLeft,
    ) -> (CoordinateT, CoordinateT) {
        let move_x = (column * BLOCK_LENGTH)
            + ROUTER_OFFSET
            + column * BLOCK_DISTANCE
            + top_left.x()
            + CORE_ROUTER_STROKE_WIDTH;
        let move_y = row * BLOCK_LENGTH
            + ROUTER_OFFSET
            + row * BLOCK_DISTANCE
            + top_left.y()
            + CORE_ROUTER_STROKE_WIDTH;

        (move_x, move_y)
    }
}

/// Object representattion of the SVG `<path>` that makes up a core.
#[derive(Serialize, MutGetters, Getters)]
pub struct Core {
    /// Core coordinates, (x, y)
    #[serde(skip)]
    #[getset(get = "pub")]
    move_coordinates: (CoordinateT, CoordinateT),
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CommonAttributes,
}

impl Core {
    /// Calculates the move coordinates for a [`Core`] path given current row, column and the viewBox [`TopLeft`].
    pub(crate) fn get_move_coordinates(
        row: &CoordinateT,
        column: &CoordinateT,
        top_left: &TopLeft,
    ) -> (CoordinateT, CoordinateT) {
        let move_x = column * BLOCK_LENGTH
            + column * BLOCK_DISTANCE
            + top_left.x()
            + CORE_ROUTER_STROKE_WIDTH;
        let move_y = row * BLOCK_LENGTH
            + ROUTER_OFFSET
            + row * BLOCK_DISTANCE
            + top_left.y()
            + CORE_ROUTER_STROKE_WIDTH;

        (move_x, move_y)
    }

    /// Generates a new [`Core`] instance from the given parameters.
    fn new(row: &CoordinateT, column: &CoordinateT, id: &ElementIDT, top_left: &TopLeft) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(row, column, top_left);

        Self {
            move_coordinates: (move_x, move_y),
            id: format!("c{}", id),
            d: format!("M{},{} {}", move_x, move_y, PROCESSOR_PATH),
            attributes: CommonAttributes::default(),
        }
    }
}

/// Object representation of an SVG `<g>` that contains a [`Core`], [`Router`] and [`Task`].
#[derive(Serialize, MutGetters, Getters)]
pub(crate) struct ProcessingGroup {
    #[serde(skip)]
    #[getset(get = "pub")]
    /// Coordinates (row, column)
    coordinates: (CoordinateT, CoordinateT),
    #[serde(rename = "@id")]
    id: ElementIDT,
    #[serde(rename = "path")]
    #[getset(get = "pub")]
    core: Core,
    #[serde(rename = "path")]
    #[getset(get = "pub")]
    router: Router,
}

impl ProcessingGroup {
    /// Generates a new [`ProcessingGroup`] instance from the given parameters.
    pub(crate) fn new(
        row: &CoordinateT,
        column: &CoordinateT,
        id: &ElementIDT,
        top_left: &TopLeft,
        clip_paths: &mut Vec<ClipPath>,
    ) -> Result<Self, SVGError> {
        // Core
        let core = Core::new(row, column, id, top_left);
        let (core_x, core_y) = core.move_coordinates;
        // Core clip path
        let core_clip = ClipPath::for_core(*id, core_x, core_y);
        clip_paths.push(core_clip);

        // Router
        let router = Router::new(row, column, id, top_left);
        let (router_x, router_y) = router.move_coordinates;
        // Router clip path
        let router_clip = ClipPath::for_router(*id, router_x, router_y);
        clip_paths.push(router_clip);

        Ok(Self {
            coordinates: (*row, *column),
            id: *id,
            core,
            router,
        })
    }
}

/// An SVG `<g>` that wraps all [`ProcessingGroup`] instances.
#[derive(Serialize, MutGetters, Getters)]
#[getset(get_mut = "pub", get = "pub")]
pub(crate) struct ProcessingParentGroup {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "@clip-path")]
    clip_path: &'static str,
    g: Vec<ProcessingGroup>,
}

impl ProcessingParentGroup {
    /// Generates a new [`ProcessingParentGroup`] with capacity for `number_of_cores` instances of [`ProcessingGroup`]s.
    pub(crate) fn new(number_of_cores: &usize) -> Self {
        Self {
            id: "processingGroup",
            g: Vec::with_capacity(*number_of_cores),
            clip_path: USE_FREEFORM_CLIP_PATH,
        }
    }
}
