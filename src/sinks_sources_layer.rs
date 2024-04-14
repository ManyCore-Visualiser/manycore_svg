use std::{cmp::max, collections::HashMap};

use const_format::concatcp;
use manycore_parser::{BorderEntry, EdgePosition, SinkSourceDirection};
use serde::Serialize;

use crate::{
    style::{DEFAULT_FILL, EDGE_DATA_CLASS_NAME},
    CoordinateT, TextInformation, CHAR_H_PADDING, HALF_ROUTER_OFFSET, MARKER_HEIGHT, ROUTER_OFFSET,
    SIDE_LENGTH, TASK_FONT_SIZE,
};

// Side lengths
const SINKS_SOURCES_SHORT_SIDE_LENGTH: CoordinateT = 70;
static SINKS_SOURCES_HALF_SHORT_SIDE_LENGTH: CoordinateT =
    SINKS_SOURCES_SHORT_SIDE_LENGTH.saturating_div(2);
static SINKS_SOURCES_SHORT_SIDE_LENGTH_STR: &'static str =
    concatcp!(SINKS_SOURCES_SHORT_SIDE_LENGTH);

// Stroke
const SINKS_SOURCES_STROKE_WIDTH: CoordinateT = 1;
static SINKS_SOURCES_STROKE_WIDTH_STR: &'static str = concatcp!(SINKS_SOURCES_STROKE_WIDTH);
static SINKS_SOURCES_RX: &str = "15";

// Connection
pub static SINKS_SOURCES_CONNECTION_LENGTH: CoordinateT = ROUTER_OFFSET.saturating_mul(3);

// Viewbox Offset
pub static SINKS_SOURCES_GROUP_OFFSET: CoordinateT = SINKS_SOURCES_CONNECTION_LENGTH
    .saturating_add(SINKS_SOURCES_SHORT_SIDE_LENGTH)
    .saturating_add(SINKS_SOURCES_STROKE_WIDTH)
    .saturating_add(MARKER_HEIGHT);

static SINK_FILL: &str = "#fb923c";
static SOURCE_FILL: &str = "#fbbf24";

pub const SINK_SOURCES_ID: &'static str = "sinksSources";

impl TextInformation {
    fn sink_source_text(
        cx: CoordinateT,
        cy: CoordinateT,
        text_content: Option<String>,
    ) -> Option<Self> {
        if let Some(text) = text_content {
            return Some(TextInformation::new(
                cx,
                cy,
                TASK_FONT_SIZE,
                "middle",
                "central",
                None,
                Some(EDGE_DATA_CLASS_NAME),
                text,
            ));
        }

        None
    }
}

#[derive(Serialize)]
struct Rect {
    #[serde(rename = "@x")]
    x: i32,
    #[serde(rename = "@y")]
    y: i32,
    #[serde(rename = "@width")]
    width: CoordinateT,
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
    fn new(
        x: CoordinateT,
        y: CoordinateT,
        variant: SinkSourceVariant,
        text_width: CoordinateT,
    ) -> Self {
        Self {
            x: x.saturating_sub(text_width.saturating_div(2)),
            y: y - SINKS_SOURCES_HALF_SHORT_SIDE_LENGTH,
            width: text_width,
            height: SINKS_SOURCES_SHORT_SIDE_LENGTH_STR,
            rx: SINKS_SOURCES_RX,
            fill: match variant {
                SinkSourceVariant::Source(_) => SOURCE_FILL,
                SinkSourceVariant::Sink(_) => SINK_FILL,
                SinkSourceVariant::None => DEFAULT_FILL,
            },
            stroke: "black",
            stroke_width: SINKS_SOURCES_STROKE_WIDTH_STR,
        }
    }
}

static NORTH_DELTA_Y: CoordinateT = 0i32
    .saturating_sub(SINKS_SOURCES_CONNECTION_LENGTH)
    .saturating_sub(SINKS_SOURCES_HALF_SHORT_SIDE_LENGTH)
    .saturating_sub(ROUTER_OFFSET)
    .saturating_sub(MARKER_HEIGHT);

static SOUTH_DELTA_Y: CoordinateT = 0i32
    .saturating_add(SIDE_LENGTH)
    .saturating_add(SINKS_SOURCES_CONNECTION_LENGTH)
    .saturating_add(MARKER_HEIGHT)
    .saturating_add(SINKS_SOURCES_HALF_SHORT_SIDE_LENGTH);

static NORTH_SOUTH_DELTA_X: CoordinateT = 0i32
    .saturating_add(SIDE_LENGTH)
    .saturating_sub(HALF_ROUTER_OFFSET);

static EAST_WEST_DELTA_Y: CoordinateT = 0i32
    .saturating_sub(ROUTER_OFFSET)
    .saturating_add(HALF_ROUTER_OFFSET);

static EAST_DELTA_X: CoordinateT = 0i32
    .saturating_add(SINKS_SOURCES_CONNECTION_LENGTH)
    .saturating_add(SIDE_LENGTH)
    .saturating_add(MARKER_HEIGHT);

static WEST_DELTA_X: CoordinateT = 0i32
    .saturating_sub(SINKS_SOURCES_CONNECTION_LENGTH)
    .saturating_sub(MARKER_HEIGHT)
    .saturating_sub(ROUTER_OFFSET);

#[derive(Clone, Copy)]
pub enum SinkSourceVariant {
    Sink(u16),
    Source(u16),
    None,
}

impl From<&BorderEntry> for SinkSourceVariant {
    fn from(input: &BorderEntry) -> Self {
        match input {
            BorderEntry::Sink(task_id) => SinkSourceVariant::Sink(*task_id),
            BorderEntry::Source(task_id) => SinkSourceVariant::Source(*task_id),
        }
    }
}

#[derive(Serialize)]
pub struct SinkSource {
    rect: Rect,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextInformation>,
}

impl SinkSource {
    pub fn new(
        router_x: &CoordinateT,
        router_y: &CoordinateT,
        direction: &SinkSourceDirection,
        variant: SinkSourceVariant,
    ) -> Self {
        let delta_x;
        let delta_y;

        let text_content = match variant {
            SinkSourceVariant::Sink(task_id) => Some(format!("T{}", task_id)),
            SinkSourceVariant::Source(task_id) => Some(format!("T{}", task_id)),
            SinkSourceVariant::None => None,
        };

        let text_width = match text_content.as_ref() {
            // TODO: This should bubble up error
            Some(text_content) => max(
                TextInformation::calculate_length_util(
                    TASK_FONT_SIZE,
                    text_content.len(),
                    Some(CHAR_H_PADDING),
                )
                .unwrap_or(SINKS_SOURCES_SHORT_SIDE_LENGTH),
                SINKS_SOURCES_SHORT_SIDE_LENGTH,
            ),
            None => SINKS_SOURCES_SHORT_SIDE_LENGTH,
        };

        match direction {
            SinkSourceDirection::North => {
                delta_y = NORTH_DELTA_Y;
                delta_x = NORTH_SOUTH_DELTA_X;
            }
            SinkSourceDirection::East => {
                delta_y = EAST_WEST_DELTA_Y;
                delta_x = EAST_DELTA_X.saturating_add(text_width.saturating_div(2));
            }
            SinkSourceDirection::South => {
                delta_y = SOUTH_DELTA_Y;
                delta_x = NORTH_SOUTH_DELTA_X;
            }
            SinkSourceDirection::West => {
                delta_y = EAST_WEST_DELTA_Y;
                delta_x = WEST_DELTA_X.saturating_sub(text_width.saturating_div(2));
            }
        };

        let cx = delta_x.wrapping_add(*router_x);
        let cy = delta_y.wrapping_add(*router_y);

        SinkSource {
            rect: Rect::new(cx, cy, variant, text_width),
            text: TextInformation::sink_source_text(cx, cy, text_content),
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

    fn get_variant(
        &self,
        core_borders: Option<&HashMap<SinkSourceDirection, BorderEntry>>,
        direction: SinkSourceDirection,
    ) -> SinkSourceVariant {
        match core_borders {
            Some(core_borders_map) => match core_borders_map.get(&direction) {
                Some(border) => border.into(),
                None => SinkSourceVariant::None,
            },
            None => SinkSourceVariant::None,
        }
    }

    pub fn insert(
        &mut self,
        edge_position: EdgePosition,
        router_x: &CoordinateT,
        router_y: &CoordinateT,
        core_borders: Option<&HashMap<SinkSourceDirection, BorderEntry>>,
    ) {
        let mut variant;
        let mut direction;

        match edge_position {
            EdgePosition::Top => {
                direction = SinkSourceDirection::North;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));
            }
            EdgePosition::TopLeft => {
                direction = SinkSourceDirection::North;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));

                direction = SinkSourceDirection::West;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));
            }
            EdgePosition::TopRight => {
                direction = SinkSourceDirection::North;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));

                direction = SinkSourceDirection::East;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));
            }
            EdgePosition::Left => {
                direction = SinkSourceDirection::West;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));
            }
            EdgePosition::Right => {
                direction = SinkSourceDirection::East;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));
            }
            EdgePosition::Bottom => {
                direction = SinkSourceDirection::South;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));
            }
            EdgePosition::BottomLeft => {
                direction = SinkSourceDirection::South;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));

                direction = SinkSourceDirection::West;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));
            }
            EdgePosition::BottomRight => {
                direction = SinkSourceDirection::South;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));

                direction = SinkSourceDirection::East;
                variant = self.get_variant(core_borders, direction);

                self.g
                    .push(SinkSource::new(router_x, router_y, &direction, variant));
            }
        }
    }
}
