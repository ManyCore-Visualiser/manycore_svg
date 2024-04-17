use std::{cmp::max, collections::HashMap};

use const_format::concatcp;
use manycore_parser::{BorderEntry, EdgePosition, SinkSourceDirection};
use serde::Serialize;

use crate::{
    style::{DEFAULT_FILL, EDGE_DATA_CLASS_NAME}, CoordinateT, ProcessedBaseConfiguration, TextInformation, CHAR_H_PADDING, HALF_ROUTER_OFFSET, MARKER_HEIGHT, ROUTER_OFFSET, SIDE_LENGTH
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
    /// Utility function to generate a [`TextInformation`] instance that contains a task id.
    fn sink_source_text(
        centre_x: CoordinateT,
        centre_y: CoordinateT,
        text_content: Option<String>,
        processed_base_configuration: &ProcessedBaseConfiguration,
    ) -> Option<Self> {
        if let Some(text) = text_content {
            return Some(TextInformation::new(
                centre_x,
                centre_y,
                *processed_base_configuration.task_font_size(),
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

/// Object representation of an SVG `<rect>`, the "shell" of an edge router.
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
    /// Generates a new [`Rect`] instance from the given parameters.
    fn new(
        centre_x: CoordinateT,
        centre_y: CoordinateT,
        variant: SinkSourceVariant,
        text_width: CoordinateT,
    ) -> Self {
        Self {
            x: centre_x.saturating_sub(text_width.saturating_div(2)),
            y: centre_y - SINKS_SOURCES_HALF_SHORT_SIDE_LENGTH,
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

/// Enum to describe an router variant. Variant content is task id, if any.
#[derive(Clone, Copy)]
pub(crate) enum SinkSourceVariant {
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

/// Object representation of an SVG border router `<g>`.
#[derive(Serialize)]
pub(crate) struct SinkSource {
    rect: Rect,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<TextInformation>,
}

impl SinkSource {
    /// Generates a new [`SinkSource`] instance according to the provided parameters.
    pub(crate) fn new(
        router_x: &CoordinateT,
        router_y: &CoordinateT,
        direction: &SinkSourceDirection,
        variant: SinkSourceVariant,
        processed_base_configuration: &ProcessedBaseConfiguration,
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
                    *processed_base_configuration.task_font_size(),
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

        let centre_x = delta_x.wrapping_add(*router_x);
        let centre_y = delta_y.wrapping_add(*router_y);

        SinkSource {
            rect: Rect::new(centre_x, centre_y, variant, text_width),
            text: TextInformation::sink_source_text(
                centre_x,
                centre_y,
                text_content,
                processed_base_configuration,
            ),
        }
    }
}

/// Object representation of an SVG `<g>` that contains all instance of [`SinkSource`].
#[derive(Serialize)]
pub(crate) struct SinksSourcesGroup {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "@class")]
    class: &'static str,
    g: Vec<SinkSource>,
}

impl SinksSourcesGroup {
    /// Generates a new [`SinksSourcesGroup`] instance with capacity for all border router,
    /// calculated fom the number of rows and columns.
    pub(crate) fn new(rows: u8, columns: u8) -> Self {
        Self {
            id: SINK_SOURCES_ID,
            class: EDGE_DATA_CLASS_NAME,
            // Formula worksout because we ignore the corners. Obv, only on 2D matrix.
            g: Vec::with_capacity(usize::from((rows + columns) * 2)),
        }
    }

    /// Utility to retrieve edge router variant.
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

    /// Generates and inserts all edge routers ([`SinkSource`]s) connected to the [`Router`] located at the given coordinates.
    pub(crate) fn insert(
        &mut self,
        edge_position: &EdgePosition,
        router_x: &CoordinateT,
        router_y: &CoordinateT,
        core_borders: Option<&HashMap<SinkSourceDirection, BorderEntry>>,
        processed_base_configuration: &ProcessedBaseConfiguration,
    ) {
        let mut variant;
        let mut direction;

        // We generate the required edge routers based on the edge position.
        // Some edges have two routers connected, while most have one.
        match edge_position {
            EdgePosition::Top => {
                direction = SinkSourceDirection::North;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));
            }
            EdgePosition::TopLeft => {
                direction = SinkSourceDirection::North;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));

                direction = SinkSourceDirection::West;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));
            }
            EdgePosition::TopRight => {
                direction = SinkSourceDirection::North;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));

                direction = SinkSourceDirection::East;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));
            }
            EdgePosition::Left => {
                direction = SinkSourceDirection::West;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));
            }
            EdgePosition::Right => {
                direction = SinkSourceDirection::East;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));
            }
            EdgePosition::Bottom => {
                direction = SinkSourceDirection::South;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));
            }
            EdgePosition::BottomLeft => {
                direction = SinkSourceDirection::South;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));

                direction = SinkSourceDirection::West;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));
            }
            EdgePosition::BottomRight => {
                direction = SinkSourceDirection::South;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));

                direction = SinkSourceDirection::East;
                variant = self.get_variant(core_borders, direction);

                self.g.push(SinkSource::new(
                    router_x,
                    router_y,
                    &direction,
                    variant,
                    processed_base_configuration,
                ));
            }
        }
    }
}
