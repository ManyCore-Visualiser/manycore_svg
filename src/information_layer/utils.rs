use std::collections::BTreeMap;

use manycore_parser::WithXMLAttributes;

use super::{
    InformationLayer, ProcessingInformation, TextInformation, OFFSET_FROM_BORDER, TEXT_GROUP_FILTER,
};
use crate::{FieldConfiguration, FONT_SIZE_WITH_OFFSET};

pub fn generate<T: WithXMLAttributes>(
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

    if let Some(configuration) = configuration.get("@id") {
        match configuration {
            FieldConfiguration::Text(title) => {
                group.information.push(TextInformation::new(
                    base_x,
                    base_y,
                    text_anchor,
                    "text-before-edge",
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
                "@id" | "@coordinates" => {}
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
                                    format!("{}: {}", title, value),
                                ));
                                base_y += FONT_SIZE_WITH_OFFSET;
                            }
                            FieldConfiguration::Fill(colour_config) => {
                                let bounds = colour_config.bounds();
                                if let Ok(value_num) = value.parse::<u64>() {
                                    let fill_idx =
                                        InformationLayer::binary_search_left_insertion_point(
                                            bounds, value_num,
                                        );

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
                                let bounds = colour_config.bounds();
                                let mut fill: Option<&String> = None;

                                if let Ok(value_num) = value.parse::<u64>() {
                                    let fill_idx =
                                        InformationLayer::binary_search_left_insertion_point(
                                            bounds, value_num,
                                        );
                                    fill = Some(&colour_config.colours()[fill_idx]);
                                }

                                group.information.push(TextInformation::new(
                                    base_x,
                                    base_y,
                                    text_anchor,
                                    "text-before-edge",
                                    fill,
                                    format!("{}: {}", title, value),
                                ));
                                base_y += FONT_SIZE_WITH_OFFSET;
                            }
                        }
                    }
                }
            }
        }
    }
}
