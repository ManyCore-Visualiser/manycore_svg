use std::ops::Div;

use manycore_parser::Directions;
use serde::Serialize;

use crate::{
    sinks_sources_layer::SINKS_SOURCES_CONNECTION_LENGTH, style::EDGE_DATA_CLASS_NAME, CoordinateT,
    FieldConfiguration, LoadConfiguration, Offsets, RoutingConfiguration, SVGError,
    CONNECTION_LENGTH, MARKER_HEIGHT, ROUTER_OFFSET,
};

use super::utils;

static HORIZONTAL_OFFSET_FROM_LINK: CoordinateT = 5;
static VERTICAL_OFFSET_FROM_LINK: CoordinateT = 1;
static OFFSET_FROM_FIRST: CoordinateT = 20;
static HALF_CONNECTION_LENGTH: CoordinateT = CONNECTION_LENGTH
    .saturating_add(MARKER_HEIGHT)
    .saturating_div(2);
pub(crate) static CHAR_HEIGHT_AT_22_PX: CoordinateT = 30;
pub(crate) static CHAR_V_PADDING: CoordinateT = 3;
pub(crate) static CHAR_H_PADDING: f32 = 2.0;
pub(crate) static DEFAULT_FONT_SIZE: f32 = 16.0;

static ROBOTO_RATIO: f32 = 1.665;

/// Wrapper around font size
pub(crate) struct FontSize {
    px: f32,
}

impl Serialize for FontSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // This conversion will truncate the font size as an f32 does not fit in a u32.
        // However, values that big should not be provided.
        serializer.serialize_str(format!("{}px", self.px as u32).as_str())
    }
}

/// Object representation of an SVG `<text>` element.
#[derive(Serialize)]
pub struct TextInformation {
    #[serde(rename = "@x")]
    x: CoordinateT,
    #[serde(rename = "@y")]
    y: CoordinateT,
    #[serde(rename = "@font-size")]
    font_size: FontSize,
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
    /// Calculates the approximate length in pixels of a `<text>` element.
    pub(crate) fn calculate_length_util(
        font_size: f32,
        length: usize,
        pad: Option<f32>,
    ) -> Result<CoordinateT, SVGError> {
        let char_width = font_size.div(ROBOTO_RATIO);

        Ok((char_width * u16::try_from(length)? as f32
            + if let Some(pad) = pad {
                char_width * pad
            } else {
                0.0
            })
        .round() as CoordinateT)
    }

    /// Calculates the apprroximate length in pixels of a [`TextInformation`] instance.
    pub(crate) fn calculate_length(&self, pad: Option<f32>) -> Result<CoordinateT, SVGError> {
        TextInformation::calculate_length_util(self.font_size.px, self.value.len(), pad)
    }

    /// Creates a new [`TextInformation`] instance from the given parameters.
    pub(crate) fn new(
        x: CoordinateT,
        y: CoordinateT,
        font_size: f32,
        text_anchor: &'static str,
        dominant_baseline: &'static str,
        fill: Option<&String>,
        class: Option<&'static str>,
        value: String,
    ) -> Self {
        Self {
            x,
            y,
            font_size: FontSize { px: font_size },
            font_family: "Roboto Mono",
            text_anchor,
            dominant_baseline,
            fill: match fill {
                Some(f) => f.clone(),
                None => "black".to_string(),
            },
            class,
            value,
        }
    }

    /// Shared logic used when generating "primary" [`TextInformation`] for a channel.
    /// The `relevant_delta` can either be x orr y and is chosen depending on `direction`.
    fn common_channel_primary(
        link_x: &CoordinateT,
        link_y: &CoordinateT,
        direction: &Directions,
        relevant_delta: CoordinateT,
        fill: Option<&String>,
        class: Option<&'static str>,
        data: String,
    ) -> Self {
        match direction {
            Directions::North => {
                let delta_y = relevant_delta;

                TextInformation::new(
                    link_x.saturating_add(HORIZONTAL_OFFSET_FROM_LINK),
                    link_y.saturating_sub(delta_y),
                    DEFAULT_FONT_SIZE,
                    "start",
                    "middle",
                    fill,
                    class,
                    data,
                )
            }
            Directions::East => {
                let delta_x = relevant_delta;

                TextInformation::new(
                    link_x.saturating_add(delta_x),
                    link_y.saturating_sub(VERTICAL_OFFSET_FROM_LINK),
                    DEFAULT_FONT_SIZE,
                    "middle",
                    "text-after-edge",
                    fill,
                    class,
                    data,
                )
            }
            Directions::South => {
                let delta_y = relevant_delta;

                TextInformation::new(
                    link_x.saturating_sub(HORIZONTAL_OFFSET_FROM_LINK),
                    link_y.saturating_add(delta_y),
                    DEFAULT_FONT_SIZE,
                    "end",
                    "middle",
                    fill,
                    class,
                    data,
                )
            }
            Directions::West => {
                let delta_x = relevant_delta;

                TextInformation::new(
                    link_x.saturating_sub(delta_x),
                    link_y.saturating_sub(VERTICAL_OFFSET_FROM_LINK),
                    DEFAULT_FONT_SIZE,
                    "middle",
                    "text-after-edge",
                    fill,
                    class,
                    data,
                )
            }
        }
    }

    /// Calculates the fill and load percentage of a channel.
    fn calculate_load_fill_and_percentage<'a>(
        load: &u16,
        bandwidth: &u16,
        routing_configuration: &'a RoutingConfiguration,
    ) -> (Option<u16>, Option<&'a String>) {
        if *bandwidth > 0 {
            // We can only calculaye load percentage if the bandwidth is above 0.
            let percentage = ((f32::from(*load) / f32::from(*bandwidth)) * 100.0).round() as u16;

            let fill_idx = utils::binary_search_left_insertion_point(
                routing_configuration.load_colours().bounds(),
                percentage.into(),
            );

            let fill = &routing_configuration.load_colours().colours()[fill_idx];
            return (Some(percentage), Some(fill));
        } else {
            // If we can't calculate a load percentage, the channel is overloaded so we pick the last colour.
            return (
                None,
                Some(&routing_configuration.load_colours().colours()[3]),
            );
        }
    }

    /// Generates the text to display for a channel's load based on user provided configuration (`routing_configuration`).
    fn generate_load_data(
        load: &u16,
        bandwidth: &u16,
        percentage: Option<u16>,
        routing_configuration: &RoutingConfiguration,
    ) -> String {
        // Does the user want fraction or percentage?
        match routing_configuration.load_configuration() {
            LoadConfiguration::Percentage => match percentage {
                Some(value) => format!("{}: {}%", routing_configuration.display(), value),
                // We can't give them a percentage for a channel with no bandwidth -> default to fraction.
                None => format!(
                    "{}: {}/{}",
                    routing_configuration.display(),
                    load,
                    bandwidth
                ),
            },
            LoadConfiguration::Fraction => format!(
                "{}: {}/{}",
                routing_configuration.display(),
                load,
                bandwidth
            ),
        }
    }

    /// Generates [`TextInformation`] for a source load.
    pub(crate) fn source_load(
        direction: &Directions,
        link_x: &CoordinateT,
        link_y: &CoordinateT,
        load: &u16,
        bandwidth: &u16,
        routing_configuration: &RoutingConfiguration,
    ) -> Self {
        let relevant_delta: i32 = match direction {
            Directions::South | Directions::West => SINKS_SOURCES_CONNECTION_LENGTH
                .saturating_add(MARKER_HEIGHT)
                .saturating_div(2),
            _ => SINKS_SOURCES_CONNECTION_LENGTH
                .saturating_add(ROUTER_OFFSET)
                .saturating_add(MARKER_HEIGHT)
                .saturating_div(2),
        };

        let (percentage, fill) = TextInformation::calculate_load_fill_and_percentage(
            load,
            bandwidth,
            routing_configuration,
        );
        let data =
            TextInformation::generate_load_data(load, bandwidth, percentage, routing_configuration);

        TextInformation::common_channel_primary(
            link_x,
            link_y,
            direction,
            relevant_delta,
            fill,
            Some(EDGE_DATA_CLASS_NAME),
            data,
        )
    }

    /// Calculates the coordinate delta and required class for a link data.
    fn link_delta_and_class(
        edge: bool,
        direction: &Directions,
    ) -> (CoordinateT, Option<&'static str>) {
        if edge {
            return match direction {
                Directions::North | Directions::East => (
                    SINKS_SOURCES_CONNECTION_LENGTH
                        .saturating_add(MARKER_HEIGHT)
                        .saturating_div(2),
                    Some(EDGE_DATA_CLASS_NAME),
                ),
                _ => (
                    SINKS_SOURCES_CONNECTION_LENGTH
                        .saturating_add(ROUTER_OFFSET)
                        .saturating_add(MARKER_HEIGHT)
                        .saturating_div(2),
                    Some(EDGE_DATA_CLASS_NAME),
                ),
            };
        }

        (HALF_CONNECTION_LENGTH, None)
    }

    /// Generates [`TextInformation`] for an inner link load.
    pub(crate) fn link_load(
        direction: &Directions,
        link_x: &CoordinateT,
        link_y: &CoordinateT,
        load: &u16,
        bandwidth: &u16,
        edge: bool,
        routing_configuration: &RoutingConfiguration,
    ) -> Self {
        let (relevant_delta, class) = TextInformation::link_delta_and_class(edge, direction);

        let (percentage, fill) = TextInformation::calculate_load_fill_and_percentage(
            load,
            bandwidth,
            routing_configuration,
        );
        let data =
            TextInformation::generate_load_data(load, bandwidth, percentage, routing_configuration);

        TextInformation::common_channel_primary(
            link_x,
            link_y,
            direction,
            relevant_delta,
            fill,
            class,
            data,
        )
    }

    /// Generates [`TextInformation`] for a channel field as primary channel text attribute.
    pub(crate) fn link_primary(
        direction: &Directions,
        link_x: &CoordinateT,
        link_y: &CoordinateT,
        data: &String,
        edge: bool,
        field_configuration: &FieldConfiguration,
    ) -> Self {
        let (relevant_delta, class) = TextInformation::link_delta_and_class(edge, direction);

        let (fill, data) = match field_configuration {
            FieldConfiguration::ColouredText(display_key, colour_config) => (
                utils::get_attribute_colour(colour_config.bounds(), colour_config.colours(), data),
                format!("{}: {}", display_key, data),
            ),
            FieldConfiguration::Text(display_key) => (None, format!("{}: {}", display_key, data)),
            _ => (None, "".into()), // Unsupported
        };

        TextInformation::common_channel_primary(
            link_x,
            link_y,
            direction,
            relevant_delta,
            fill,
            class,
            data,
        )
    }

    /// Generates [`TextInformation`] for a channel field as secondary channel text attribute.
    pub(crate) fn link_secondary(
        direction: &Directions,
        link_x: &CoordinateT,
        link_y: &CoordinateT,
        data: &String,
        field_configuration: &FieldConfiguration,
    ) -> Self {
        // This function is called only for non edge links. Core output edge links cannot have secondary information, even if present,
        // due to simmetry. The information would be missing on their input counterpart as that channel is not present in the XML file.
        let (relevant_delta, class) = TextInformation::link_delta_and_class(false, direction);

        let (fill, display) = match field_configuration {
            FieldConfiguration::ColouredText(display_key, colour_config) => (
                utils::get_attribute_colour(colour_config.bounds(), colour_config.colours(), data),
                format!("{}: {}", display_key, data),
            ),
            FieldConfiguration::Text(display_key) => (None, format!("{}: {}", display_key, data)),
            _ => (None, "".into()), // Any other variant shouldn't be used.
        };

        match direction {
            Directions::North => {
                let delta_y = relevant_delta;

                TextInformation::new(
                    link_x.saturating_add(HORIZONTAL_OFFSET_FROM_LINK),
                    link_y
                        .saturating_sub(delta_y)
                        .saturating_add(OFFSET_FROM_FIRST),
                    DEFAULT_FONT_SIZE,
                    "start",
                    "middle",
                    fill,
                    class,
                    display,
                )
            }
            Directions::East => {
                let delta_x = relevant_delta;

                TextInformation::new(
                    link_x.saturating_add(delta_x),
                    link_y.saturating_add(VERTICAL_OFFSET_FROM_LINK),
                    DEFAULT_FONT_SIZE,
                    "middle",
                    "text-before-edge",
                    fill,
                    class,
                    display,
                )
            }
            Directions::South => {
                let delta_y = relevant_delta;

                TextInformation::new(
                    link_x.saturating_sub(HORIZONTAL_OFFSET_FROM_LINK),
                    link_y
                        .saturating_add(delta_y)
                        .saturating_add(OFFSET_FROM_FIRST),
                    DEFAULT_FONT_SIZE,
                    "end",
                    "middle",
                    fill,
                    class,
                    display,
                )
            }
            Directions::West => {
                let delta_x = relevant_delta;

                TextInformation::new(
                    link_x.saturating_sub(delta_x),
                    link_y.saturating_add(VERTICAL_OFFSET_FROM_LINK),
                    DEFAULT_FONT_SIZE,
                    "middle",
                    "text-before-edge",
                    fill,
                    class,
                    display,
                )
            }
        }
    }
}

impl TryFrom<&TextInformation> for Offsets {
    type Error = SVGError;

    fn try_from(value: &TextInformation) -> Result<Self, Self::Error> {
        Ok(Offsets::new(
            value.x,
            value.y,
            value.x.saturating_add(value.calculate_length(None)?),
            value.y.saturating_add(CHAR_HEIGHT_AT_22_PX),
        ))
    }
}
