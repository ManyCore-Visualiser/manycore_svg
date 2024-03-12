use manycore_parser::SinkSourceDirection;
use serde::Serialize;

use crate::{style::DEFAULT_FILL, TextInformation, ROUTER_OFFSET};

static SINKS_SOURCES_SIDE_LENGTH: &str = "100";
static SINKS_SOURCES_STROKE_WIDTH: &str = "1";
static SINKS_SOURCES_RX: &str = "15";
pub static SINKS_SOURCES_GROUP_OFFSET: u16 = 121;
pub static I_SINKS_SOURCES_GROUP_OFFSET: i16 = 121;
static I_SINKS_SOURCES_HALF_SIDE_LENGTH: i16 = 50;
static SINK_FILL: &str = "#fb923c";
static SOURCE_FILL: &str = "#fbbf24";

impl TextInformation {
    fn sink_source_text(sink_x: i16, sink_y: i16, variant: SinkSourceVariant) -> Option<Self> {
        let text = match variant {
            SinkSourceVariant::Sink => Some(String::from("Sink")),
            SinkSourceVariant::Source => Some(String::from("Source")),
            SinkSourceVariant::None => None,
        };

        if let Some(text) = text {
            return Some(TextInformation::new_signed(
                sink_x + I_SINKS_SOURCES_HALF_SIDE_LENGTH,
                sink_y + I_SINKS_SOURCES_HALF_SIDE_LENGTH,
                "middle",
                "middle",
                None,
                text,
            ));
        }

        None
    }
}

#[derive(Serialize)]
struct Rect {
    #[serde(rename = "@x")]
    x: i16,
    #[serde(rename = "@y")]
    y: i16,
    #[serde(rename = "@width")]
    width: &'static str,
    #[serde(rename = "@height")]
    height: &'static str,
    #[serde(rename = "@rx")]
    rx: &'static str,
    #[serde(rename = "@fill")]
    fill: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl Rect {
    fn new(x: i16, y: i16, variant: SinkSourceVariant) -> Self {
        Self {
            x,
            y,
            width: SINKS_SOURCES_SIDE_LENGTH,
            height: SINKS_SOURCES_SIDE_LENGTH,
            rx: SINKS_SOURCES_RX,
            fill: match variant {
                SinkSourceVariant::Source => SOURCE_FILL,
                SinkSourceVariant::Sink => SINK_FILL,
                SinkSourceVariant::None => DEFAULT_FILL,
            },
            stroke: "black",
            stroke_width: SINKS_SOURCES_STROKE_WIDTH,
        }
    }
}

#[derive(Clone, Copy)]
pub enum SinkSourceVariant {
    Sink,
    Source,
    None,
}

#[derive(Serialize)]
pub struct SinkSource {
    rect: Rect,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextInformation>,
}

impl SinkSource {
    pub fn new(
        router_x: &u16,
        router_y: &u16,
        direction: &SinkSourceDirection,
        variant: SinkSourceVariant,
    ) -> Self {
        let mut delta_x = 0i16;
        let mut delta_y = 0i16;

        match direction {
            SinkSourceDirection::North => {
                delta_y -= I_SINKS_SOURCES_GROUP_OFFSET;
            }
            SinkSourceDirection::East => {
                delta_x += I_SINKS_SOURCES_GROUP_OFFSET;
            }
            SinkSourceDirection::South => {
                delta_y += I_SINKS_SOURCES_GROUP_OFFSET.wrapping_add_unsigned(ROUTER_OFFSET);
            }
            SinkSourceDirection::West => {
                delta_x -= I_SINKS_SOURCES_GROUP_OFFSET.wrapping_add_unsigned(ROUTER_OFFSET);
            }
        };

        let x = delta_x.wrapping_add_unsigned(*router_x);
        let y = delta_y.wrapping_add_unsigned(*router_y);

        SinkSource {
            rect: Rect::new(x, y, variant),
            text: TextInformation::sink_source_text(x, y, variant),
        }
    }
}

#[derive(Serialize)]
pub struct SinksSourcesGroup {
    #[serde(rename = "@id")]
    id: &'static str,
    g: Vec<SinkSource>,
}

impl SinksSourcesGroup {
    pub fn new(rows: u16, columns: u16) -> Self {
        Self {
            id: "sinksSources",
            // Formula worksout because we ignore the corners
            g: Vec::with_capacity(usize::from((rows + columns) * 2)),
        }
    }

    pub fn push(&mut self, element: SinkSource) {
        self.g.push(element)
    }

    pub fn clear(&mut self) {
        self.g.clear()
    }

    pub fn should_serialise(&self) -> bool {
        self.g.is_empty()
    }
}
