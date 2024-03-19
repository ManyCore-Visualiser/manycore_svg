use manycore_parser::SinkSourceDirection;
use serde::Serialize;

use crate::{
    style::DEFAULT_FILL, Connection, TextInformation, MARKER_HEIGHT, ROUTER_OFFSET, SIDE_LENGTH,
};

static SINKS_SOURCES_SIDE_LENGTH: &str = "70";
static SINKS_SOURCES_STROKE_WIDTH: &str = "1";
static SINKS_SOURCES_RX: &str = "15";
static SINKS_SOURCE_STROKE_WIDTH_VAL: u16 = 1;
static SINKS_SOURCES_SIDE_LENGTH_VAL: u16 = 70;
static SINKS_SOURCES_CONNECTION_EXTRA_LENGTH: u16 = 100;
pub static SINKS_SOURCES_GROUP_OFFSET: u16 = SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
    + SINKS_SOURCES_SIDE_LENGTH_VAL
    + SINKS_SOURCE_STROKE_WIDTH_VAL;
static I_SINKS_SOURCE_DRAWING_OFFSET: i16 =
    0i16.wrapping_add_unsigned(SINKS_SOURCES_GROUP_OFFSET - SINKS_SOURCE_STROKE_WIDTH_VAL);
static I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH: i16 = 100;
static I_SINKS_SOURCE_NORTH_SOUTH_X_OFFSET: i16 =
    0i16.wrapping_add_unsigned(SIDE_LENGTH - SINKS_SOURCES_SIDE_LENGTH_VAL) / 2;
pub static I_SINKS_SOURCES_GROUP_OFFSET: i16 =
    0i16.wrapping_add_unsigned(SINKS_SOURCES_GROUP_OFFSET);
static I_SINKS_SOURCES_HALF_SIDE_LENGTH: i16 = 35;
static I_SINKS_SOURCE_CONNECTION_SPACING: i16 = 15;
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
    path: [Connection; 2],
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
                delta_y -= I_SINKS_SOURCE_DRAWING_OFFSET;
                delta_x += I_SINKS_SOURCE_NORTH_SOUTH_X_OFFSET;
            }
            SinkSourceDirection::East => {
                delta_x +=
                    I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH.wrapping_add_unsigned(SIDE_LENGTH);
            }
            SinkSourceDirection::South => {
                delta_y += I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                    .wrapping_add_unsigned(ROUTER_OFFSET)
                    .wrapping_add_unsigned(SIDE_LENGTH);
                delta_x += I_SINKS_SOURCE_NORTH_SOUTH_X_OFFSET;
            }
            SinkSourceDirection::West => {
                delta_x -= I_SINKS_SOURCE_DRAWING_OFFSET.wrapping_add_unsigned(ROUTER_OFFSET);
            }
        };

        let x = delta_x.wrapping_add_unsigned(*router_x);
        let y = delta_y.wrapping_add_unsigned(*router_y);

        let input_connection = Connection::new(
            None,
            Connection::get_path_from_edge_router(&x, &y, true, direction),
        );

        let output_connection = Connection::new(
            None,
            Connection::get_path_from_edge_router(&x, &y, false, direction),
        );

        SinkSource {
            rect: Rect::new(x, y, variant),
            text: TextInformation::sink_source_text(x, y, variant),
            path: [input_connection, output_connection],
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

impl Connection {
    fn get_path_from_edge_router(
        x: &i16,
        y: &i16,
        is_input: bool,
        direction: &SinkSourceDirection,
    ) -> String {
        let mut start_x = *x;
        let mut start_y = *y;

        let ret: String;

        match direction {
            SinkSourceDirection::North => {
                start_x += I_SINKS_SOURCES_HALF_SIDE_LENGTH;
                let line_command;
                if is_input {
                    start_x += I_SINKS_SOURCE_CONNECTION_SPACING;
                    start_y += I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH;
                    line_command = "v-";
                } else {
                    start_x -= I_SINKS_SOURCE_CONNECTION_SPACING;
                    line_command = "v";
                }
                start_y = start_y.wrapping_add_unsigned(SINKS_SOURCES_SIDE_LENGTH_VAL);

                ret = format!(
                    "M{},{} {}{}",
                    start_x,
                    start_y,
                    line_command,
                    SINKS_SOURCES_CONNECTION_EXTRA_LENGTH - MARKER_HEIGHT
                );
            }
            SinkSourceDirection::East => {
                start_y += I_SINKS_SOURCES_HALF_SIDE_LENGTH;
                let line_command;
                if is_input {
                    line_command = "h";
                    start_y -= I_SINKS_SOURCE_CONNECTION_SPACING;
                    start_x -= I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH;
                } else {
                    line_command = "h-";
                    start_y += I_SINKS_SOURCE_CONNECTION_SPACING;
                }

                ret = format!(
                    "M{},{} {}{}",
                    start_x,
                    start_y,
                    line_command,
                    SINKS_SOURCES_CONNECTION_EXTRA_LENGTH - MARKER_HEIGHT
                );
            }
            SinkSourceDirection::South => {
                start_x += I_SINKS_SOURCES_HALF_SIDE_LENGTH;
                let line_command;
                if is_input {
                    start_x -= I_SINKS_SOURCE_CONNECTION_SPACING;
                    start_y -= I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                        .wrapping_add_unsigned(ROUTER_OFFSET);
                    line_command = "v";
                } else {
                    start_x += I_SINKS_SOURCE_CONNECTION_SPACING;
                    line_command = "v-";
                }

                ret = format!(
                    "M{},{} {}{}",
                    start_x,
                    start_y,
                    line_command,
                    SINKS_SOURCES_CONNECTION_EXTRA_LENGTH + ROUTER_OFFSET - MARKER_HEIGHT
                );
            }
            SinkSourceDirection::West => {
                start_y += I_SINKS_SOURCES_HALF_SIDE_LENGTH;
                let line_command;
                if is_input {
                    line_command = "h-";
                    start_y += I_SINKS_SOURCE_CONNECTION_SPACING;
                    start_x += I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
                        .wrapping_add_unsigned(ROUTER_OFFSET);
                } else {
                    line_command = "h";
                    start_y -= I_SINKS_SOURCE_CONNECTION_SPACING;
                }
                start_x = start_x.wrapping_add_unsigned(SINKS_SOURCES_SIDE_LENGTH_VAL);

                ret = format!(
                    "M{},{} {}{}",
                    start_x,
                    start_y,
                    line_command,
                    SINKS_SOURCES_CONNECTION_EXTRA_LENGTH + ROUTER_OFFSET - MARKER_HEIGHT
                );
            }
        }

        ret
    }
}
