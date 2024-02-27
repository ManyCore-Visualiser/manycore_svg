use getset::{MutGetters, Setters};
use serde::Serialize;

pub static DEFAULT_FILL: &str = "#e5e5e5";

use crate::{GROUP_DISTANCE, PROCESSOR_PATH, ROUTER_OFFSET, ROUTER_PATH, UNIT_LENGTH};

#[derive(Serialize, Setters)]
pub struct CoreRouterCommon {
    #[serde(rename = "@fill")]
    #[getset(set = "pub")]
    fill: String,
    #[serde(rename = "@fill-rule")]
    fill_rule: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-linecap")]
    stroke_linecap: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl Default for CoreRouterCommon {
    fn default() -> Self {
        Self {
            fill: DEFAULT_FILL.to_string(),
            fill_rule: "evenodd",
            stroke: "black",
            stroke_linecap: "butt",
            stroke_width: "1",
        }
    }
}

#[derive(Serialize, MutGetters)]
pub struct Router {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CoreRouterCommon,
}

impl Router {
    pub fn new(r: &u16, c: &u16, group_id: &String) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(r, c);

        Self {
            id: format!("{}r", group_id),
            d: format!("M{},{} {}", move_x, move_y, ROUTER_PATH),
            attributes: CoreRouterCommon::default(),
        }
    }

    pub fn get_move_coordinates(r: &u16, c: &u16) -> (u16, u16) {
        let move_x =
            (c * UNIT_LENGTH) + ROUTER_OFFSET + if *c == 0 { 0 } else { c * GROUP_DISTANCE };
        let move_y = r * UNIT_LENGTH + ROUTER_OFFSET + if *r == 0 { 0 } else { r * GROUP_DISTANCE };

        (move_x, move_y)
    }
}

#[derive(Serialize, MutGetters)]
pub struct Core {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CoreRouterCommon,
}

impl Core {
    pub fn get_move_coordinates(r: &u16, c: &u16) -> (u16, u16) {
        let move_x = c * UNIT_LENGTH + if *c == 0 { 0 } else { c * GROUP_DISTANCE };
        let move_y = r * UNIT_LENGTH + ROUTER_OFFSET + if *r == 0 { 0 } else { r * GROUP_DISTANCE };

        (move_x, move_y)
    }
    fn new(r: &u16, c: &u16, group_id: &String) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(r, c);

        Self {
            id: format!("{}c", group_id),
            d: format!("M{},{} {}", move_x, move_y, PROCESSOR_PATH),
            attributes: CoreRouterCommon::default(),
        }
    }
}

#[derive(Serialize, MutGetters)]
pub struct ProcessingGroup {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "path")]
    #[getset(get_mut = "pub")]
    core: Core,
    #[serde(rename = "path")]
    #[getset(get_mut = "pub")]
    router: Router,
}

impl ProcessingGroup {
    pub fn new(r: &u16, c: &u16, group_id: &String) -> Self {
        Self {
            id: group_id.clone(),
            core: Core::new(r, c, &group_id),
            router: Router::new(r, c, &group_id),
        }
    }
}

#[derive(Serialize, MutGetters)]
#[getset(get_mut = "pub")]
pub struct ProcessingParentGroup {
    g: Vec<ProcessingGroup>,
}

impl ProcessingParentGroup {
    pub fn new() -> Self {
        Self { g: vec![] }
    }
}