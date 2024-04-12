use manycore_parser::Directions;
use serde::Serialize;

use crate::{
    coordinate, sinks_sources_layer::SINKS_SOURCES_CONNECTION_LENGTH, style::EDGE_DATA_CLASS_NAME,
    FieldConfiguration, LoadConfiguration, RoutingConfiguration, CONNECTION_LENGTH, MARKER_HEIGHT,
    ROUTER_OFFSET,
};

use super::utils;

static HORIZONTAL_OFFSET_FROM_LINK: coordinate = 5;
static VERTICAL_OFFSET_FROM_LINK: coordinate = 1;
static OFFSET_FROM_FIRST: coordinate = 20;
static HALF_CONNECTION_LENGTH: coordinate = CONNECTION_LENGTH
    .saturating_add(MARKER_HEIGHT)
    .saturating_div(2);
// pub static CHAR_WIDTH_AT_16_PX: f32 = 9.3;
pub static CHAR_WIDTH_AT_22_PX: f32 = 13.3;
pub static CHAR_HEIGHT_AT_22_PX: coordinate = 30;
pub static CHAR_V_PADDING: coordinate = 3;
pub static CHAR_H_PADDING: f32 = CHAR_WIDTH_AT_22_PX * 2.0;

#[derive(Serialize)]
pub struct TextInformation {
    #[serde(rename = "@x")]
    x: coordinate,
    #[serde(rename = "@y")]
    y: coordinate,
    #[serde(rename = "@font-size")]
    font_size: String,
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
    pub fn new(
        x: coordinate,
        y: coordinate,
        font_size: Option<&str>,
        text_anchor: &'static str,
        dominant_baseline: &'static str,
        fill: Option<&String>,
        class: Option<&'static str>,
        value: String,
    ) -> Self {
        Self {
            x,
            y,
            font_size: match font_size {
                Some(fs) => fs.to_string(),
                None => "16px".to_string(),
            },
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

    fn common_link_primary(
        link_x: &coordinate,
        link_y: &coordinate,
        direction: &Directions,
        relevant_delta: coordinate,
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
                    None,
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
                    None,
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
                    None,
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
                    None,
                    "middle",
                    "text-after-edge",
                    fill,
                    class,
                    data,
                )
            }
        }
    }

    fn calculate_load_fill_and_percentage<'a>(
        load: &u16,
        bandwidth: &u16,
        routing_configuration: &'a RoutingConfiguration,
    ) -> (Option<u16>, Option<&'a String>) {
        if *bandwidth > 0 {
            let percentage = ((f32::from(*load) / f32::from(*bandwidth)) * 100.0).round() as u16;

            let fill_idx = utils::binary_search_left_insertion_point(
                routing_configuration.load_colours().bounds(),
                percentage.into(),
            );

            let fill = &routing_configuration.load_colours().colours()[fill_idx];
            return (Some(percentage), Some(fill));
        } else {
            return (
                None,
                Some(&routing_configuration.load_colours().colours()[3]),
            );
        }
    }

    fn calculate_load_data(
        load: &u16,
        bandwidth: &u16,
        percentage: Option<u16>,
        routing_configuration: &RoutingConfiguration,
    ) -> String {
        match routing_configuration.load_configuration() {
            LoadConfiguration::Percentage => match percentage {
                Some(value) => format!("{}: {}%", routing_configuration.display(), value),
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

    pub fn source_load(
        direction: &Directions,
        link_x: &coordinate,
        link_y: &coordinate,
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
        let data = TextInformation::calculate_load_data(
            load,
            bandwidth,
            percentage,
            routing_configuration,
        );

        TextInformation::common_link_primary(
            link_x,
            link_y,
            direction,
            relevant_delta,
            fill,
            Some(EDGE_DATA_CLASS_NAME),
            data,
        )
    }

    fn link_delta_and_class(
        edge: bool,
        direction: &Directions,
    ) -> (coordinate, Option<&'static str>) {
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

    pub fn link_load(
        direction: &Directions,
        link_x: &coordinate,
        link_y: &coordinate,
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
        let data = TextInformation::calculate_load_data(
            load,
            bandwidth,
            percentage,
            routing_configuration,
        );

        TextInformation::common_link_primary(
            link_x,
            link_y,
            direction,
            relevant_delta,
            fill,
            class,
            data,
        )
    }

    pub fn link_primary(
        direction: &Directions,
        link_x: &coordinate,
        link_y: &coordinate,
        data: &String,
        edge: bool,
        field_configuration: &FieldConfiguration,
    ) -> Self {
        let (relevant_delta, class) = TextInformation::link_delta_and_class(edge, direction);

        let (fill, display) = match field_configuration {
            FieldConfiguration::ColouredText(display_key, colour_config) => (
                utils::get_attribute_colour(colour_config.bounds(), colour_config.colours(), data),
                format!("{}: {}", display_key, data),
            ),
            FieldConfiguration::Text(display_key) => (None, format!("{}: {}", display_key, data)),
            _ => (None, "".into()), // Unsupported
        };

        TextInformation::common_link_primary(
            link_x,
            link_y,
            direction,
            relevant_delta,
            fill,
            class,
            display,
        )
    }

    pub fn link_secondary(
        direction: &Directions,
        link_x: &coordinate,
        link_y: &coordinate,
        data: &String,
        field_configuration: &FieldConfiguration,
    ) -> Self {
        // This function is called only for non edge links
        let (relevant_delta, class) = TextInformation::link_delta_and_class(false, direction);

        let (fill, display) = match field_configuration {
            FieldConfiguration::ColouredText(display_key, colour_config) => (
                utils::get_attribute_colour(colour_config.bounds(), colour_config.colours(), data),
                format!("{}: {}", display_key, data),
            ),
            FieldConfiguration::Text(display_key) => (None, format!("{}: {}", display_key, data)),
            _ => (None, "".into()), // Unsupported
        };

        match direction {
            Directions::North => {
                let delta_y = relevant_delta;

                TextInformation::new(
                    link_x.saturating_add(HORIZONTAL_OFFSET_FROM_LINK),
                    link_y
                        .saturating_sub(delta_y)
                        .saturating_add(OFFSET_FROM_FIRST),
                    None,
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
                    None,
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
                    None,
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
                    None,
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
