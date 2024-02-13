use std::collections::HashMap;

use manycore_parser::Neighbours;
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
    pub fn get_path(direction: ConnectionDirection, r: &u16, c: &u16) -> String {
        let (router_x, router_y) = Router::get_move_coordinates(r, c);
        let router_centre_x = router_x + HALF_SIDE_LENGTH;
        let router_centre_y = router_y + (SIDE_LENGTH - ROUTER_OFFSET) - HALF_SIDE_LENGTH;

        match direction {
            ConnectionDirection::TOP => format!(
                "M{},{} v-{}",
                router_centre_x + OUTPUT_LINK_OFFSET,
                router_centre_y - HALF_SIDE_LENGTH,
                CONNECTION_LENGTH
            ),
            ConnectionDirection::RIGHT => format!(
                "M{},{} h{}",
                router_centre_x + HALF_SIDE_LENGTH,
                router_centre_y - OUTPUT_LINK_OFFSET,
                CONNECTION_LENGTH
            ),
            ConnectionDirection::BOTTOM => format!(
                "M{},{} v{}",
                router_centre_x,
                router_centre_y + HALF_SIDE_LENGTH,
                CONNECTION_LENGTH
            ),
            ConnectionDirection::LEFT => format!(
                "M{},{} h-{}",
                router_centre_x - HALF_SIDE_LENGTH,
                router_centre_y,
                CONNECTION_LENGTH
            ),
        }
    }
}

#[derive(Serialize)]
pub struct ConnectionsParentGroup {
    #[serde(serialize_with = "serialise_map")]
    path: HashMap<String, Connection>,
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
        }
    }
    fn add_neighbour(
        &mut self,
        i: usize,
        neighbour: usize,
        direction: ConnectionDirection,
        r: &u16,
        c: &u16,
    ) {
        let connection_id = format!("{}-{}", i, neighbour);
        self.path.insert(
            connection_id.clone(),
            Connection {
                id: connection_id,
                d: Connection::get_path(direction, &r, &c),
                attributes: CoreRouterCommon::default(),
                marker_end: MARKER_REFERENCE,
            },
        );
    }

    pub fn add_neighbours(
        &mut self,
        i: usize,
        opt_neighbours: Option<&Neighbours>,
        r: &u16,
        c: &u16,
    ) {
        if let Some(neighbours) = opt_neighbours {
            if let Some(top) = neighbours.top() {
                self.add_neighbour(i, *top, ConnectionDirection::TOP, r, c);
            }

            if let Some(right) = neighbours.right() {
                self.add_neighbour(i, *right, ConnectionDirection::RIGHT, r, c);
            }

            if let Some(bottom) = neighbours.bottom() {
                self.add_neighbour(i, *bottom, ConnectionDirection::BOTTOM, r, c);
            }

            if let Some(left) = neighbours.left() {
                self.add_neighbour(i, *left, ConnectionDirection::LEFT, r, c);
            }
        }
    }
}
