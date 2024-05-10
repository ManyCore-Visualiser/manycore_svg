use std::{collections::HashMap, fmt::Display};

use getset::Getters;
use manycore_parser::{Core, Directions, EdgePosition, ElementIDT, WithID};
use serde::Serialize;

use crate::{
    sinks_sources_layer::SINKS_SOURCES_CONNECTION_LENGTH, style::EDGE_DATA_CLASS_NAME,
    CommonAttributes, CoordinateT, Router, TopLeft, HALF_ROUTER_OFFSET, MARKER_HEIGHT,
    MARKER_REFERENCE, ROUTER_OFFSET, SIDE_LENGTH, USE_FREEFORM_CLIP_PATH,
};

pub(crate) const EDGE_CONNECTIONS_ID: &'static str = "edgeConnetions";
static CONNECTION_GAP: CoordinateT = 0i32
    .saturating_add(HALF_ROUTER_OFFSET)
    .saturating_mul(3)
    .saturating_div(4)
    .wrapping_sub(5);

pub(crate) static CONNECTION_LENGTH: CoordinateT = ROUTER_OFFSET.saturating_mul(4);

/// Object representation of a connection path.
#[derive(Serialize, Getters, Debug)]
pub(crate) struct Connection {
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    attributes: CommonAttributes,
    #[serde(rename = "@marker-end")]
    marker_end: &'static str,
    #[serde(skip)]
    #[getset(get = "pub")]
    x: CoordinateT,
    #[serde(skip)]
    #[getset(get = "pub")]
    y: CoordinateT,
    #[serde(rename = "@clip-path")]
    clip_path: &'static str,
}

/// Helper struct used when calculating connection paths.
struct ConnectionPath {
    path: String,
    x: CoordinateT,
    y: CoordinateT,
}

/// Helper struct to group input and output connections paths together.
struct EdgePath {
    input: ConnectionPath,
    output: ConnectionPath,
}

impl Connection {
    /// Calculates the path for an inner matrix connection.
    fn get_inner_path(
        direction: &Directions,
        r: &CoordinateT,
        c: &CoordinateT,
        top_left: &TopLeft,
    ) -> ConnectionPath {
        let (mut router_x, mut router_y) = Router::get_move_coordinates(r, c, top_left);

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

    /// Calculates the paths (input and output at once) for an edge router connection.
    fn get_edge_paths(
        direction: &Directions,
        r: &CoordinateT,
        c: &CoordinateT,
        top_left: &TopLeft,
    ) -> EdgePath {
        let (mut router_x, mut router_y) = Router::get_move_coordinates(r, c, top_left);

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
                let input_y = router_y.saturating_add(CONNECTION_GAP);
                let input_s = format!("M{},{} h-{}", input_x, input_y, render_length);

                // Output
                let output_y = router_y.saturating_sub(CONNECTION_GAP);
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

    /// Creates a new [`Connection`] instance given a [`ConnectionPath`]. Remaining parameters are default.
    fn new(connection_path: ConnectionPath) -> Self {
        Self {
            d: connection_path.path,
            attributes: CommonAttributes::with_no_class(),
            marker_end: MARKER_REFERENCE,
            x: connection_path.x,
            y: connection_path.y,
            clip_path: USE_FREEFORM_CLIP_PATH,
        }
    }
}

/// Wrapper around [`Connection`] to serialise them all as SVG `<path>`.
#[derive(Serialize, Default, Getters, Debug)]
pub(crate) struct Connections {
    #[getset(get = "pub")]
    path: Vec<Connection>,
}

/// Object rpresentation fo the edge connections SVG group.
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

/// Enum variants to describe [`Connection`] types. Variant content is index of element.
pub(crate) enum ConnectionType {
    EdgeConnection(usize),
    Connection(usize),
}

/// Enum variants to describe a connection direction. Variant content is cardinal [`Directions`].
#[derive(Hash, PartialEq, Eq)]
pub(crate) enum DirectionType {
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

/// Object representation of the SVG group that contains channel connections.
#[derive(Serialize, Getters, Default)]
#[getset(get = "pub")]
pub(crate) struct ConnectionsParentGroup {
    #[serde(rename = "g")]
    connections: Connections,
    #[serde(rename = "g")]
    edge_connections: EdgeConnections,
    /// A double map to quickly retrieve a core's (router) connections in the SVG.
    #[serde(skip)]
    core_connections_map: HashMap<ElementIDT, HashMap<DirectionType, ConnectionType>>,
}

impl ConnectionsParentGroup {
    /// Inserts an SVG core connection in the core_connections_map.
    fn insert_in_map(&mut self, core_id: &ElementIDT, direction: DirectionType, element: ConnectionType) {
        self.core_connections_map
            .entry(*core_id)
            // Each core has 4 connections, so we preallocate 4 slots in the inner map.
            .or_insert(HashMap::with_capacity(4))
            .insert(direction, element);
    }

    /// Generates edge SVG connections for a given core.
    fn add_edge_connection(
        &mut self,
        core_id: &ElementIDT,
        direction: &Directions,
        r: &CoordinateT,
        c: &CoordinateT,
        top_left: &TopLeft,
    ) {
        let EdgePath { input, output } = Connection::get_edge_paths(direction, r, c, top_left);
        let current_source_size = self.edge_connections.source.len();
        let current_sink_size = self.edge_connections.sink.len();

        self.edge_connections.source.push(Connection::new(input));
        self.edge_connections.sink.push(Connection::new(output));

        // When we insert in map, we store direction and the index of the element in its
        // respective vector so we can grab it quickly in case we need to display its load,
        // and hence need its coordinates.
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

    /// Generates inner SVG connections (so output only) for a given core.
    fn add_inner_connection(
        &mut self,
        core_id: &ElementIDT,
        direction: &Directions,
        r: &CoordinateT,
        c: &CoordinateT,
        top_left: &TopLeft,
    ) {
        let path = Connection::get_inner_path(direction, &r, &c, top_left);
        let current_size = self.connections.path.len();

        self.connections.path.push(Connection::new(path));

        // When we insert in map, we store direction and the index of the element in its
        // respective vector so we can grab it quickly in case we need to display its load,
        // and hence need its coordinates.
        self.insert_in_map(
            core_id,
            DirectionType::Out(*direction),
            ConnectionType::Connection(current_size),
        );
    }

    // Generates a core's SVG connections.
    pub(crate) fn add_connections(
        &mut self,
        core: &Core,
        r: &CoordinateT,
        c: &CoordinateT,
        top_left: &TopLeft,
    ) {
        // Does this core have edge connections?
        let on_edge = core.matrix_edge();

        // For each core's channel direction in the provided manycore system
        for direction in core.channels().channel().keys() {
            if let Some(edge_position) = on_edge.as_ref() {
                // Here we match against the direction and the edge position to decide
                // what kind of SVG connection (inner (core->core) or edge (core->border, border->core)) to generate.
                match direction {
                    Directions::North => match edge_position {
                        EdgePosition::Top | EdgePosition::TopLeft | EdgePosition::TopRight => {
                            self.add_edge_connection(core.id(), direction, r, c, top_left)
                        }
                        _ => self.add_inner_connection(core.id(), direction, r, c, top_left),
                    },
                    Directions::East => match edge_position {
                        EdgePosition::Right
                        | EdgePosition::TopRight
                        | EdgePosition::BottomRight => {
                            self.add_edge_connection(core.id(), direction, r, c, top_left)
                        }
                        _ => self.add_inner_connection(core.id(), direction, r, c, top_left),
                    },
                    Directions::South => match edge_position {
                        EdgePosition::Bottom
                        | EdgePosition::BottomLeft
                        | EdgePosition::BottomRight => {
                            self.add_edge_connection(core.id(), direction, r, c, top_left)
                        }
                        _ => self.add_inner_connection(core.id(), direction, r, c, top_left),
                    },
                    Directions::West => match edge_position {
                        EdgePosition::Left | EdgePosition::TopLeft | EdgePosition::BottomLeft => {
                            self.add_edge_connection(core.id(), direction, r, c, top_left)
                        }
                        _ => self.add_inner_connection(core.id(), direction, r, c, top_left),
                    },
                }
            } else {
                // This core is not on edge so this is just a core->core connection.
                self.add_inner_connection(core.id(), direction, r, c, top_left);
            }
        }
    }
}
