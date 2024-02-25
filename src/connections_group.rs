use std::collections::HashMap;

use manycore_parser::{Neighbour, Neighbours, RoutingAlgorithms};
use serde::{Serialize, Serializer};

use crate::{
    CoreRouterCommon, Router, CONNECTION_LENGTH, HALF_SIDE_LENGTH, MARKER_REFERENCE,
    OUTPUT_LINK_OFFSET, ROUTER_OFFSET, SIDE_LENGTH,
};

#[derive(Serialize)]
pub struct Connection {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    attributes: CoreRouterCommon,
    #[serde(rename = "@marker-end")]
    marker_end: &'static str,
}

pub enum ConnectionDirection {
    TOP,
    RIGHT,
    BOTTOM,
    LEFT,
}

impl Connection {
    pub fn get_path(direction: &ConnectionDirection, r: &u16, c: &u16) -> (String, u16, u16) {
        let (mut router_x, mut router_y) = Router::get_move_coordinates(r, c);
        router_x += HALF_SIDE_LENGTH;
        router_y += SIDE_LENGTH - ROUTER_OFFSET;
        router_y -= HALF_SIDE_LENGTH;

        let ret_string: String;

        match direction {
            ConnectionDirection::TOP => {
                router_x += OUTPUT_LINK_OFFSET;
                router_y -= HALF_SIDE_LENGTH;
                ret_string = format!("M{},{} v-{}", router_x, router_y, CONNECTION_LENGTH);
            }
            ConnectionDirection::RIGHT => {
                router_x += HALF_SIDE_LENGTH;
                router_y -= OUTPUT_LINK_OFFSET;
                ret_string = format!("M{},{} h{}", router_x, router_y, CONNECTION_LENGTH);
            }
            ConnectionDirection::BOTTOM => {
                router_y += HALF_SIDE_LENGTH;
                ret_string = format!("M{},{} v{}", router_x, router_y, CONNECTION_LENGTH);
            }
            ConnectionDirection::LEFT => {
                router_x -= HALF_SIDE_LENGTH;
                ret_string = format!("M{},{} h-{}", router_x, router_y, CONNECTION_LENGTH);
            }
        }

        (ret_string, router_x, router_y)
    }
}

#[derive(Serialize)]
struct ConnectionLoad {
    #[serde(rename = "@x")]
    x: u16,
    #[serde(rename = "@y")]
    y: u16,
    #[serde(rename = "@dominant-baseline", skip_serializing_if = "Option::is_none")]
    dominant_baseline: Option<&'static str>,
    #[serde(rename = "@text-anchor", skip_serializing_if = "Option::is_none")]
    text_anchor: Option<&'static str>,
    #[serde(rename = "$text")]
    content: u8,
}

impl ConnectionLoad {
    fn from_router_coordinates(
        direction: &ConnectionDirection,
        router_x: u16,
        router_y: u16,
        link_cost: &u8,
    ) -> Self {
        match direction {
            ConnectionDirection::TOP => ConnectionLoad {
                x: router_x + 5,
                y: router_y - 175,
                dominant_baseline: Some("text-before-edge"),
                text_anchor: None,
                content: *link_cost,
            },
            ConnectionDirection::RIGHT => ConnectionLoad {
                x: router_x + 175,
                y: router_y,
                dominant_baseline: Some("text-after-edge"),
                text_anchor: Some("end"),
                content: *link_cost,
            },
            ConnectionDirection::BOTTOM => ConnectionLoad {
                x: router_x - 5,
                y: router_y + 175,
                dominant_baseline: Some("text-after-edge"),
                text_anchor: Some("end"),
                content: *link_cost,
            },
            ConnectionDirection::LEFT => ConnectionLoad {
                x: router_x - 175,
                y: router_y,
                dominant_baseline: Some("text-before-edge"),
                text_anchor: None,
                content: *link_cost,
            },
        }
    }
}

#[derive(Serialize)]
pub struct ConnectionsParentGroup {
    #[serde(serialize_with = "serialise_map")]
    path: HashMap<String, Connection>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    text: Vec<ConnectionLoad>,
}

fn serialise_map<S>(map: &HashMap<String, Connection>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    // TODO: Clean this up without unwrap
    let entries: Vec<&Connection> = keys.iter().map(|k| map.get(*k).unwrap()).collect();

    entries.serialize(serializer)
}

impl ConnectionsParentGroup {
    pub fn new() -> Self {
        Self {
            path: HashMap::new(),
            text: Vec::new(),
        }
    }
    fn add_neighbour(
        &mut self,
        i: usize,
        neighbour: &Neighbour,
        direction: &ConnectionDirection,
        r: &u16,
        c: &u16,
        algorithm: &Option<&RoutingAlgorithms>,
    ) {
        let connection_id = format!("{}-{}", i, neighbour.id());
        let (path, router_x, router_y) = Connection::get_path(direction, &r, &c);
        self.path.insert(
            connection_id.clone(),
            Connection {
                id: connection_id,
                d: path,
                attributes: CoreRouterCommon::default(),
                marker_end: MARKER_REFERENCE,
            },
        );

        if let Some(_) = algorithm {
            self.text.push(ConnectionLoad::from_router_coordinates(
                direction,
                router_x,
                router_y,
                neighbour.link_cost(),
            ))
        }
    }

    pub fn add_neighbours(
        &mut self,
        i: usize,
        opt_neighbours: Option<&Neighbours>,
        r: &u16,
        c: &u16,
        algorithm: &Option<&RoutingAlgorithms>,
    ) {
        if let Some(neighbours) = opt_neighbours {
            if let Some(top) = neighbours.top() {
                self.add_neighbour(i, top, &ConnectionDirection::TOP, r, c, algorithm);
            }

            if let Some(right) = neighbours.right() {
                self.add_neighbour(i, right, &ConnectionDirection::RIGHT, r, c, algorithm);
            }

            if let Some(bottom) = neighbours.bottom() {
                self.add_neighbour(i, bottom, &ConnectionDirection::BOTTOM, r, c, algorithm);
            }

            if let Some(left) = neighbours.left() {
                self.add_neighbour(i, left, &ConnectionDirection::LEFT, r, c, algorithm);
            }
        }
    }
}
