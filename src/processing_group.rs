use getset::{MutGetters, Setters};
use serde::Serialize;

use crate::{
    style::BASE_FILL_CLASS_NAME, GROUP_DISTANCE, PROCESSOR_PATH, ROUTER_OFFSET, ROUTER_PATH,
    UNIT_LENGTH,
};

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

#[derive(Serialize, MutGetters)]
pub struct Router {
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
            id: format!("r{}", id),
            d: format!("M{},{} {}", move_x, move_y, ROUTER_PATH),
            attributes: CommonAttributes::default(),
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
    attributes: CommonAttributes,
}

impl Core {
    pub fn get_move_coordinates(r: &u16, c: &u16) -> (u16, u16) {
        let move_x = c * UNIT_LENGTH + if *c == 0 { 0 } else { c * GROUP_DISTANCE };
        let move_y = r * UNIT_LENGTH + ROUTER_OFFSET + if *r == 0 { 0 } else { r * GROUP_DISTANCE };

        (move_x, move_y)
    }
    fn new(r: &u16, c: &u16, id: &u8) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(r, c);

        Self {
            id: format!("c{}", id),
            d: format!("M{},{} {}", move_x, move_y, PROCESSOR_PATH),
            attributes: CommonAttributes::default(),
        }
    }
}

#[derive(Serialize, MutGetters)]
pub struct ProcessingGroup {
    #[serde(rename = "@id")]
    id: u8,
    #[serde(rename = "path")]
    #[getset(get_mut = "pub")]
    core: Core,
    #[serde(rename = "path")]
    #[getset(get_mut = "pub")]
    router: Router,
}

impl ProcessingGroup {
    pub fn new(r: &u16, c: &u16, id: &u8) -> Self {
        Self {
            id: *id,
            core: Core::new(r, c, id),
            router: Router::new(r, c, id),
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
