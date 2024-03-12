use std::{collections::HashMap, error::Error, fmt::Display};

use const_format::concatcp;
use manycore_parser::{FIFODirection, FIFOs, Neighbour, Neighbours};
use serde::Serialize;

use crate::{
    text_background::TEXT_BACKGROUND_ID, Configuration, Core, FieldConfiguration, Router,
    HALF_SIDE_LENGTH, OUTPUT_LINK_OFFSET, ROUTER_OFFSET, SIDE_LENGTH,
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
    x: i16,
    #[serde(rename = "@y")]
    y: i16,
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
        x: i16,
        y: i16,
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
            x as i16,
            y as i16,
            text_anchor,
            dominant_baseline,
            fill,
            value,
        )
    }

    fn get_link_load_fill(
        direction: &FIFODirection,
        link_cost: &u8,
        fifos: Option<&FIFOs>,
    ) -> Option<String> {
        if let Some(fifos) = fifos {
            if let Some(fifo) = fifos.fifo().get(direction) {
                // Multiply by 100 so we don't deal with floating point
                let load_fraction = (u16::from(*link_cost) * 100) / fifo.bandwidth();

                if load_fraction <= 25 {
                    return Some(LINK_LOAD_25.into());
                } else if load_fraction <= 50 {
                    return Some(LINK_LOAD_50.into());
                } else {
                    return Some(LINK_LOAD_75.into());
                }
            }
        }

        None
    }

    fn link_load(
        direction: &FIFODirection,
        router_x: u16,
        router_y: u16,
        link_cost: &u8,
        fifos: Option<&FIFOs>,
    ) -> Self {
        let fill = TextInformation::get_link_load_fill(direction, link_cost, fifos);

        match direction {
            FIFODirection::NorthOutput => TextInformation::new(
                router_x + OUTPUT_LINK_OFFSET + OFFSET_FROM_LINK,
                router_y - (HALF_SIDE_LENGTH + OFFSET_FROM_LINK),
                "start",
                "text-after-edge",
                fill.as_ref(),
                link_cost.to_string(),
            ),
            FIFODirection::NorthInput => TextInformation::new(
                router_x - OFFSET_FROM_LINK,
                router_y - (HALF_SIDE_LENGTH + OFFSET_FROM_LINK),
                "end",
                "text-after-edge",
                fill.as_ref(),
                link_cost.to_string(),
            ),
            FIFODirection::EastOutput => TextInformation::new(
                router_x + HALF_SIDE_LENGTH + 2 * OFFSET_FROM_LINK,
                router_y - OUTPUT_LINK_OFFSET,
                "start",
                "text-after-edge",
                fill.as_ref(),
                link_cost.to_string(),
            ),
            FIFODirection::EastInput => TextInformation::new(
                router_x + HALF_SIDE_LENGTH + 2 * OFFSET_FROM_LINK,
                router_y,
                "start",
                "text-before-edge",
                fill.as_ref(),
                link_cost.to_string(),
            ),
            FIFODirection::SouthOutput => TextInformation::new(
                router_x - OFFSET_FROM_LINK,
                router_y + HALF_SIDE_LENGTH + OFFSET_FROM_LINK,
                "end",
                "text-before-edge",
                fill.as_ref(),
                link_cost.to_string(),
            ),
            FIFODirection::SouthInput => TextInformation::new(
                router_x + OUTPUT_LINK_OFFSET + OFFSET_FROM_LINK,
                router_y + HALF_SIDE_LENGTH + OFFSET_FROM_LINK,
                "start",
                "text-before-edge",
                fill.as_ref(),
                link_cost.to_string(),
            ),
            FIFODirection::WestOutput => TextInformation::new(
                router_x - (HALF_SIDE_LENGTH + 2 * OFFSET_FROM_LINK),
                router_y,
                "end",
                "text-before-edge",
                fill.as_ref(),
                link_cost.to_string(),
            ),
            // Ignore local links, they are not taken into account when routing
            FIFODirection::WestInput | _ => TextInformation::new(
                router_x - (HALF_SIDE_LENGTH + 2 * OFFSET_FROM_LINK),
                router_y - OUTPUT_LINK_OFFSET,
                "end",
                "text-after-edge",
                fill.as_ref(),
                link_cost.to_string(),
            ),
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
        // But we don't know if we are because we are the same as the next element
        // or greater than the previous but smaller than next
        if corrected_l > 0 && bounds[corrected_l] > val {
            corrected_l - 1
        } else {
            corrected_l
        }
    }

    pub fn new(
        total_rows: &u16,
        r: &u16,
        c: &u16,
        configuration: &Configuration,
        core: &manycore_parser::Core,
        core_index: &usize,
        connections: &HashMap<usize, Neighbours>,
        css: &mut String,
        core_loads: Option<&Vec<FIFODirection>>,
        fifos: Option<&FIFOs>,
    ) -> Result<Self, InformationLayerError> {
        let mut ret = InformationLayer::default();
        let core_config = configuration.core_config();

        let (core_x, core_y) = Core::get_move_coordinates(r, c);

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
            core_x,
            core_y,
            configuration.core_config(),
            core,
            &mut ret.core_group,
            "start",
            css,
        );
        ret.core_group.clip_path = CORE_CLIP;

        // Router
        let (mut router_x, mut router_y) = Router::get_move_coordinates(r, c);
        router_y -= ROUTER_OFFSET;
        generate(
            router_x,
            router_y,
            configuration.router_config(),
            core.router(),
            &mut ret.router_group,
            "start",
            css,
        );
        ret.router_group.clip_path = ROUTER_CLIP;

        // Link loads
        if let Some(directions) = core_loads {
            // Move router coordinates to centre
            router_x += HALF_SIDE_LENGTH;
            router_y += HALF_SIDE_LENGTH;

            let get_cost = |i: &usize,
                            selector: &dyn Fn(&Neighbours) -> &Option<Neighbour>,
                            accessor: &dyn Fn(&Neighbour) -> &u8|
             -> Result<&u8, InformationLayerError> {
                Ok(accessor(
                    selector(connections.get(i).ok_or(InformationLayerError)?)
                        .as_ref()
                        .ok_or(InformationLayerError)?,
                ))
            };

            for direction in directions {
                let link_cost = match direction {
                    FIFODirection::NorthOutput => {
                        get_cost(core_index, &Neighbours::top, &Neighbour::link_cost)?
                    }
                    FIFODirection::SouthOutput => {
                        get_cost(core_index, &Neighbours::bottom, &Neighbour::link_cost)?
                    }
                    FIFODirection::WestOutput => {
                        get_cost(core_index, &Neighbours::left, &Neighbour::link_cost)?
                    }
                    FIFODirection::EastOutput => {
                        get_cost(core_index, &Neighbours::right, &Neighbour::link_cost)?
                    }
                    FIFODirection::NorthInput => {
                        get_cost(core_index, &Neighbours::top, &Neighbour::input_link_cost)?
                    }
                    FIFODirection::SouthInput => {
                        get_cost(core_index, &Neighbours::bottom, &Neighbour::input_link_cost)?
                    }
                    FIFODirection::WestInput => {
                        get_cost(core_index, &Neighbours::left, &Neighbour::input_link_cost)?
                    }
                    FIFODirection::EastInput => {
                        get_cost(core_index, &Neighbours::right, &Neighbour::input_link_cost)?
                    }
                    // Ignore local links
                    _ => &0,
                };

                ret.links_load.push(TextInformation::link_load(
                    direction, router_x, router_y, link_cost, fifos,
                ));
            }
        }

        Ok(ret)
    }
}
