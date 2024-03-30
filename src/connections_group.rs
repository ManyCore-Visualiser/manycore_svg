use std::collections::{BTreeMap, HashMap};

use getset::Getters;
use manycore_parser::{Core, Directions, EdgePosition, WithXMLAttributes};
use serde::Serialize;

use crate::{
    style::EDGE_DATA_CLASS_NAME, CommonAttributes, Router, CONNECTION_LENGTH, HALF_ROUTER_OFFSET,
    MARKER_HEIGHT, MARKER_REFERENCE, ROUTER_OFFSET, SIDE_LENGTH, SVG,
};

// static I_SINKS_SOURCE_CONNECTION_SPACING: i32 = 15;
pub static I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH: i32 = 100;
pub const EDGE_CONNECTIONS_ID: &'static str = "edgeConnetions";
static CONNECTION_GAP: u16 = 0u16
    .saturating_add(HALF_ROUTER_OFFSET)
    .saturating_mul(3)
    .div_ceil(4)
    .wrapping_sub(5);
static I_CONNECTION_GAP: i32 = 0i32.saturating_add_unsigned(CONNECTION_GAP as u32);

#[derive(Serialize, Getters, Debug)]
pub struct Connection {
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    attributes: CommonAttributes,
    #[serde(rename = "@marker-end")]
    marker_end: &'static str,
    #[serde(skip)]
    #[getset(get = "pub")]
    x: i32,
    #[serde(skip)]
    #[getset(get = "pub")]
    y: i32,
}

// pub enum ConnectionDirection {
//     TOP,
//     RIGHT,
//     BOTTOM,
//     LEFT,
// }

struct ConnectionPath {
    path: String,
    x: i32,
    y: i32,
}

struct EdgePath {
    input: ConnectionPath,
    output: ConnectionPath,
}

impl Connection {
    fn get_inner_path(direction: &Directions, r: &u16, c: &u16) -> ConnectionPath {
        let (mut router_x, mut router_y) = Router::get_move_coordinates(r, c);

        let path: String;

        match direction {
            Directions::North => {
                router_x = router_x + SIDE_LENGTH - HALF_ROUTER_OFFSET + CONNECTION_GAP;
                router_y = router_y - ROUTER_OFFSET;
                path = format!("M{},{} v-{}", router_x, router_y, CONNECTION_LENGTH);
            }
            Directions::East => {
                router_x = router_x + SIDE_LENGTH;
                router_y = router_y - HALF_ROUTER_OFFSET - CONNECTION_GAP;
                path = format!("M{},{} h{}", router_x, router_y, CONNECTION_LENGTH);
            }
            Directions::South => {
                router_x = router_x + SIDE_LENGTH - HALF_ROUTER_OFFSET - CONNECTION_GAP;
                router_y = router_y - ROUTER_OFFSET + SIDE_LENGTH;
                path = format!("M{},{} v{}", router_x, router_y, CONNECTION_LENGTH);
            }
            Directions::West => {
                router_y = router_y - HALF_ROUTER_OFFSET + CONNECTION_GAP;
                path = format!("M{},{} h-{}", router_x, router_y, CONNECTION_LENGTH);
            }
        }

        ConnectionPath {
            path,
            x: router_x.into(),
            y: router_y.into(),
        }
    }

    fn get_edge_paths(direction: &Directions, r: &u16, c: &u16) -> EdgePath {
        let (router_x, router_y) = Router::get_move_coordinates(r, c);
        let mut start_x = i32::from(router_x);
        let mut start_y = i32::from(router_y);

        let (input, output) = match direction {
            Directions::North => {
                start_x = start_x
                    .wrapping_add_unsigned(SIDE_LENGTH.into())
                    .wrapping_sub_unsigned(HALF_ROUTER_OFFSET.into());
                start_y = start_y.wrapping_sub_unsigned(ROUTER_OFFSET.into());

                let connection_length = I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                    .wrapping_add_unsigned(MARKER_HEIGHT.into());
                let render_length = I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH;

                // Input
                let input_x = start_x - I_CONNECTION_GAP;
                let input_y = start_y - connection_length;
                let input_s = format!("M{},{} v{}", input_x, input_y, render_length);

                // Output
                let output_x = start_x + I_CONNECTION_GAP;
                let output_s = format!("M{},{} v-{}", output_x, start_y, render_length);

                (
                    ConnectionPath {
                        path: input_s,
                        x: input_x,
                        y: input_y,
                    },
                    ConnectionPath {
                        path: output_s,
                        x: output_x,
                        y: start_y,
                    },
                )
            }
            Directions::East => {
                start_x = start_x.wrapping_add_unsigned(SIDE_LENGTH.into());
                start_y = start_y
                    .wrapping_sub_unsigned(ROUTER_OFFSET.into())
                    .wrapping_add_unsigned(HALF_ROUTER_OFFSET.into());

                let connection_length = I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                    .wrapping_add_unsigned(MARKER_HEIGHT.into());
                let render_length = I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH;

                // Input
                let input_x = start_x + connection_length;
                let input_y = start_y - I_CONNECTION_GAP;
                let input_s = format!("M{},{} h-{}", input_x, input_y, render_length);

                // Output
                let output_y = start_y + I_CONNECTION_GAP;
                let output_s = format!("M{},{} h{}", start_x, output_y, render_length);

                (
                    ConnectionPath {
                        path: input_s,
                        x: input_x,
                        y: input_y,
                    },
                    ConnectionPath {
                        path: output_s,
                        x: start_x,
                        y: output_y,
                    },
                )
            }
            Directions::South => {
                start_x = start_x
                    .wrapping_add_unsigned(SIDE_LENGTH.into())
                    .wrapping_sub_unsigned(HALF_ROUTER_OFFSET.into());
                start_y = start_y
                    .wrapping_sub_unsigned(ROUTER_OFFSET.into())
                    .wrapping_add_unsigned(SIDE_LENGTH.into());

                let render_length = I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                    .wrapping_add_unsigned(ROUTER_OFFSET.into());
                let connection_length = render_length.wrapping_add_unsigned(MARKER_HEIGHT.into());

                // Input
                let input_x = start_x + I_CONNECTION_GAP;
                let input_y = start_y + connection_length;
                let input_s = format!("M{},{} v-{}", input_x, input_y, render_length);

                // Output
                let output_x = start_x - I_CONNECTION_GAP;
                let output_s = format!("M{},{} v{}", output_x, start_y, render_length);

                (
                    ConnectionPath {
                        path: input_s,
                        x: input_x,
                        y: input_y,
                    },
                    ConnectionPath {
                        path: output_s,
                        x: output_x,
                        y: start_y,
                    },
                )
            }
            Directions::West => {
                start_y = start_y
                    .wrapping_sub_unsigned(ROUTER_OFFSET.into())
                    .wrapping_add_unsigned(HALF_ROUTER_OFFSET.into());

                let render_length = I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                    .wrapping_add_unsigned(ROUTER_OFFSET.into());
                let connection_length = render_length.wrapping_add_unsigned(MARKER_HEIGHT.into());

                // Input
                let input_x = start_x - connection_length;
                let input_y = start_y - I_CONNECTION_GAP;
                let input_s = format!("M{},{} h{}", input_x, input_y, render_length);

                // Output
                let output_y = start_y + I_CONNECTION_GAP;
                let output_s = format!("M{},{} h-{}", start_x, output_y, render_length);

                (
                    ConnectionPath {
                        path: input_s,
                        x: input_x,
                        y: input_y,
                    },
                    ConnectionPath {
                        path: output_s,
                        x: start_x,
                        y: output_y,
                    },
                )
            }
        };

        EdgePath { input, output }
    }

    fn new(connection_path: ConnectionPath) -> Self {
        Self {
            d: connection_path.path,
            attributes: CommonAttributes::with_no_class(),
            marker_end: MARKER_REFERENCE,
            x: connection_path.x,
            y: connection_path.y,
        }
    }
}

#[derive(Serialize, Default, Getters, Debug)]
pub struct Connections {
    #[getset(get = "pub")]
    path: Vec<Connection>,
}

#[derive(Serialize, Getters)]
pub struct EdgeConnections {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "@class")]
    class: &'static str,
    #[getset(get = "pub")]
    path: Vec<Connection>,
}

impl Default for EdgeConnections {
    fn default() -> Self {
        Self {
            id: EDGE_CONNECTIONS_ID,
            class: EDGE_DATA_CLASS_NAME,
            path: Vec::new(),
        }
    }
}

pub enum ConnectionType {
    EdgeConnection(usize),
    Connection(usize),
}

#[derive(Serialize, Getters, Default)]
#[getset(get = "pub")]
pub struct ConnectionsParentGroup {
    #[serde(rename = "g")]
    connections: Connections,
    #[serde(rename = "g")]
    edge_connections: EdgeConnections,
    #[serde(skip)]
    core_connections_map: HashMap<u8, HashMap<Directions, ConnectionType>>,
}

impl ConnectionsParentGroup {
    fn insert_in_map(&mut self, core_id: &u8, direction: &Directions, element: ConnectionType) {
        // Each core has 4 connections
        self.core_connections_map
            .entry(*core_id)
            .or_insert(HashMap::with_capacity(4))
            .insert(*direction, element);
    }

    fn add_edge_connection(&mut self, core_id: &u8, direction: &Directions, r: &u16, c: &u16) {
        let EdgePath { input, output } = Connection::get_edge_paths(direction, r, c);
        let current_size = self.edge_connections.path.len();

        self.edge_connections.path.push(Connection::new(input));
        self.edge_connections.path.push(Connection::new(output));

        // TMP: Iggnoring input
        // self.insert_in_map(
        //     core_id,
        //     direction,
        //     ConnectionType::EdgeConnection(current_size),
        // );
        self.insert_in_map(
            core_id,
            direction,
            ConnectionType::EdgeConnection(current_size + 1),
        );
    }

    fn add_inner_connection(&mut self, core_id: &u8, direction: &Directions, r: &u16, c: &u16) {
        let path = Connection::get_inner_path(direction, &r, &c);
        let current_size = self.connections.path.len();

        self.connections.path.push(Connection::new(path));

        self.insert_in_map(core_id, direction, ConnectionType::Connection(current_size));
    }

    pub fn add_connections(&mut self, core: &Core, r: &u16, c: &u16, columns: u8, rows: u8) {
        let on_edge = core.is_on_edge(columns, rows);

        for direction in core.channels().channel().keys() {
            if let Some(edge_position) = on_edge.as_ref() {
                match direction {
                    Directions::North => match edge_position {
                        EdgePosition::Top | EdgePosition::TopLeft | EdgePosition::TopRight => {
                            self.add_edge_connection(core.id(), direction, r, c)
                        }
                        _ => self.add_inner_connection(core.id(), direction, r, c),
                    },
                    Directions::East => match edge_position {
                        EdgePosition::Right
                        | EdgePosition::TopRight
                        | EdgePosition::BottomRight => {
                            self.add_edge_connection(core.id(), direction, r, c)
                        }
                        _ => self.add_inner_connection(core.id(), direction, r, c),
                    },
                    Directions::South => match edge_position {
                        EdgePosition::Bottom
                        | EdgePosition::BottomLeft
                        | EdgePosition::BottomRight => {
                            self.add_edge_connection(core.id(), direction, r, c)
                        }
                        _ => self.add_inner_connection(core.id(), direction, r, c),
                    },
                    Directions::West => match edge_position {
                        EdgePosition::Left | EdgePosition::TopLeft | EdgePosition::BottomLeft => {
                            self.add_edge_connection(core.id(), direction, r, c)
                        }
                        _ => self.add_inner_connection(core.id(), direction, r, c),
                    },
                }
            } else {
                self.add_inner_connection(core.id(), direction, r, c);
            }
        }
    }
}
