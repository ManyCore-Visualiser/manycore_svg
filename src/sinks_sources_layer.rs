use const_format::concatcp;
use manycore_parser::{EdgePosition, SinkSourceDirection};
use serde::Serialize;

use crate::{
    style::{DEFAULT_FILL, EDGE_DATA_CLASS_NAME},
    HALF_ROUTER_OFFSET, MARKER_HEIGHT, ROUTER_OFFSET, SIDE_LENGTH,
};

// Side lengths
const SINKS_SOURCES_SIDE_LENGTH: u16 = 70;
static I_SINKS_SOURCES_SIDE_LENGTH: i16 = 0i16.saturating_add_unsigned(SINKS_SOURCES_SIDE_LENGTH);
static I_SINKS_SOURCES_HALF_SIDE_LENGTH: i16 = I_SINKS_SOURCES_SIDE_LENGTH.saturating_div(2);
static SINKS_SOURCES_SIDE_LENGTH_STR: &'static str = concatcp!(SINKS_SOURCES_SIDE_LENGTH);

// Stroke
const SINKS_SOURCES_STROKE_WIDTH: u16 = 1;
static SINKS_SOURCES_STROKE_WIDTH_STR: &'static str = concatcp!(SINKS_SOURCES_STROKE_WIDTH);
static SINKS_SOURCES_RX: &str = "15";

// Connection
pub static SINKS_SOURCES_CONNECTION_EXTRA_LENGTH: u16 = 100;
static I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH: i16 =
    0i16.saturating_add_unsigned(SINKS_SOURCES_CONNECTION_EXTRA_LENGTH);

// Viewbox Offset
pub static SINKS_SOURCES_GROUP_OFFSET: u16 = SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
    .saturating_add(SINKS_SOURCES_SIDE_LENGTH)
    .saturating_add(SINKS_SOURCES_STROKE_WIDTH)
    .saturating_add(MARKER_HEIGHT);
pub static I_SINKS_SOURCES_GROUP_OFFSET: i16 =
    0i16.wrapping_add_unsigned(SINKS_SOURCES_GROUP_OFFSET);

// static SINK_FILL: &str = "#fb923c";
// static SOURCE_FILL: &str = "#fbbf24";

pub const SINK_SOURCES_ID: &'static str = "sinksSources";

// Comenting out the variant stuff as some could be both source/sinks so this doessn't really work

// impl TextInformation {
//     fn sink_source_text(sink_x: i16, sink_y: i16, variant: SinkSourceVariant) -> Option<Self> {
//         let text = match variant {
//             SinkSourceVariant::Sink => Some(String::from("Sink")),
//             SinkSourceVariant::Source => Some(String::from("Source")),
//             SinkSourceVariant::None => None,
//         };

//         if let Some(text) = text {
//             return Some(TextInformation::new_signed(
//                 sink_x + I_SINKS_SOURCES_HALF_SIDE_LENGTH,
//                 sink_y + I_SINKS_SOURCES_HALF_SIDE_LENGTH,
//                 "middle",
//                 "middle",
//                 None,
//                 text,
//             ));
//         }

//         None
//     }
// }

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
    fn new(x: i16, y: i16 /*, variant: SinkSourceVariant */) -> Self {
        Self {
            x,
            y,
            width: SINKS_SOURCES_SIDE_LENGTH_STR,
            height: SINKS_SOURCES_SIDE_LENGTH_STR,
            rx: SINKS_SOURCES_RX,
            fill: DEFAULT_FILL, /*match variant {
                                    SinkSourceVariant::Source => SOURCE_FILL,
                                    SinkSourceVariant::Sink => SINK_FILL,
                                    SinkSourceVariant::None => DEFAULT_FILL,
                                }*/
            stroke: "black",
            stroke_width: SINKS_SOURCES_STROKE_WIDTH_STR,
        }
    }
}

// #[derive(Clone, Copy)]
// pub enum SinkSourceVariant {
//     Sink,
//     Source,
//     None,
// }

static NORTH_DELTA_Y: i16 = 0i16
    .wrapping_sub(I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH)
    .wrapping_sub(I_SINKS_SOURCES_SIDE_LENGTH)
    .wrapping_sub_unsigned(ROUTER_OFFSET)
    .wrapping_sub_unsigned(MARKER_HEIGHT);

static SOUTH_DELTA_Y: i16 = 0i16
    .wrapping_add_unsigned(SIDE_LENGTH)
    .wrapping_add(I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH)
    .wrapping_add_unsigned(MARKER_HEIGHT);

static NORTH_SOUTH_DELTA_X: i16 = 0i16
    .wrapping_add_unsigned(SIDE_LENGTH)
    .wrapping_sub_unsigned(HALF_ROUTER_OFFSET)
    .wrapping_sub(I_SINKS_SOURCES_HALF_SIDE_LENGTH);

static EAST_WEST_DELTA_Y: i16 = 0i16
    .wrapping_sub_unsigned(ROUTER_OFFSET)
    .wrapping_add_unsigned(HALF_ROUTER_OFFSET)
    .wrapping_sub(I_SINKS_SOURCES_HALF_SIDE_LENGTH);

static EAST_DELTA_X: i16 = 0i16
    .wrapping_add(I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH)
    .wrapping_add_unsigned(SIDE_LENGTH)
    .wrapping_add_unsigned(MARKER_HEIGHT);

static WEST_DELTA_X: i16 = 0i16
    .wrapping_sub(I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH)
    .wrapping_sub_unsigned(MARKER_HEIGHT)
    .wrapping_sub_unsigned(ROUTER_OFFSET)
    .wrapping_sub(I_SINKS_SOURCES_SIDE_LENGTH);

#[derive(Serialize)]
pub struct SinkSource {
    rect: Rect,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // text: Option<TextInformation>,
}

impl SinkSource {
    pub fn new(
        router_x: &u16,
        router_y: &u16,
        direction: &SinkSourceDirection,
        // variant: SinkSourceVariant,
    ) -> Self {
        let delta_x;
        let delta_y;

        match direction {
            SinkSourceDirection::North => {
                delta_y = NORTH_DELTA_Y;
                delta_x = NORTH_SOUTH_DELTA_X;
            }
            SinkSourceDirection::East => {
                delta_y = EAST_WEST_DELTA_Y;
                delta_x = EAST_DELTA_X;
            }
            SinkSourceDirection::South => {
                delta_y = SOUTH_DELTA_Y;
                delta_x = NORTH_SOUTH_DELTA_X;
            }
            SinkSourceDirection::West => {
                delta_y = EAST_WEST_DELTA_Y;
                delta_x = WEST_DELTA_X;
            }
        };

        let x = delta_x.wrapping_add_unsigned(*router_x);
        let y = delta_y.wrapping_add_unsigned(*router_y);

        SinkSource {
            rect: Rect::new(x, y), //, variant),
                                   // text: TextInformation::sink_source_text(x, y, variant),
        }
    }
}

#[derive(Serialize)]
pub struct SinksSourcesGroup {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "@class")]
    class: &'static str,
    g: Vec<SinkSource>,
}

impl SinksSourcesGroup {
    pub fn new(rows: u8, columns: u8) -> Self {
        Self {
            id: SINK_SOURCES_ID,
            class: EDGE_DATA_CLASS_NAME,
            // Formula worksout because we ignore the corners
            g: Vec::with_capacity(usize::from((rows + columns) * 2)),
        }
    }

    pub fn insert(&mut self, edge_position: EdgePosition, router_x: &u16, router_y: &u16) {
        match edge_position {
            EdgePosition::Top => {
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::North,
                ));
            }
            EdgePosition::TopLeft => {
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::North,
                ));
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::West,
                ));
            }
            EdgePosition::TopRight => {
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::North,
                ));
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::East,
                ));
            }
            EdgePosition::Left => {
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::West,
                ));
            }
            EdgePosition::Right => {
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::East,
                ));
            }
            EdgePosition::Bottom => {
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::South,
                ));
            }
            EdgePosition::BottomLeft => {
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::South,
                ));
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::West,
                ));
            }
            EdgePosition::BottomRight => {
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::South,
                ));
                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &SinkSourceDirection::East,
                ));
            }
        }
    }
}

// impl Connection {
//     fn get_path_from_edge_router(
//         x: &i16,
//         y: &i16,
//         is_input: bool,
//         direction: &SinkSourceDirection,
//     ) -> String {
//         let mut start_x = *x;
//         let mut start_y = *y;

//         let ret: String;

//         match direction {
//             SinkSourceDirection::North => {
//                 start_x += I_SINKS_SOURCES_HALF_SIDE_LENGTH;
//                 let line_command;
//                 if is_input {
//                     start_x += I_SINKS_SOURCE_CONNECTION_SPACING;
//                     start_y += I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH;
//                     line_command = "v-";
//                 } else {
//                     start_x -= I_SINKS_SOURCE_CONNECTION_SPACING;
//                     line_command = "v";
//                 }
//                 start_y = start_y.wrapping_add_unsigned(SINKS_SOURCES_SIDE_LENGTH_VAL);

//                 ret = format!(
//                     "M{},{} {}{}",
//                     start_x,
//                     start_y,
//                     line_command,
//                     SINKS_SOURCES_CONNECTION_EXTRA_LENGTH - MARKER_HEIGHT
//                 );
//             }
//             SinkSourceDirection::East => {
//                 start_y += I_SINKS_SOURCES_HALF_SIDE_LENGTH;
//                 let line_command;
//                 if is_input {
//                     line_command = "h";
//                     start_y -= I_SINKS_SOURCE_CONNECTION_SPACING;
//                     start_x -= I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH;
//                 } else {
//                     line_command = "h-";
//                     start_y += I_SINKS_SOURCE_CONNECTION_SPACING;
//                 }

//                 ret = format!(
//                     "M{},{} {}{}",
//                     start_x,
//                     start_y,
//                     line_command,
//                     SINKS_SOURCES_CONNECTION_EXTRA_LENGTH - MARKER_HEIGHT
//                 );
//             }
//             SinkSourceDirection::South => {
//                 start_x += I_SINKS_SOURCES_HALF_SIDE_LENGTH;
//                 let line_command;
//                 if is_input {
//                     start_x -= I_SINKS_SOURCE_CONNECTION_SPACING;
//                     start_y -= I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
//                         .wrapping_add_unsigned(ROUTER_OFFSET);
//                     line_command = "v";
//                 } else {
//                     start_x += I_SINKS_SOURCE_CONNECTION_SPACING;
//                     line_command = "v-";
//                 }

//                 ret = format!(
//                     "M{},{} {}{}",
//                     start_x,
//                     start_y,
//                     line_command,
//                     SINKS_SOURCES_CONNECTION_EXTRA_LENGTH + ROUTER_OFFSET - MARKER_HEIGHT
//                 );
//             }
//             SinkSourceDirection::West => {
//                 start_y += I_SINKS_SOURCES_HALF_SIDE_LENGTH;
//                 let line_command;
//                 if is_input {
//                     line_command = "h-";
//                     start_y += I_SINKS_SOURCE_CONNECTION_SPACING;
//                     start_x += I_SINKS_SOURCES_CONNECTION_EXTRA_LENGTH
//                         .wrapping_add_unsigned(ROUTER_OFFSET);
//                 } else {
//                     line_command = "h";
//                     start_y -= I_SINKS_SOURCE_CONNECTION_SPACING;
//                 }
//                 start_x = start_x.wrapping_add_unsigned(SINKS_SOURCES_SIDE_LENGTH_VAL);

//                 ret = format!(
//                     "M{},{} {}{}",
//                     start_x,
//                     start_y,
//                     line_command,
//                     SINKS_SOURCES_CONNECTION_EXTRA_LENGTH + ROUTER_OFFSET - MARKER_HEIGHT
//                 );
//             }
//         }

//         ret
//     }
// }
