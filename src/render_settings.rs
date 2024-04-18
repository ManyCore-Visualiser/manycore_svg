use std::{
    collections::BTreeMap,
    ops::{Div, Sub},
};

use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};

use crate::{
    CoordinateT, FontSizeT, CHAR_V_PADDING, DEFAULT_ATTRIBUTE_FONT_SIZE, DEFAULT_TASK_FONT_SIZE,
};

mod field_configuration;
mod configurable_base_configuration;

pub use field_configuration::*;
pub use configurable_base_configuration::*;

#[cfg(doc)]
use manycore_parser::{Channel, Core, Router};

/// Object representation of user-defined configuration.
/// * `core_config`: A [`BTreeMap`] with [`String`] attribute key and [`FieldConfiguration`] value. Controls what [`Core`] information to display and how.
/// * `router_config`: A [`BTreeMap`] with [`String`] attribute key and [`FieldConfiguration`] value. Controls what [`Router`] information to display and how.
/// * `channel_config`: A [`BTreeMap`] with [`String`] attribute key and [`FieldConfiguration`] value. Controls what [`Channel`] information to display and how.
#[derive(Serialize, Deserialize, Getters, MutGetters, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub", get_mut = "pub")]
pub struct Configuration {
    core_config: BTreeMap<String, FieldConfiguration>,
    router_config: BTreeMap<String, FieldConfiguration>,
    channel_config: BTreeMap<String, FieldConfiguration>,
}

/// Object representation of user-defined base configuration.
/// This configuration contains fundamental details of the SVG structure that would require
/// a full re-generation upon change.
#[derive(Serialize, Deserialize, Getters, PartialEq, Debug, Clone, Copy)]
#[getset(get = "pub")]
pub struct BaseConfiguration {
    attribute_font_size: FontSizeT,
    task_font_size: FontSizeT,
}

impl BaseConfiguration {
    #[cfg(test)]
    pub(crate) fn new(attribute_font_size: FontSizeT, task_font_size: FontSizeT) -> Self {
        Self {
            attribute_font_size,
            task_font_size,
        }
    }

    pub(crate) const fn default() -> Self {
        Self {
            attribute_font_size: DEFAULT_ATTRIBUTE_FONT_SIZE,
            task_font_size: DEFAULT_TASK_FONT_SIZE,
        }
    }
}

/// This struct is used to hold values derived from the user provided [`BaseConfiguration`]
/// to avoid repeated calculations.
#[derive(Getters)]
#[getset(get = "pub")]
pub(crate) struct ProcessedBaseConfiguration {
    attribute_font_size: FontSizeT,
    task_font_size: FontSizeT,
    task_rect_height: CoordinateT,
    task_rect_half_height: CoordinateT,
    task_rect_centre_offset: CoordinateT,
    task_rect_bottom_padding: CoordinateT,
}

impl From<&BaseConfiguration> for ProcessedBaseConfiguration {
    fn from(base_configuration: &BaseConfiguration) -> Self {
        let task_rect_height =
            (base_configuration.task_font_size.round() as CoordinateT) + CHAR_V_PADDING;
        let task_rect_centre_offset = task_rect_height.div(5);

        Self {
            attribute_font_size: base_configuration.attribute_font_size,
            task_font_size: base_configuration.task_font_size,
            task_rect_height,
            task_rect_half_height: task_rect_height.div(2),
            task_rect_centre_offset,
            task_rect_bottom_padding: task_rect_height.sub(task_rect_centre_offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use manycore_parser::{ManycoreSystem, RoutingAlgorithms, BORDER_ROUTERS_KEY, ROUTING_KEY};

    use std::{
        collections::BTreeMap,
        fs::{self, read_to_string},
    };

    use crate::{
        BaseConfiguration, ColourSettings, Configuration, CoordinatesOrientation,
        FieldConfiguration, LoadConfiguration, RoutingConfiguration, DEFAULT_ATTRIBUTE_FONT_SIZE,
        DEFAULT_TASK_FONT_SIZE, SVG,
    };

    static BASE_CONFIG: BaseConfiguration = BaseConfiguration::default();

    #[test]
    fn can_parse_configuration() {
        let expected_configuration = Configuration {
            core_config: BTreeMap::from([
                (
                    "@id".to_string(),
                    FieldConfiguration::Text("ID".to_string()),
                ),
                (
                    "@coordinates".to_string(),
                    FieldConfiguration::Coordinates(CoordinatesOrientation::T),
                ),
                (
                    "@age".to_string(),
                    FieldConfiguration::Fill(ColourSettings::new(
                        [30, 100, 200, 300],
                        [
                            "#22c55e".to_string(),
                            "#eab308".to_string(),
                            "#f97316".to_string(),
                            "#dc2626".to_string(),
                        ],
                    )),
                ),
                (
                    "@temperature".to_string(),
                    FieldConfiguration::ColouredText(
                        "Temp".to_string(),
                        ColourSettings::new(
                            [30, 31, 50, 75],
                            [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        ),
                    ),
                ),
            ]),
            router_config: BTreeMap::from([
                (
                    "@age".to_string(),
                    FieldConfiguration::Fill(ColourSettings::new(
                        [30, 100, 200, 300],
                        [
                            "#22c55e".to_string(),
                            "#eab308".to_string(),
                            "#f97316".to_string(),
                            "#dc2626".to_string(),
                        ],
                    )),
                ),
                (
                    "@temperature".to_string(),
                    FieldConfiguration::ColouredText(
                        "Temp".to_string(),
                        ColourSettings::new(
                            [30, 31, 50, 75],
                            [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        ),
                    ),
                ),
            ]),
            channel_config: BTreeMap::from([
                (
                    "@age".to_string(),
                    FieldConfiguration::ColouredText(
                        "Age".to_string(),
                        ColourSettings::new(
                            [30, 100, 200, 300],
                            [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        ),
                    ),
                ),
                (
                    BORDER_ROUTERS_KEY.to_string(),
                    FieldConfiguration::Boolean(true),
                ),
                (
                    ROUTING_KEY.to_string(),
                    FieldConfiguration::Routing(RoutingConfiguration::new(
                        RoutingAlgorithms::RowFirst,
                        LoadConfiguration::Percentage,
                        ColourSettings::new(
                            [20, 50, 70, 90],
                            [
                                "#1a5fb4".to_string(),
                                "#26a269".to_string(),
                                "#c64600".to_string(),
                                "#a51d2d".to_string(),
                            ],
                        ),
                        String::from("L"),
                    )),
                ),
            ]),
        };

        let conf_file = fs::File::open("tests/conf_test.json")
            .expect("Could not open \"tests/conf_test.json\"");
        let configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf_test.json\"");

        assert_eq!(configuration, expected_configuration)
    }

    #[test]
    fn can_generate_according_to_conf() {
        let conf_file =
            fs::File::open("tests/conf2.json").expect("Could not open \"tests/conf2.json\"");
        let mut configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf2.json\"");

        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let mut svg: SVG = (&manycore)
            .try_into()
            .expect("Could not convert Manycorer to SVG.");
        let _ = svg
            .update_configurable_information(
                &mut manycore,
                &mut configuration,
                &BaseConfiguration::default(),
            )
            .expect("Could not generate SVG update.");

        let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG2.svg")
            .expect("Could not read input test file \"tests/SVG2.svg\"");

        assert_eq!(res, expected)
        // println!("SVG2: {res}\n\n")
    }

    #[test]
    fn can_serialise_configuration_update() {
        let conf_file =
            fs::File::open("tests/conf3.json").expect("Could not open \"tests/conf3.json\"");
        let mut configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf3.json\"");

        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let mut svg: SVG = (&manycore)
            .try_into()
            .expect("Could not convert Manycorer to SVG.");
        let update = svg
            .update_configurable_information(&mut manycore, &mut configuration, &BASE_CONFIG)
            .expect("Could not generate update based on configuration.");

        let expected_style = read_to_string("tests/style_update.css")
            .expect("Could not open \"tests/style_update.css\"");
        let expected_information = read_to_string("tests/information_update.xml")
            .expect("Could not open \"tests/information_update.xml\"");
        let expected_view_box = read_to_string("tests/view_box_update.txt")
            .expect("Could not open \"tests/view_box_update.txt\"");

        assert_eq!(update.style, expected_style);
        assert_eq!(update.information_group, expected_information);
        assert_eq!(update.view_box, expected_view_box);
        // println!("Update info: {}\n\n", update.information_group);
        // println!("Update viewBox: {}\n\n", update.view_box);
    }

    #[test]
    fn can_flip_coordinates() {
        let conf_file =
            fs::File::open("tests/conf3.json").expect("Could not open \"tests/conf3.json\"");
        let mut configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf3.json\"");

        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let mut svg: SVG = (&manycore)
            .try_into()
            .expect("Could not convert Manycorer to SVG.");
        let _ = svg
            .update_configurable_information(&mut manycore, &mut configuration, &BASE_CONFIG)
            .expect("Could not generate SVG update");

        let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG3.svg")
            .expect("Could not read input test file \"tests/SVG3.svg\"");

        assert_eq!(res, expected)
        // println!("SVG3: {res}\n\n")
    }

    #[test]
    fn all_links_are_correct() {
        let conf_file =
            fs::File::open("tests/conf4.json").expect("Could not open \"tests/conf4.json\"");
        let mut configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf4.json\"");

        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let mut svg: SVG = (&manycore)
            .try_into()
            .expect("Could not convert Manycorer to SVG.");

        let _ = svg
            .update_configurable_information(&mut manycore, &mut configuration, &BASE_CONFIG)
            .expect("Could not generate SVG update");

        let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG4.svg")
            .expect("Could not read input test file \"tests/SVG4.svg\"");

        assert_eq!(res, expected)
        // println!("SVG4: {res}\n\n")
    }

    #[test]
    fn handles_base_configuration() {
        let conf_file =
            fs::File::open("tests/conf2.json").expect("Could not open \"tests/conf2.json\"");
        let mut configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf2.json\"");

        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let base_configuration = BaseConfiguration::new(
            DEFAULT_ATTRIBUTE_FONT_SIZE * 2.0,
            DEFAULT_TASK_FONT_SIZE * 2.0,
        );

        let mut svg: SVG = SVG::try_from(&manycore).expect("Could not convert Manycore to SVG.");

        let _ = svg
            .update_configurable_information(&mut manycore, &mut configuration, &base_configuration)
            .expect("Could not generate SVG update");

        let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG5.svg")
            .expect("Could not read input test file \"tests/SVG5.svg\"");

        assert_eq!(res, expected)
        // println!("SVG5: {res}\n\n")
    }
}
