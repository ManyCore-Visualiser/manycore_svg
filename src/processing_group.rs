use const_format::concatcp;
use getset::{Getters, MutGetters, Setters};
use serde::Serialize;

use crate::{
    style::BASE_FILL_CLASS_NAME, CoordinateT, SVGError, TextInformation, TopLeft,
    CHAR_HEIGHT_AT_22_PX, CHAR_H_PADDING, CHAR_V_PADDING, CONNECTION_LENGTH, MARKER_HEIGHT,
};

pub const SIDE_LENGTH: CoordinateT = 100;
pub const ROUTER_OFFSET: CoordinateT = SIDE_LENGTH.saturating_div(4).saturating_mul(3);
pub static BLOCK_LENGTH: CoordinateT = SIDE_LENGTH + ROUTER_OFFSET;
pub static HALF_SIDE_LENGTH: CoordinateT = SIDE_LENGTH.saturating_div(2);
pub static HALF_ROUTER_OFFSET: CoordinateT = ROUTER_OFFSET.saturating_div(2);
pub static BLOCK_DISTANCE: CoordinateT = CONNECTION_LENGTH
    .saturating_sub(ROUTER_OFFSET)
    .saturating_add(MARKER_HEIGHT);
static TASK_RECT_X_OFFSET: CoordinateT = 10;

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

pub const CORE_ROUTER_STROKE_WIDTH: CoordinateT = 1;
static CORE_ROUTER_STROKE_WIDTH_STR: &'static str = concatcp!(CORE_ROUTER_STROKE_WIDTH);

pub static TASK_FONT_SIZE: f32 = 22.0;
pub static TASK_RECT_STROKE: CoordinateT = 1;
static TASK_RECT_HEIGHT: CoordinateT = CHAR_HEIGHT_AT_22_PX + CHAR_V_PADDING * 2;
pub static HALF_TASK_RECT_HEIGHT: CoordinateT = TASK_RECT_HEIGHT.saturating_div(2);
pub static TASK_RECT_CENTRE_OFFSET: CoordinateT =
    HALF_TASK_RECT_HEIGHT.saturating_sub(TASK_RECT_HEIGHT.saturating_div(3));
static TASK_RECT_FILL: &'static str = "#bfdbfe";
pub static TASK_BOTTOM_OFFSET: CoordinateT = TASK_RECT_CENTRE_OFFSET
    .saturating_add(HALF_TASK_RECT_HEIGHT)
    .saturating_add(TASK_RECT_STROKE);

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
    x: CoordinateT,
    #[serde(rename = "@y")]
    y: CoordinateT,
    #[serde(rename = "@width")]
    width: CoordinateT,
    #[serde(rename = "@height")]
    height: CoordinateT,
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
    fn new(cx: CoordinateT, cy: CoordinateT, text_width: CoordinateT) -> Self {
        Self {
            x: cx - (text_width.saturating_div(2)),
            y: cy - HALF_TASK_RECT_HEIGHT,
            width: text_width,
            height: TASK_RECT_HEIGHT,
            rx: "15",
            fill: TASK_RECT_FILL,
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
    fn new(
        r: &CoordinateT,
        c: &CoordinateT,
        task: &Option<u16>,
        top_left: &TopLeft,
    ) -> Result<Option<Self>, SVGError> {
        match task {
            Some(task) => {
                let text = format!("T{}", task);

                let text_width = TextInformation::calculate_length_util(
                    TASK_FONT_SIZE,
                    text.len(),
                    Some(CHAR_H_PADDING),
                )?;

                let (cx, cy) = Self::get_centre_coordinates(r, c, text_width, top_left);
                Ok(Some(Self {
                    rect: TaskRect::new(cx, cy, text_width),
                    text: TextInformation::new(
                        cx,
                        cy,
                        TASK_FONT_SIZE,
                        "middle",
                        "central",
                        None,
                        None,
                        text,
                    ),
                }))
            }
            None => Ok(None),
        }
    }

    fn get_centre_coordinates(
        r: &CoordinateT,
        c: &CoordinateT,
        text_width: CoordinateT,
        top_left: &TopLeft,
    ) -> (CoordinateT, CoordinateT) {
        let cx = c * BLOCK_LENGTH + c * BLOCK_DISTANCE - (text_width.saturating_div(2))
            + TASK_RECT_X_OFFSET
            + top_left.x();
        let cy = TASK_RECT_CENTRE_OFFSET
            + r * BLOCK_LENGTH
            + ROUTER_OFFSET
            + SIDE_LENGTH
            + r * BLOCK_DISTANCE
            + top_left.y();

        (cx, cy)
    }
}

#[derive(Serialize, MutGetters, Getters)]
pub struct Router {
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
    pub fn new(r: &CoordinateT, c: &CoordinateT, id: &u8, top_left: &TopLeft) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(r, c, top_left);

        Self {
            move_coordinates: (move_x, move_y),
            id: format!("r{}", id),
            d: format!("M{},{} {}", move_x, move_y, ROUTER_PATH),
            attributes: CommonAttributes::default(),
        }
    }

    pub fn get_move_coordinates(
        r: &CoordinateT,
        c: &CoordinateT,
        top_left: &TopLeft,
    ) -> (CoordinateT, CoordinateT) {
        let move_x = (c * BLOCK_LENGTH)
            + ROUTER_OFFSET
            + c * BLOCK_DISTANCE
            + top_left.x()
            + CORE_ROUTER_STROKE_WIDTH;
        let move_y = r * BLOCK_LENGTH
            + ROUTER_OFFSET
            + r * BLOCK_DISTANCE
            + top_left.y()
            + CORE_ROUTER_STROKE_WIDTH;

        (move_x, move_y)
    }
}

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
    pub fn get_move_coordinates(
        r: &CoordinateT,
        c: &CoordinateT,
        top_left: &TopLeft,
    ) -> (CoordinateT, CoordinateT) {
        let move_x =
            c * BLOCK_LENGTH + c * BLOCK_DISTANCE + top_left.x() + CORE_ROUTER_STROKE_WIDTH;
        let move_y = r * BLOCK_LENGTH
            + ROUTER_OFFSET
            + r * BLOCK_DISTANCE
            + top_left.y()
            + CORE_ROUTER_STROKE_WIDTH;

        (move_x, move_y)
    }
    fn new(r: &CoordinateT, c: &CoordinateT, id: &u8, top_left: &TopLeft) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(r, c, top_left);

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
    coordinates: (CoordinateT, CoordinateT),
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
    pub fn new(
        r: &CoordinateT,
        c: &CoordinateT,
        id: &u8,
        allocated_task: &Option<u16>,
        top_left: &TopLeft,
    ) -> Result<Self, SVGError> {
        Ok(Self {
            coordinates: (*r, *c),
            id: *id,
            core: Core::new(r, c, id, top_left),
            router: Router::new(r, c, id, top_left),
            task: Task::new(r, c, allocated_task, top_left)?,
        })
    }

    pub fn task_start(&self) -> Option<CoordinateT> {
        match &self.task {
            Some(task) => Some(task.rect.x),
            None => None,
        }
    }
}

#[derive(Serialize, MutGetters, Getters)]
#[getset(get_mut = "pub", get = "pub")]
pub struct ProcessingParentGroup {
    #[serde(rename = "@id")]
    id: &'static str,
    g: Vec<ProcessingGroup>,
}

impl ProcessingParentGroup {
    pub fn new(number_of_cores: &usize) -> Self {
        Self {
            id: "processingGroup",
            g: Vec::with_capacity(*number_of_cores),
        }
    }
}
