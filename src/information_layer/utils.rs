use std::{collections::BTreeMap, fmt::Display};

use manycore_parser::{Directions, WithID, WithXMLAttributes, COORDINATES_KEY, ID_KEY};

use super::{ProcessingInformation, TextInformation, OFFSET_FROM_BORDER, TEXT_GROUP_FILTER};
use crate::{
    ConnectionType, ConnectionsParentGroup, DirectionType, FieldConfiguration, SVGError,
    SVGErrorKind,
};

pub static FONT_SIZE_WITH_OFFSET: u16 = 18;

pub fn binary_search_left_insertion_point(bounds: &[u64; 4], val: u64) -> usize {
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

pub fn generate_with_id<K: Display, T: WithID<K> + WithXMLAttributes>(
    mut base_x: u16,
    mut base_y: u16,
    configuration: &BTreeMap<String, FieldConfiguration>,
    target: &T,
    group: &mut ProcessingInformation,
    text_anchor: &'static str,
    css: &mut String,
) {
    base_x += OFFSET_FROM_BORDER;
    base_y += OFFSET_FROM_BORDER;

    // Id value is outside of attributes map
    if let Some(configuration) = configuration.get(ID_KEY) {
        match configuration {
            FieldConfiguration::Text(title) => {
                group.information.push(TextInformation::new(
                    base_x,
                    base_y,
                    text_anchor,
                    "text-before-edge",
                    None,
                    None,
                    format!("{}: {}", title, target.id()),
                ));
                base_y += FONT_SIZE_WITH_OFFSET;
            }
            _ => {}
        }
    }

    if let Some(map) = target.other_attributes() {
        for k in configuration.keys() {
            match k.as_str() {
                id_coordinates if id_coordinates == ID_KEY || id_coordinates == COORDINATES_KEY => {
                    // These have been handled
                }
                valid_key => {
                    if let (Some(field_configuration), Some(value)) =
                        (configuration.get(valid_key), map.get(k))
                    {
                        match field_configuration {
                            FieldConfiguration::Text(title) => {
                                group.information.push(TextInformation::new(
                                    base_x,
                                    base_y,
                                    text_anchor,
                                    "text-before-edge",
                                    None,
                                    None,
                                    format!("{}: {}", title, value),
                                ));
                                base_y += FONT_SIZE_WITH_OFFSET;
                            }
                            FieldConfiguration::Fill(colour_config) => {
                                let bounds = colour_config.bounds();
                                if let Ok(value_num) = value.parse::<u64>() {
                                    let fill_idx =
                                        binary_search_left_insertion_point(bounds, value_num);

                                    css.push_str(
                                        format!(
                                            "\n#{}{} {{fill: {};}}",
                                            target.variant(),
                                            target.id(),
                                            colour_config.colours()[fill_idx]
                                        )
                                        .as_str(),
                                    );

                                    group.filter = Some(TEXT_GROUP_FILTER);
                                }
                            }
                            FieldConfiguration::ColouredText(title, colour_config) => {
                                let fill = get_attribute_colour(
                                    colour_config.bounds(),
                                    colour_config.colours(),
                                    value,
                                );

                                group.information.push(TextInformation::new(
                                    base_x,
                                    base_y,
                                    text_anchor,
                                    "text-before-edge",
                                    fill,
                                    None,
                                    format!("{}: {}", title, value),
                                ));
                                base_y += FONT_SIZE_WITH_OFFSET;
                            }
                            _ => {
                                // Remaining variants are handled elsewhere
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn get_attribute_colour<'a>(
    bounds: &'a [u64; 4],
    colours: &'a [String; 4],
    attribute_value: &'a String,
) -> Option<&'a String> {
    let mut fill: Option<&String> = None;

    if let Ok(value_num) = attribute_value.parse::<u64>() {
        let fill_idx = binary_search_left_insertion_point(bounds, value_num);
        fill = Some(&colours[fill_idx]);
    }

    fill
}

pub fn get_connection_type<'a>(
    connections_group: &'a ConnectionsParentGroup,
    direction_type: &'a DirectionType,
    core_id: &'a u8,
) -> Result<&'a ConnectionType, SVGError> {
    connections_group
        .core_connections_map()
        .get(core_id)
        .ok_or(SVGError::new(SVGErrorKind::ConnectionError(format!(
            "Could not get connections for Core {}",
            core_id
        ))))?
        .get(direction_type)
        .ok_or(SVGError::new(SVGErrorKind::ConnectionError(format!(
            "Could not get connection {} for Core {}",
            direction_type, core_id
        ))))
}

pub fn missing_connection(idx: &usize) -> SVGError {
    SVGError::new(SVGErrorKind::ConnectionError(format!(
        "Could not grab SVG connection path for Core {}",
        idx
    )))
}

pub fn missing_source(task_id: &u16) -> SVGError {
    SVGError::new(SVGErrorKind::ManycoreMismatch(format!(
        "Could not retrieve Source for Task {}",
        task_id
    )))
}

pub fn missing_channel(core_id: &u8, direction: &Directions) -> SVGError {
    SVGError::new(SVGErrorKind::ManycoreMismatch(format!(
        "Could not retrieve {} channel for Core {}",
        direction, core_id
    )))
}
