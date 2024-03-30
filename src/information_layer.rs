use std::{collections::HashMap, error::Error, fmt::Display, ops::Div};

use const_format::concatcp;
use manycore_parser::{Channels, Directions, WithXMLAttributes};
use serde::Serialize;

use crate::{
    processing_group::Core, sinks_sources_layer::SINKS_SOURCES_CONNECTION_EXTRA_LENGTH,
    text_background::TEXT_BACKGROUND_ID, Configuration, ConnectionType, Connections,
    ConnectionsParentGroup, FieldConfiguration, ProcessingGroup, Router, HALF_CONNECTION_LENGTH,
    HALF_SIDE_LENGTH, I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH, MARKER_HEIGHT, OUTPUT_LINK_OFFSET,
    ROUTER_OFFSET, SIDE_LENGTH,
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
            value,
        }
    }

    pub fn new(
        x: u16,
        y: u16,
        text_anchor: &'static str,
        dominant_baseline: &'static str,
        fill: Option<&String>,
        value: String,
    ) -> Self {
        // TODO: Actually check conversions. This needs doing all over really.
        Self::new_signed(
            x.into(),
            y.into(),
            text_anchor,
            dominant_baseline,
            fill,
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

    fn link_load(
        direction: &Directions,
        link_x: &i32,
        link_y: &i32,
        load: &u16,
        bandwidth: &u16,
        edge: bool,
    ) -> Self {
        let load_fraction = match *bandwidth {
            0 => None,
            b => Some(load.div(b).saturating_mul(100)),
        };
        let fill = TextInformation::get_link_load_fill(load_fraction.as_ref());

        let relevant_delta: i32 = match edge {
            true => match direction {
                Directions::North | Directions::East => I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                    .saturating_add(MARKER_HEIGHT.into())
                    .div(2),
                _ => I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                    .saturating_add(ROUTER_OFFSET.into())
                    .saturating_add(MARKER_HEIGHT.into())
                    .div(2),
            },
            false => HALF_CONNECTION_LENGTH.into(),
        };

        match direction {
            Directions::North => {
                let delta_y = relevant_delta;

                TextInformation::new_signed(
                    link_x.saturating_add(OFFSET_FROM_LINK.into()),
                    link_y.saturating_sub(delta_y),
                    "start",
                    "middle",
                    fill.as_ref(),
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
                    load.to_string(),
                )
            }
        }
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

#[derive(Debug)]
pub struct InformationLayerError;

impl Error for InformationLayerError {}
impl Display for InformationLayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Could not generate SVG information because a routing connection could not be found."
        )
    }
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

    pub fn new(
        total_rows: &u16,
        configuration: &Configuration,
        core: &manycore_parser::Core,
        css: &mut String,
        core_loads: Option<&Vec<Directions>>,
        processing_group: &ProcessingGroup,
        connections_group: &ConnectionsParentGroup,
    ) -> Result<Self, InformationLayerError> {
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
                        BOTTOM_COORDINATES => (total_rows - r, c + 1),
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
                let connection_type = connections_group
                    .core_connections_map()
                    .get(core.id())
                    .unwrap() // TODO: Replace with error
                    .get(direction)
                    .unwrap(); // TODO: Replace with error

                // TODO: Handle unwraps
                let (x, y, edge) = match connection_type {
                    ConnectionType::Connection(idx) => {
                        let connection = connections_group.connections().path().get(*idx).unwrap();

                        (connection.x(), connection.y(), false)
                    }
                    ConnectionType::EdgeConnection(idx) => {
                        let connection = connections_group
                            .edge_connections()
                            .path()
                            .get(*idx)
                            .unwrap();

                        (connection.x(), connection.y(), true)
                    }
                };

                let channel = core.channels().channel().get(direction).unwrap();

                let load = channel.current_load();

                let bandwidth = channel.bandwidth();

                ret.links_load.push(TextInformation::link_load(
                    direction, x, y, load, bandwidth, edge,
                ));
            }
        }

        Ok(ret)
    }
}
