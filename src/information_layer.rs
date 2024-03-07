use const_format::concatcp;
use serde::Serialize;

use crate::{
    text_background::TEXT_BACKGROUND_ID, Configuration, Core, FieldConfiguration,
    Router, HALF_SIDE_LENGTH, ROUTER_OFFSET, SIDE_LENGTH,
};

static OFFSET_FROM_BORDER: u16 = 1;
static TEXT_GROUP_FILTER: &str = concatcp!("url(#", TEXT_BACKGROUND_ID, ")");
static CORE_CLIP: &str = "path('m0,0 l0,100 l98,0 l0,-75 l-25,-25 l-75,0 Z')";
static ROUTER_CLIP: &str = "path('m0,0 l0,74 l25,25 l73,0 l0,-100 Z')";
const TOP_COORDINATES: &str = "T";
const BOTTOM_COORDINATES: &str = "B";

#[derive(Serialize)]
struct TextInformation {
    #[serde(rename = "@x")]
    x: u16,
    #[serde(rename = "@y")]
    y: u16,
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
    fn new(
        x: u16,
        y: u16,
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
}

mod utils;
use utils::generate;

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
        css: &mut String,
    ) -> Self {
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
            css
        );
        ret.core_group.clip_path = CORE_CLIP;

        // Router
        let (router_x, mut router_y) = Router::get_move_coordinates(r, c);
        // Router x does not need offset
        router_y -= ROUTER_OFFSET;
        generate(
            router_x,
            router_y,
            configuration.router_config(),
            core.router(),
            &mut ret.router_group,
            "start",
            css
        );
        ret.router_group.clip_path = ROUTER_CLIP;

        ret
    }
}
