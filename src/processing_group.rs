use std::collections::BTreeMap;

use const_format::concatcp;
use getset::{Getters, MutGetters, Setters};
use manycore_utils::serialise_btreemap;
use serde::Serialize;

use crate::{
    coordinate,
    style::{BASE_FILL_CLASS_NAME, DEFAULT_FILL},
    TextInformation, CHAR_HEIGHT_AT_22_PX, CHAR_H_PADDING, CHAR_V_PADDING, CHAR_WIDTH_AT_16_PX,
    CHAR_WIDTH_AT_22_PX, CONNECTION_LENGTH, FONT_SIZE_WITH_OFFSET, MARKER_HEIGHT,
};

pub const SIDE_LENGTH: coordinate = 100;
pub const ROUTER_OFFSET: coordinate = SIDE_LENGTH.saturating_div(4).saturating_mul(3);
pub static BLOCK_LENGTH: coordinate = SIDE_LENGTH + ROUTER_OFFSET;
pub static HALF_SIDE_LENGTH: coordinate = SIDE_LENGTH.saturating_div(2);
pub static HALF_ROUTER_OFFSET: coordinate = ROUTER_OFFSET.saturating_div(2);
pub static BLOCK_DISTANCE: coordinate = CONNECTION_LENGTH
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

pub const CORE_ROUTER_STROKE_WIDTH: coordinate = 1;
static CORE_ROUTER_STROKE_WIDTH_STR: &'static str = concatcp!(CORE_ROUTER_STROKE_WIDTH);

static TASK_FONT_SIZE: &'static str = "22px";
static TASK_RECT_STROKE: coordinate = 1;
static TASK_RECT_HEIGHT: coordinate = CHAR_HEIGHT_AT_22_PX + CHAR_V_PADDING * 2;
static TASK_RECT_OFFSET: coordinate = TASK_RECT_HEIGHT.saturating_div(2);

#[derive(Serialize, Setters, Debug)]
pub struct CommonAttributes {
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
    pub fn with_no_class() -> Self {
        Self {
            class: None,
            fill_rule: "evenodd",
            stroke: "black",
            stroke_linecap: "butt",
            stroke_width: CORE_ROUTER_STROKE_WIDTH_STR,
        }
    }
}

#[derive(Serialize)]
struct TaskRect {
    #[serde(rename = "@x")]
    x: coordinate,
    #[serde(rename = "@y")]
    y: coordinate,
    #[serde(rename = "@width")]
    width: coordinate,
    #[serde(rename = "@height")]
    height: coordinate,
    #[serde(rename = "@rx")]
    rx: &'static str,
    #[serde(rename = "@fill")]
    fill: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl TaskRect {
    fn new(cx: coordinate, cy: coordinate, text_width: f32) -> Self {
        Self {
            x: cx - (text_width / 2.0).round() as coordinate,
            y: cy - TASK_RECT_OFFSET,
            width: text_width.round() as coordinate,
            height: TASK_RECT_HEIGHT,
            rx: "15",
            fill: DEFAULT_FILL,
            stroke: "black",
            stroke_width: CORE_ROUTER_STROKE_WIDTH_STR,
        }
    }
}

#[derive(Serialize)]
struct Task {
    rect: TaskRect,
    text: TextInformation,
}

impl Task {
    fn new(r: &coordinate, c: &coordinate, task: &Option<u16>) -> Option<Self> {
        match task {
            Some(task) => {
                let (cx, cy) = Self::get_centre_coordinates(r, c);
                let text = format!("T{}", task);
                let text_width = CHAR_WIDTH_AT_22_PX * u16::try_from(text.len()).unwrap() as f32
                    + CHAR_H_PADDING;
                Some(Self {
                    rect: TaskRect::new(cx, cy, text_width),
                    text: TextInformation::new(
                        cx,
                        cy,
                        Some(TASK_FONT_SIZE),
                        "middle",
                        "middle",
                        None,
                        None,
                        format!("T{}", task),
                    ),
                })
            }
            None => None,
        }
    }

    fn get_centre_coordinates(r: &coordinate, c: &coordinate) -> (coordinate, coordinate) {
        let cx = c * BLOCK_LENGTH + c * BLOCK_DISTANCE;
        let cy = TASK_RECT_OFFSET
            + r * BLOCK_LENGTH
            + ROUTER_OFFSET
            + SIDE_LENGTH
            + TASK_RECT_STROKE
            + r * BLOCK_DISTANCE;

        (cx, cy)
    }
}

#[derive(Serialize, MutGetters, Getters)]
pub struct Router {
    /// Router coordinates, (x, y)
    #[serde(skip)]
    #[getset(get = "pub")]
    move_coordinates: (coordinate, coordinate),
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CommonAttributes,
}

impl Router {
    pub fn new(r: &coordinate, c: &coordinate, id: &u8) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(r, c);

        Self {
            move_coordinates: (move_x, move_y),
            id: format!("r{}", id),
            d: format!("M{},{} {}", move_x, move_y, ROUTER_PATH),
            attributes: CommonAttributes::default(),
        }
    }

    pub fn get_move_coordinates(r: &coordinate, c: &coordinate) -> (coordinate, coordinate) {
        let move_x = (c * BLOCK_LENGTH) + ROUTER_OFFSET + c * BLOCK_DISTANCE;
        let move_y = r * BLOCK_LENGTH + ROUTER_OFFSET + r * BLOCK_DISTANCE;

        (move_x, move_y)
    }
}

#[derive(Serialize, MutGetters, Getters)]
pub struct Core {
    /// Core coordinates, (x, y)
    #[serde(skip)]
    #[getset(get = "pub")]
    move_coordinates: (coordinate, coordinate),
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CommonAttributes,
}

impl Core {
    pub fn get_move_coordinates(r: &coordinate, c: &coordinate) -> (coordinate, coordinate) {
        let move_x = c * BLOCK_LENGTH + c * BLOCK_DISTANCE;
        let move_y = r * BLOCK_LENGTH + ROUTER_OFFSET + r * BLOCK_DISTANCE;

        (move_x, move_y)
    }
    fn new(r: &coordinate, c: &coordinate, id: &u8) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(r, c);

        Self {
            move_coordinates: (move_x, move_y),
            id: format!("c{}", id),
            d: format!("M{},{} {}", move_x, move_y, PROCESSOR_PATH),
            attributes: CommonAttributes::default(),
        }
    }
}

#[derive(Serialize, MutGetters, Getters)]
pub struct ProcessingGroup {
    #[serde(skip)]
    #[getset(get = "pub")]
    /// Coordinates (row, column)
    coordinates: (coordinate, coordinate),
    #[serde(rename = "@id")]
    id: u8,
    #[serde(rename = "path")]
    #[getset(get = "pub", get_mut = "pub")]
    core: Core,
    #[serde(rename = "path")]
    #[getset(get_mut = "pub", get = "pub")]
    router: Router,
    #[serde(rename = "g", skip_serializing_if = "Option::is_none")]
    task: Option<Task>,
}

impl ProcessingGroup {
    pub fn new(r: &coordinate, c: &coordinate, id: &u8, allocated_task: &Option<u16>) -> Self {
        Self {
            coordinates: (*r, *c),
            id: *id,
            core: Core::new(r, c, id),
            router: Router::new(r, c, id),
            task: Task::new(r, c, allocated_task),
        }
    }
}

#[derive(Serialize, MutGetters, Getters)]
#[getset(get_mut = "pub", get = "pub")]
pub struct ProcessingParentGroup {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(serialize_with = "serialise_btreemap")]
    g: BTreeMap<u8, ProcessingGroup>,
}

impl ProcessingParentGroup {
    pub fn new() -> Self {
        Self {
            id: "processingGroup",
            g: BTreeMap::new(),
        }
    }
}
