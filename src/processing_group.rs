use std::collections::BTreeMap;

use getset::{Getters, MutGetters, Setters};
use serde::{Serialize, Serializer};

use crate::{
    style::{BASE_FILL_CLASS_NAME, DEFAULT_FILL},
    TextInformation, GROUP_DISTANCE, PROCESSOR_PATH, ROUTER_OFFSET, ROUTER_PATH, SIDE_LENGTH,
    UNIT_LENGTH,
};

static TASK_CIRCLE_OFFSET: u16 = 10;
static TASK_CIRCLE_RADIUS: u16 = 20;
static TASK_CIRCLE_STROKE: u16 = 1;

pub static TASK_CIRCLE_TOTAL_OFFSET: u16 =
    TASK_CIRCLE_OFFSET + TASK_CIRCLE_RADIUS + TASK_CIRCLE_STROKE;

#[derive(Serialize, Setters)]
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
            stroke_width: "1",
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
            stroke_width: "1",
        }
    }
}

#[derive(Serialize)]
struct Circle {
    #[serde(rename = "@cx")]
    cx: u16,
    #[serde(rename = "@cy")]
    cy: u16,
    #[serde(rename = "@r")]
    r: &'static str,
    #[serde(rename = "@fill")]
    fill: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl Circle {
    fn new(cx: u16, cy: u16) -> Self {
        Self {
            cx,
            cy,
            r: "20",
            fill: DEFAULT_FILL,
            stroke: "black",
            stroke_width: "1",
        }
    }
}

#[derive(Serialize)]
struct Task {
    circle: Circle,
    text: TextInformation,
}

impl Task {
    fn new(r: &u16, c: &u16, task: &Option<u16>) -> Option<Self> {
        match task {
            Some(task) => {
                let (cx, cy) = Self::get_centre_coordinates(r, c);
                Some(Self {
                    circle: Circle::new(cx, cy),
                    text: TextInformation::new(
                        cx,
                        cy,
                        "middle",
                        "middle",
                        None,
                        format!("T{}", task),
                    ),
                })
            }
            None => None,
        }
    }

    fn get_centre_coordinates(r: &u16, c: &u16) -> (u16, u16) {
        let cx = c * UNIT_LENGTH
            + TASK_CIRCLE_RADIUS
            + TASK_CIRCLE_STROKE
            + if *c == 0 { 0 } else { c * GROUP_DISTANCE };
        let cy = r * UNIT_LENGTH
            + ROUTER_OFFSET
            + SIDE_LENGTH
            + TASK_CIRCLE_OFFSET
            + TASK_CIRCLE_STROKE
            + if *r == 0 { 0 } else { r * GROUP_DISTANCE };

        (cx, cy)
    }
}

#[derive(Serialize, MutGetters, Getters)]
pub struct Router {
    /// Router coordinates, (x, y)
    #[serde(skip)]
    #[getset(get = "pub")]
    move_coordinates: (u16, u16),
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CommonAttributes,
}

impl Router {
    pub fn new(r: &u16, c: &u16, id: &u8) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(r, c);

        Self {
            move_coordinates: (move_x, move_y),
            id: format!("r{}", id),
            d: format!("M{},{} {}", move_x, move_y, ROUTER_PATH),
            attributes: CommonAttributes::default(),
        }
    }

    pub fn get_move_coordinates(r: &u16, c: &u16) -> (u16, u16) {
        let move_x = (c * UNIT_LENGTH)
            + ROUTER_OFFSET
            + TASK_CIRCLE_TOTAL_OFFSET
            + if *c == 0 { 0 } else { c * GROUP_DISTANCE };
        let move_y = r * UNIT_LENGTH + ROUTER_OFFSET + if *r == 0 { 0 } else { r * GROUP_DISTANCE };

        (move_x, move_y)
    }
}

#[derive(Serialize, MutGetters, Getters)]
pub struct Core {
    /// Core coordinates, (x, y)
    #[serde(skip)]
    #[getset(get = "pub")]
    move_coordinates: (u16, u16),
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CommonAttributes,
}

impl Core {
    pub fn get_move_coordinates(r: &u16, c: &u16) -> (u16, u16) {
        let move_x = c * UNIT_LENGTH
            + TASK_CIRCLE_TOTAL_OFFSET
            + if *c == 0 { 0 } else { c * GROUP_DISTANCE };
        let move_y = r * UNIT_LENGTH + ROUTER_OFFSET + if *r == 0 { 0 } else { r * GROUP_DISTANCE };

        (move_x, move_y)
    }
    fn new(r: &u16, c: &u16, id: &u8) -> Self {
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
    coordinates: (u16, u16),
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
    pub fn new(r: &u16, c: &u16, id: &u8, allocated_task: &Option<u16>) -> Self {
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
    #[serde(serialize_with = "ProcessingParentGroup::serialize_group")]
    g: BTreeMap<u8, ProcessingGroup>,
}

impl ProcessingParentGroup {
    pub fn new() -> Self {
        Self {
            id: "processingGroup",
            g: BTreeMap::new(),
        }
    }

    fn serialize_group<S: Serializer>(
        map: &BTreeMap<u8, ProcessingGroup>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(map.values())
    }
}
