use std::{
    collections::{BTreeMap, HashMap},
    ops::Div,
};

use const_format::concatcp;
use manycore_parser::{
    source::Source, Directions, EdgePosition, SinkSourceDirection, WithXMLAttributes,
};
use serde::Serialize;

use crate::{
    style::EDGE_DATA_CLASS_NAME, text_background::TEXT_BACKGROUND_ID, Configuration,
    ConnectionType, ConnectionsParentGroup, DirectionType, FieldConfiguration, ProcessingGroup,
    SVGError, SVGErrorKind, HALF_CONNECTION_LENGTH, HALF_SIDE_LENGTH,
    I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH, MARKER_HEIGHT, ROUTER_OFFSET, SIDE_LENGTH,
};

static OFFSET_FROM_BORDER: u16 = 1;
static TEXT_GROUP_FILTER: &str = concatcp!("url(#", TEXT_BACKGROUND_ID, ")");
static CORE_CLIP: &str = "path('m0,0 l0,100 l98,0 l0,-75 l-25,-25 l-75,0 Z')";
static ROUTER_CLIP: &str = "path('m0,0 l0,74 l25,25 l73,0 l0,-100 Z')";
static OFFSET_FROM_LINK: u16 = 5;
const TOP_COORDINATES: &str = "T";
const BOTTOM_COORDINATES: &str = "B";

static LINK_LOAD_25: &str = "#84cc16";
static LINK_LOAD_50: &str = "#f59e0b";
static LINK_LOAD_75: &str = "#ef4444";

#[derive(Serialize)]
pub struct TextInformation {
    #[serde(rename = "@x")]
    x: i32,
    #[serde(rename = "@y")]
    y: i32,
    #[serde(rename = "@font-size")]
    font_size: &'static str,
    #[serde(rename = "@font-family")]
    font_family: &'static str,
    #[serde(rename = "@text-anchor")]
    text_anchor: &'static str,
    #[serde(rename = "@dominant-baseline")]
    dominant_baseline: &'static str,
    #[serde(rename = "@fill")]
    fill: String,
    #[serde(rename = "@class", skip_serializing_if = "Option::is_none")]
    class: Option<&'static str>,
    #[serde(rename = "$text")]
    value: String,
}

impl TextInformation {
    pub fn new_signed(
        x: i32,
        y: i32,
        text_anchor: &'static str,
        dominant_baseline: &'static str,
        fill: Option<&String>,
        class: Option<&'static str>,
        value: String,
    ) -> Self {
        Self {
            x,
            y,
            font_size: "16px",
            font_family: "Roboto Mono",
            text_anchor,
            dominant_baseline,
            fill: if let Some(f) = fill {
                f.clone()
            } else {
                "black".to_string()
            },
            class,
            value,
        }
    }

    pub fn new(
        x: u16,
        y: u16,
        text_anchor: &'static str,
        dominant_baseline: &'static str,
        fill: Option<&String>,
        class: Option<&'static str>,
        value: String,
    ) -> Self {
        // TODO: Actually check conversions. This needs doing all over really.
        Self::new_signed(
            x.into(),
            y.into(),
            text_anchor,
            dominant_baseline,
            fill,
            class,
            value,
        )
    }

    fn get_link_load_fill(load_fraction: Option<&u16>) -> Option<String> {
        match load_fraction.copied() {
            None => Some(LINK_LOAD_75.into()),
            Some(load_fraction) => {
                if load_fraction <= 25 {
                    return Some(LINK_LOAD_25.into());
                } else if load_fraction <= 50 {
                    return Some(LINK_LOAD_50.into());
                } else {
                    return Some(LINK_LOAD_75.into());
                }
            }
        }
    }

    fn common_load(
        link_x: &i32,
        link_y: &i32,
        direction: &Directions,
        relevant_delta: i32,
        class: Option<&'static str>,
        load: &u16,
        bandwidth: &u16,
    ) -> Self {
        let load_fraction = match *bandwidth {
            0 => None,
            b => Some(load.div(b).saturating_mul(100)),
        };
        let fill = TextInformation::get_link_load_fill(load_fraction.as_ref());

        match direction {
            Directions::North => {
                let delta_y = relevant_delta;

                TextInformation::new_signed(
                    link_x.saturating_add(OFFSET_FROM_LINK.into()),
                    link_y.saturating_sub(delta_y),
                    "start",
                    "middle",
                    fill.as_ref(),
                    class,
                    load.to_string(),
                )
            }
            Directions::East => {
                let delta_x = relevant_delta;

                TextInformation::new_signed(
                    link_x.saturating_add(delta_x),
                    link_y.saturating_sub(OFFSET_FROM_LINK.into()),
                    "middle",
                    "text-after-edge",
                    fill.as_ref(),
                    class,
                    load.to_string(),
                )
            }
            Directions::South => {
                let delta_y = relevant_delta;

                TextInformation::new_signed(
                    link_x.saturating_sub(OFFSET_FROM_LINK.into()),
                    link_y.saturating_add(delta_y),
                    "end",
                    "middle",
                    fill.as_ref(),
                    class,
                    load.to_string(),
                )
            }
            Directions::West => {
                let delta_x = relevant_delta;

                TextInformation::new_signed(
                    link_x.saturating_sub(delta_x),
                    link_y.saturating_add(OFFSET_FROM_LINK.into()),
                    "middle",
                    "text-before-edge",
                    fill.as_ref(),
                    class,
                    load.to_string(),
                )
            }
        }
    }

    fn source_load(
        direction: &Directions,
        link_x: &i32,
        link_y: &i32,
        load: &u16,
        bandwidth: &u16,
    ) -> Self {
        let relevant_delta: i32 = match direction {
            Directions::South | Directions::West => I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                .saturating_add(MARKER_HEIGHT.into())
                .div(2),
            _ => I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                .saturating_add(ROUTER_OFFSET.into())
                .saturating_add(MARKER_HEIGHT.into())
                .div(2),
        };

        TextInformation::common_load(
            link_x,
            link_y,
            direction,
            relevant_delta,
            Some(EDGE_DATA_CLASS_NAME),
            load,
            bandwidth,
        )
    }

    fn link_load(
        direction: &Directions,
        link_x: &i32,
        link_y: &i32,
        load: &u16,
        bandwidth: &u16,
        edge: bool,
    ) -> Self {
        let (relevant_delta, class): (i32, Option<&'static str>) = match edge {
            true => match direction {
                Directions::North | Directions::East => (
                    I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                        .saturating_add(MARKER_HEIGHT.into())
                        .div(2),
                    Some(EDGE_DATA_CLASS_NAME),
                ),
                _ => (
                    I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                        .saturating_add(ROUTER_OFFSET.into())
                        .saturating_add(MARKER_HEIGHT.into())
                        .div(2),
                    Some(EDGE_DATA_CLASS_NAME),
                ),
            },
            false => (HALF_CONNECTION_LENGTH.into(), None),
        };

        TextInformation::common_load(
            link_x,
            link_y,
            direction,
            relevant_delta,
            class,
            load,
            bandwidth,
        )
    }
}

#[derive(Serialize, Default)]
struct ProcessingInformation {
    #[serde(rename = "@filter", skip_serializing_if = "Option::is_none")]
    filter: Option<&'static str>,
    #[serde(rename = "@clip-path")]
    clip_path: &'static str,
    #[serde(rename = "text")]
    information: Vec<TextInformation>,
}

#[derive(Serialize, Default)]
#[serde(rename = "g")]
pub struct InformationLayer {
    #[serde(rename = "g")]
    core_group: ProcessingInformation,
    #[serde(rename = "g")]
    router_group: ProcessingInformation,
    #[serde(rename = "text", skip_serializing_if = "Option::is_none")]
    coordinates: Option<TextInformation>,
    #[serde(rename = "text", skip_serializing_if = "Vec::is_empty")]
    links_load: Vec<TextInformation>,
}

mod utils;
use utils::generate;

fn missing_connection(idx: &usize) -> SVGError {
    SVGError::new(SVGErrorKind::ConnectionError(format!(
        "Could not grab SVG connection path for Core {}",
        idx
    )))
}

fn missing_source(task_id: &u16) -> SVGError {
    SVGError::new(SVGErrorKind::ManycoreMismatch(format!(
        "Could not retrieve Source for Task {}",
        task_id
    )))
}

fn missing_channel(core_id: &u8, direction: &Directions) -> SVGError {
    SVGError::new(SVGErrorKind::ManycoreMismatch(format!(
        "Could not retrieve {} channel for Core {}",
        direction, core_id
    )))
}

impl InformationLayer {
    fn binary_search_left_insertion_point(bounds: &[u64; 4], val: u64) -> usize {
        // Bounds has always length 4
        let mut l: i8 = 0;
        let max = (bounds.len() - 1) as i8;
        let mut r: i8 = max;

        while l <= r {
            let m = l + (r - l) / 2;
            let cmp = bounds[m as usize];

            if cmp >= val {
                r = m - 1;
            } else {
                l = m + 1
            }
        }

        let corrected_l = std::cmp::max(std::cmp::min(l, max), 0) as usize;

        // We found the left most insertion point
        // But we don't know if we are here because we are the same as the next element
        // or greater than the previous but smaller than next
        if corrected_l > 0 && bounds[corrected_l] > val {
            corrected_l - 1
        } else {
            corrected_l
        }
    }

    fn get_connection_type<'a>(
        connections_group: &'a ConnectionsParentGroup,
        direction_type: &'a DirectionType,
        core_id: &'a u8,
    ) -> Result<&'a ConnectionType, SVGError> {
        connections_group
            .core_connections_map()
            .get(core_id)
            .ok_or(SVGError::new(SVGErrorKind::ConnectionError(format!(
                "Could not get connections for Core {}",
                core_id
            ))))?
            .get(direction_type)
            .ok_or(SVGError::new(SVGErrorKind::ConnectionError(format!(
                "Could not get connection {} for Core {}",
                direction_type, core_id
            ))))
    }

    pub fn new(
        rows: u8,
        columns: u8,
        configuration: &Configuration,
        core: &manycore_parser::Core,
        sources_ids: Option<&HashMap<SinkSourceDirection, Vec<u16>>>,
        sources: &BTreeMap<u16, Source>,
        css: &mut String,
        core_loads: Option<&Vec<Directions>>,
        processing_group: &ProcessingGroup,
        connections_group: &ConnectionsParentGroup,
    ) -> Result<Self, SVGError> {
        let mut ret = InformationLayer::default();
        let core_config = configuration.core_config();

        let (r, c) = processing_group.coordinates();
        let (core_x, core_y) = processing_group.core().move_coordinates();

        // Coordinates are stored in the core config but apply to whole group
        if let Some(order_config) = core_config.get("@coordinates") {
            let x = core_x + HALF_SIDE_LENGTH;
            let y = core_y + SIDE_LENGTH;

            let (cx, cy) = match order_config {
                FieldConfiguration::Text(order) => {
                    match order.as_str() {
                        BOTTOM_COORDINATES => (u16::from(rows) - r, c + 1),
                        TOP_COORDINATES | _ => {
                            // Top or anything else (malformeed input)
                            (r + 1, c + 1)
                        }
                    }
                }
                _ => (r + 1, c + 1), // Don't know what happened. Wrong enum variant, default to top left.
            };

            ret.coordinates = Some(TextInformation::new(
                x,
                y,
                "middle",
                "text-before-edge",
                None,
                None,
                format!("({},{})", cx, cy),
            ));
        }

        // Core
        generate(
            *core_x,
            *core_y,
            configuration.core_config(),
            core,
            &mut ret.core_group,
            "start",
            css,
        );
        ret.core_group.clip_path = CORE_CLIP;

        // Router
        let (router_x, router_y) = processing_group.router().move_coordinates();
        generate(
            *router_x,
            router_y - ROUTER_OFFSET,
            configuration.router_config(),
            core.router(),
            &mut ret.router_group,
            "start",
            css,
        );
        ret.router_group.clip_path = ROUTER_CLIP;

        // Link loads
        if let Some(directions) = core_loads {
            for direction in directions {
                let direction_type = DirectionType::Out(*direction);

                let connection_type = InformationLayer::get_connection_type(
                    connections_group,
                    &direction_type,
                    core.id(),
                )?;

                let (x, y, edge) = match connection_type {
                    ConnectionType::Connection(idx) => {
                        let connection = connections_group
                            .connections()
                            .path()
                            .get(*idx)
                            .ok_or(missing_connection(idx))?;

                        (connection.x(), connection.y(), false)
                    }
                    ConnectionType::EdgeConnection(idx) => {
                        let connection = connections_group
                            .edge_connections()
                            .sink()
                            .get(*idx)
                            .ok_or(missing_connection(idx))?;

                        (connection.x(), connection.y(), true)
                    }
                };

                let channel = core
                    .channels()
                    .channel()
                    .get(direction)
                    .ok_or(missing_channel(core.id(), &direction))?;

                let load = channel.current_load();

                let bandwidth = channel.bandwidth();

                ret.links_load.push(TextInformation::link_load(
                    direction, x, y, load, bandwidth, edge,
                ));
            }
        }

        // Source loads
        if let Some(sources_ids) = sources_ids {
            if let Some(edge_position) = core.is_on_edge(columns, rows).as_ref() {
                let keys: Vec<SinkSourceDirection> = edge_position.into();

                for source_direction in keys {
                    if let Some(tasks) = sources_ids.get(&source_direction) {
                        let mut load = 0;

                        for task_id in tasks {
                            let source = sources.get(&task_id).ok_or(missing_source(task_id))?;
                            load += source.current_load();
                        }

                        let mut direction: Directions = (&source_direction).into();
                        let direction_type = DirectionType::Source(direction.clone());
                        let connection_type = InformationLayer::get_connection_type(
                            connections_group,
                            &direction_type,
                            core.id(),
                        )?;

                        if let ConnectionType::EdgeConnection(idx) = connection_type {
                            let connection = connections_group
                                .edge_connections()
                                .source()
                                .get(*idx)
                                .ok_or(missing_connection(idx))?;

                            let channel = core
                                .channels()
                                .channel()
                                .get(&direction)
                                .ok_or(missing_channel(core.id(), &direction))?;

                            // Flip direction, source notation is inverted
                            direction = match direction {
                                Directions::North => Directions::South,
                                Directions::South => Directions::North,
                                Directions::East => Directions::West,
                                Directions::West => Directions::East,
                            };

                            ret.links_load.push(TextInformation::source_load(
                                &direction,
                                connection.x(),
                                connection.y(),
                                &load,
                                channel.bandwidth(),
                            ));
                        } else {
                            panic!("Not supposed to be this");
                        }
                    }
                }
            } else {
                panic!("Core must be on edge for source ids")
            }
        }

        Ok(ret)
    }
}
