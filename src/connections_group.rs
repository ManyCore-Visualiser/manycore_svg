use manycore_parser::{Core, Directions, EdgePosition};
use serde::Serialize;

use crate::{
    CommonAttributes, Router, CONNECTION_LENGTH, HALF_ROUTER_OFFSET, HALF_SIDE_LENGTH,
    MARKER_HEIGHT, MARKER_REFERENCE, OUTPUT_LINK_OFFSET, ROUTER_OFFSET, SIDE_LENGTH,
};

static I_SINKS_SOURCE_CONNECTION_SPACING: i32 = 15;
static I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH: i32 = 100;
pub const EDGE_CONNECTIONS_ID: &'static str = "edgeConnetions";

#[derive(Serialize)]
pub struct Connection {
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

struct EdgePath {
    input: String,
    output: String,
}

impl Connection {
    fn get_inner_path(direction: &Directions, r: &u16, c: &u16) -> String {
        let (mut router_x, mut router_y) = Router::get_move_coordinates(r, c);
        router_x += HALF_SIDE_LENGTH;
        router_y += SIDE_LENGTH - ROUTER_OFFSET;
        router_y -= HALF_SIDE_LENGTH;

        let ret: String;

        match direction {
            Directions::North => {
                router_x += OUTPUT_LINK_OFFSET;
                router_y -= HALF_SIDE_LENGTH;
                ret = format!("M{},{} v-{}", router_x, router_y, CONNECTION_LENGTH);
            }
            Directions::East => {
                router_x += HALF_SIDE_LENGTH;
                router_y -= OUTPUT_LINK_OFFSET;
                ret = format!("M{},{} h{}", router_x, router_y, CONNECTION_LENGTH);
            }
            Directions::South => {
                router_y += HALF_SIDE_LENGTH;
                ret = format!("M{},{} v{}", router_x, router_y, CONNECTION_LENGTH);
            }
            Directions::West => {
                router_x -= HALF_SIDE_LENGTH;
                ret = format!("M{},{} h-{}", router_x, router_y, CONNECTION_LENGTH);
            }
        }

        ret
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
                let input_x = start_x - I_SINKS_SOURCE_CONNECTION_SPACING;
                let input_y = start_y - connection_length;
                let input_s = format!("M{},{} v{}", input_x, input_y, render_length);

                // Output
                let output_x = start_x + I_SINKS_SOURCE_CONNECTION_SPACING;
                let output_s = format!("M{},{} v-{}", output_x, start_y, render_length);

                (input_s, output_s)
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
                let input_y = start_y - I_SINKS_SOURCE_CONNECTION_SPACING;
                let input_s = format!("M{},{} h-{}", input_x, input_y, render_length);

                // Output
                let output_y = start_y + I_SINKS_SOURCE_CONNECTION_SPACING;
                let output_s = format!("M{},{} h{}", start_x, output_y, render_length);

                (input_s, output_s)
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
                let input_x = start_x + I_SINKS_SOURCE_CONNECTION_SPACING;
                let input_y = start_y + connection_length;
                let input_s = format!("M{},{} v-{}", input_x, input_y, render_length);

                // Output
                let output_x = start_x - I_SINKS_SOURCE_CONNECTION_SPACING;
                let output_s = format!("M{},{} v{}", output_x, start_y, render_length);

                (input_s, output_s)
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
                let input_y = start_y - I_SINKS_SOURCE_CONNECTION_SPACING;
                let input_s = format!("M{},{} h{}", input_x, input_y, render_length);

                // Output
                let output_y = start_y + I_SINKS_SOURCE_CONNECTION_SPACING;
                let output_s = format!("M{},{} h-{}", start_x, output_y, render_length);

                (input_s, output_s)
            }
        };

        EdgePath { input, output }
    }

    pub fn new(path: String) -> Self {
        Self {
            d: path,
            attributes: CommonAttributes::with_no_class(),
            marker_end: MARKER_REFERENCE,
        }
    }
}

#[derive(Serialize, Default)]
pub struct Connections {
    path: Vec<Connection>,
}

#[derive(Serialize)]
pub struct EdgeConnections {
    #[serde(rename = "@id")]
    id: &'static str,
    path: Vec<Connection>,
}

impl Default for EdgeConnections {
    fn default() -> Self {
        Self {
            id: EDGE_CONNECTIONS_ID,
            path: Vec::new(),
        }
    }
}

#[derive(Serialize, Default)]
pub struct ConnectionsParentGroup {
    #[serde(rename = "g")]
    connections: Connections,
    #[serde(rename = "g")]
    edge_connections: EdgeConnections,
}

impl ConnectionsParentGroup {
    fn add_edge_connection(&mut self, direction: &Directions, r: &u16, c: &u16) {
        let EdgePath { input, output } = Connection::get_edge_paths(direction, r, c);
        self.edge_connections.path.push(Connection::new(input));
        self.edge_connections.path.push(Connection::new(output));
    }

    fn add_inner_connection(&mut self, direction: &Directions, r: &u16, c: &u16) {
        let path = Connection::get_inner_path(direction, &r, &c);
        self.connections.path.push(Connection::new(path));
    }

    pub fn add_connections(&mut self, core: &Core, r: &u16, c: &u16, columns: u8, rows: u8) {
        let on_edge = core.is_on_edge(columns, rows);

        for direction in core.channels().channel().keys() {
            if let Some(edge_position) = on_edge.as_ref() {
                match direction {
                    Directions::North => match edge_position {
                        EdgePosition::Top | EdgePosition::TopLeft | EdgePosition::TopRight => {
                            self.add_edge_connection(direction, r, c)
                        }
                        _ => self.add_inner_connection(direction, r, c),
                    },
                    Directions::East => match edge_position {
                        EdgePosition::Right
                        | EdgePosition::TopRight
                        | EdgePosition::BottomRight => self.add_edge_connection(direction, r, c),
                        _ => self.add_inner_connection(direction, r, c),
                    },
                    Directions::South => match edge_position {
                        EdgePosition::Bottom
                        | EdgePosition::BottomLeft
                        | EdgePosition::BottomRight => self.add_edge_connection(direction, r, c),
                        _ => self.add_inner_connection(direction, r, c),
                    },
                    Directions::West => match edge_position {
                        EdgePosition::Left | EdgePosition::TopLeft | EdgePosition::BottomLeft => {
                            self.add_edge_connection(direction, r, c)
                        }
                        _ => self.add_inner_connection(direction, r, c),
                    },
                }
            } else {
                self.add_inner_connection(direction, r, c);
            }
        }
    }
}
