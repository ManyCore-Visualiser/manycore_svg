use std::collections::HashMap;

use getset::Getters;
use manycore_parser::{Neighbour, Neighbours};
use serde::{Serialize, Serializer};

use crate::{
    CommonAttributes, Router, CONNECTION_LENGTH, HALF_SIDE_LENGTH, MARKER_REFERENCE,
    OUTPUT_LINK_OFFSET, ROUTER_OFFSET, SIDE_LENGTH,
};

#[derive(Serialize)]
pub struct Connection {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    attributes: CommonAttributes,
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
    pub fn get_path(direction: &ConnectionDirection, r: &u16, c: &u16) -> String {
        let (mut router_x, mut router_y) = Router::get_move_coordinates(r, c);
        router_x += HALF_SIDE_LENGTH;
        router_y += SIDE_LENGTH - ROUTER_OFFSET;
        router_y -= HALF_SIDE_LENGTH;

        let ret: String;

        match direction {
            ConnectionDirection::TOP => {
                router_x += OUTPUT_LINK_OFFSET;
                router_y -= HALF_SIDE_LENGTH;
                ret = format!("M{},{} v-{}", router_x, router_y, CONNECTION_LENGTH);
            }
            ConnectionDirection::RIGHT => {
                router_x += HALF_SIDE_LENGTH;
                router_y -= OUTPUT_LINK_OFFSET;
                ret = format!("M{},{} h{}", router_x, router_y, CONNECTION_LENGTH);
            }
            ConnectionDirection::BOTTOM => {
                router_y += HALF_SIDE_LENGTH;
                ret = format!("M{},{} v{}", router_x, router_y, CONNECTION_LENGTH);
            }
            ConnectionDirection::LEFT => {
                router_x -= HALF_SIDE_LENGTH;
                ret = format!("M{},{} h-{}", router_x, router_y, CONNECTION_LENGTH);
            }
        }

        ret
    }

    pub fn new(connection_id: Option<String>, path: String) -> Self {
        Self {
            id: connection_id,
            d: path,
            attributes: CommonAttributes::with_no_class(),
            marker_end: MARKER_REFERENCE,
        }
    }
}

#[derive(Serialize, Getters)]
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
        neighbour: &Neighbour,
        direction: &ConnectionDirection,
        r: &u16,
        c: &u16,
    ) {
        let connection_id = format!("{}-{}", i, neighbour.id());
        let path = Connection::get_path(direction, &r, &c);
        self.path
            .insert(connection_id.clone(), Connection::new(Some(connection_id), path));
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
                self.add_neighbour(i, top, &ConnectionDirection::TOP, r, c);
            }

            if let Some(right) = neighbours.right() {
                self.add_neighbour(i, right, &ConnectionDirection::RIGHT, r, c);
            }

            if let Some(bottom) = neighbours.bottom() {
                self.add_neighbour(i, bottom, &ConnectionDirection::BOTTOM, r, c);
            }

            if let Some(left) = neighbours.left() {
                self.add_neighbour(i, left, &ConnectionDirection::LEFT, r, c);
            }
        }
    }
}
