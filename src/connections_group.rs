use std::{collections::HashMap, fmt::Display};

use getset::Getters;
use manycore_parser::{Core, Directions, EdgePosition, WithID};
use serde::Serialize;

use crate::{
    coordinate, sinks_sources_layer::SINKS_SOURCES_CONNECTION_LENGTH, style::EDGE_DATA_CLASS_NAME,
    CommonAttributes, Router, HALF_ROUTER_OFFSET, MARKER_HEIGHT, MARKER_REFERENCE, ROUTER_OFFSET,
    SIDE_LENGTH,
};

pub const EDGE_CONNECTIONS_ID: &'static str = "edgeConnetions";
static CONNECTION_GAP: coordinate = 0i32
    .saturating_add(HALF_ROUTER_OFFSET)
    .saturating_mul(3)
    .saturating_div(4)
    .wrapping_sub(5);

pub static CONNECTION_LENGTH: coordinate = ROUTER_OFFSET.saturating_mul(4);

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
    x: coordinate,
    #[serde(skip)]
    #[getset(get = "pub")]
    y: coordinate,
}

struct ConnectionPath {
    path: String,
    x: coordinate,
    y: coordinate,
}

struct EdgePath {
    input: ConnectionPath,
    output: ConnectionPath,
}

impl Connection {
    fn get_inner_path(direction: &Directions, r: &coordinate, c: &coordinate) -> ConnectionPath {
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
            x: router_x,
            y: router_y,
        }
    }

    fn get_edge_paths(direction: &Directions, r: &coordinate, c: &coordinate) -> EdgePath {
        let (mut router_x, mut router_y) = Router::get_move_coordinates(r, c);

        let (input, output) = match direction {
            Directions::North => {
                router_x = router_x
                    .saturating_add(SIDE_LENGTH)
                    .saturating_sub(HALF_ROUTER_OFFSET);
                router_y = router_y.saturating_sub(ROUTER_OFFSET);

                let connection_length =
                    SINKS_SOURCES_CONNECTION_LENGTH.saturating_add(MARKER_HEIGHT);
                let render_length = SINKS_SOURCES_CONNECTION_LENGTH;

                // Input
                let input_x = router_x.saturating_sub(CONNECTION_GAP);
                let input_y = router_y.saturating_sub(connection_length);
                let input_s = format!("M{},{} v{}", input_x, input_y, render_length);

                // Output
                let output_x = router_x.saturating_add(CONNECTION_GAP);
                let output_s = format!("M{},{} v-{}", output_x, router_y, render_length);

                (
                    ConnectionPath {
                        path: input_s,
                        x: input_x,
                        y: input_y,
                    },
                    ConnectionPath {
                        path: output_s,
                        x: output_x,
                        y: router_y,
                    },
                )
            }
            Directions::East => {
                router_x = router_x.saturating_add(SIDE_LENGTH);
                router_y = router_y
                    .saturating_sub(ROUTER_OFFSET)
                    .saturating_add(HALF_ROUTER_OFFSET);

                let connection_length =
                    SINKS_SOURCES_CONNECTION_LENGTH.saturating_add(MARKER_HEIGHT);
                let render_length = SINKS_SOURCES_CONNECTION_LENGTH;

                // Input
                let input_x = router_x.saturating_add(connection_length);
                let input_y = router_y.saturating_sub(CONNECTION_GAP);
                let input_s = format!("M{},{} h-{}", input_x, input_y, render_length);

                // Output
                let output_y = router_y.saturating_add(CONNECTION_GAP);
                let output_s = format!("M{},{} h{}", router_x, output_y, render_length);

                (
                    ConnectionPath {
                        path: input_s,
                        x: input_x,
                        y: input_y,
                    },
                    ConnectionPath {
                        path: output_s,
                        x: router_x,
                        y: output_y,
                    },
                )
            }
            Directions::South => {
                router_x = router_x
                    .saturating_add(SIDE_LENGTH)
                    .saturating_sub(HALF_ROUTER_OFFSET);
                router_y = router_y
                    .saturating_sub(ROUTER_OFFSET)
                    .saturating_add(SIDE_LENGTH);

                let render_length = SINKS_SOURCES_CONNECTION_LENGTH.saturating_add(ROUTER_OFFSET);
                let connection_length = render_length.saturating_add(MARKER_HEIGHT);

                // Input
                let input_x = router_x.saturating_add(CONNECTION_GAP);
                let input_y = router_y.saturating_add(connection_length);
                let input_s = format!("M{},{} v-{}", input_x, input_y, render_length);

                // Output
                let output_x = router_x.saturating_sub(CONNECTION_GAP);
                let output_s = format!("M{},{} v{}", output_x, router_y, render_length);

                (
                    ConnectionPath {
                        path: input_s,
                        x: input_x,
                        y: input_y,
                    },
                    ConnectionPath {
                        path: output_s,
                        x: output_x,
                        y: router_y,
                    },
                )
            }
            Directions::West => {
                router_y = router_y
                    .saturating_sub(ROUTER_OFFSET)
                    .saturating_add(HALF_ROUTER_OFFSET);

                let render_length = SINKS_SOURCES_CONNECTION_LENGTH.saturating_add(ROUTER_OFFSET);
                let connection_length = render_length.saturating_add(MARKER_HEIGHT);

                // Input
                let input_x = router_x.saturating_sub(connection_length);
                let input_y = router_y.saturating_sub(CONNECTION_GAP);
                let input_s = format!("M{},{} h{}", input_x, input_y, render_length);

                // Output
                let output_y = router_y.saturating_add(CONNECTION_GAP);
                let output_s = format!("M{},{} h-{}", router_x, output_y, render_length);

                (
                    ConnectionPath {
                        path: input_s,
                        x: input_x,
                        y: input_y,
                    },
                    ConnectionPath {
                        path: output_s,
                        x: router_x,
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
    #[serde(rename = "path")]
    #[getset(get = "pub")]
    source: Vec<Connection>,
    #[serde(rename = "path")]
    #[getset(get = "pub")]
    sink: Vec<Connection>,
}

impl Default for EdgeConnections {
    fn default() -> Self {
        Self {
            id: EDGE_CONNECTIONS_ID,
            class: EDGE_DATA_CLASS_NAME,
            source: Vec::new(),
            sink: Vec::new(),
        }
    }
}

pub enum ConnectionType {
    EdgeConnection(usize),
    Connection(usize),
}

#[derive(Hash, PartialEq, Eq)]
pub enum DirectionType {
    Out(Directions),
    Source(Directions),
}

impl Display for DirectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectionType::Out(direction) => write!(f, "DirectionType(Out({}))", direction),
            DirectionType::Source(direction) => write!(f, "DirectionType(Source({}))", direction),
        }
    }
}

#[derive(Serialize, Getters, Default)]
#[getset(get = "pub")]
pub struct ConnectionsParentGroup {
    #[serde(rename = "g")]
    connections: Connections,
    #[serde(rename = "g")]
    edge_connections: EdgeConnections,
    #[serde(skip)]
    core_connections_map: HashMap<u8, HashMap<DirectionType, ConnectionType>>,
}

impl ConnectionsParentGroup {
    fn insert_in_map(&mut self, core_id: &u8, direction: DirectionType, element: ConnectionType) {
        // Each core has 4 connections
        self.core_connections_map
            .entry(*core_id)
            .or_insert(HashMap::with_capacity(4))
            .insert(direction, element);
    }

    fn add_edge_connection(
        &mut self,
        core_id: &u8,
        direction: &Directions,
        r: &coordinate,
        c: &coordinate,
    ) {
        let EdgePath { input, output } = Connection::get_edge_paths(direction, r, c);
        let current_source_size = self.edge_connections.source.len();
        let current_sink_size = self.edge_connections.sink.len();

        self.edge_connections.source.push(Connection::new(input));
        self.edge_connections.sink.push(Connection::new(output));

        self.insert_in_map(
            core_id,
            DirectionType::Source(*direction),
            ConnectionType::EdgeConnection(current_source_size),
        );
        self.insert_in_map(
            core_id,
            DirectionType::Out(*direction),
            ConnectionType::EdgeConnection(current_sink_size),
        );
    }

    fn add_inner_connection(
        &mut self,
        core_id: &u8,
        direction: &Directions,
        r: &coordinate,
        c: &coordinate,
    ) {
        let path = Connection::get_inner_path(direction, &r, &c);
        let current_size = self.connections.path.len();

        self.connections.path.push(Connection::new(path));

        self.insert_in_map(
            core_id,
            DirectionType::Out(*direction),
            ConnectionType::Connection(current_size),
        );
    }

    pub fn add_connections(&mut self, core: &Core, r: &coordinate, c: &coordinate, columns: u8, rows: u8) {
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
