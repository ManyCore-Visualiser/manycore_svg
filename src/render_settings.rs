use std::{
    collections::BTreeMap,
    ops::{Div, Sub},
};

use getset::{Getters, MutGetters};
use manycore_parser::ElementIDT;
use serde::{Deserialize, Serialize};

use crate::{
    CoordinateT, FontSizeT, CHAR_V_PADDING, DEFAULT_ATTRIBUTE_FONT_SIZE, DEFAULT_TASK_FONT_SIZE,
};

mod configurable_base_configuration;
mod field_configuration;

pub use configurable_base_configuration::*;
pub use field_configuration::*;

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
    core_fills: BTreeMap<ElementIDT, String>,
    router_fills: BTreeMap<ElementIDT, String>,
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
    attribute_font_size_coordinate: CoordinateT,
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
            attribute_font_size_coordinate: base_configuration.attribute_font_size.round()
                as CoordinateT,
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
                    FieldConfiguration::Text {
                        display: "ID".to_string(),
                        colour: None,
                    },
                ),
                (
                    "@coordinates".to_string(),
                    FieldConfiguration::Coordinates {
                        orientation: CoordinatesOrientation::T,
                    },
                ),
                (
                    "@age".to_string(),
                    FieldConfiguration::Fill {
                        colour_settings: ColourSettings::new(
                            [30, 100, 200, 300],
                            [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        ),
                    },
                ),
                (
                    "@temperature".to_string(),
                    FieldConfiguration::ColouredText {
                        display: "Temp".to_string(),
                        colour_settings: ColourSettings::new(
                            [30, 31, 50, 75],
                            [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        ),
                    },
                ),
            ]),
            router_config: BTreeMap::from([
                (
                    "@age".to_string(),
                    FieldConfiguration::Fill {
                        colour_settings: ColourSettings::new(
                            [30, 100, 200, 300],
                            [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        ),
                    },
                ),
                (
                    "@temperature".to_string(),
                    FieldConfiguration::ColouredText {
                        display: "Temp".to_string(),
                        colour_settings: ColourSettings::new(
                            [30, 31, 50, 75],
                            [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        ),
                    },
                ),
            ]),
            channel_config: BTreeMap::from([
                (
                    "@age".to_string(),
                    FieldConfiguration::ColouredText {
                        display: "Age".to_string(),
                        colour_settings: ColourSettings::new(
                            [30, 100, 200, 300],
                            [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        ),
                    },
                ),
                (
                    BORDER_ROUTERS_KEY.to_string(),
                    FieldConfiguration::Boolean { value: true },
                ),
                (
                    ROUTING_KEY.to_string(),
                    FieldConfiguration::Routing {
                        configuration: RoutingConfiguration::new(
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
                        ),
                    },
                ),
            ]),
            core_fills: BTreeMap::new(),
            router_fills: BTreeMap::new(),
        };

        let conf_file = fs::File::open("tests/conf_test.json")
            .expect("Could not open \"tests/conf_test.json\"");
        let configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf_test.json\"");

        #[cfg(feature = "print")]
        println!(
            "Conf: {}\n\n",
            serde_json::to_string(&expected_configuration).unwrap()
        );
        #[cfg(not(feature = "print"))]
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

        let res = String::try_from(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG2.svg")
            .expect("Could not read input test file \"tests/SVG2.svg\"");

        #[cfg(feature = "print")]
        fs::write("tests-out/SVG2.svg", res);
        #[cfg(not(feature = "print"))]
        assert_eq!(res, expected);
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

        #[cfg(feature = "print")]
        {
            fs::write("tests-out/style_update.css", update.style);
            fs::write("tests-out/information_update.xml", update.information_group);
            fs::write("tests-out/view_box_update.txt", update.view_box);
        }
        #[cfg(not(feature = "print"))]
        {
            assert_eq!(update.style, expected_style);
            assert_eq!(update.information_group, expected_information);
            assert_eq!(update.view_box, expected_view_box);
        }
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

        let res = String::try_from(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG3.svg")
            .expect("Could not read input test file \"tests/SVG3.svg\"");

        #[cfg(feature = "print")]
        fs::write("tests-out/SVG3.svg", res);
        #[cfg(not(feature = "print"))]
        assert_eq!(res, expected);
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

        let res = String::try_from(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG4.svg")
            .expect("Could not read input test file \"tests/SVG4.svg\"");

        #[cfg(feature = "print")]
        fs::write("tests-out/SVG4.svg", res);
        #[cfg(not(feature = "print"))]
        assert_eq!(res, expected);
    }

    #[test]
    fn handles_base_configuration() {
        let conf_file =
            fs::File::open("tests/conf3.json").expect("Could not open \"tests/conf3.json\"");
        let mut configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf3.json\"");

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

        let res = String::try_from(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG5.svg")
            .expect("Could not read input test file \"tests/SVG5.svg\"");

        #[cfg(feature = "print")]
        fs::write("tests-out/SVG5.svg", res);
        #[cfg(not(feature = "print"))]
        assert_eq!(res, expected);
    }

    #[test]
    fn can_colour_text() {
        let conf_file: fs::File =
            fs::File::open("tests/conf6.json").expect("Could not open \"tests/conf6.json\"");
        let mut configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf6.json\"");

        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let mut svg: SVG = SVG::try_from(&manycore).expect("Could not convert Manycore to SVG.");

        let _ = svg
            .update_configurable_information(&mut manycore, &mut configuration, &BASE_CONFIG)
            .expect("Could not generate SVG update");

        let res = String::try_from(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG6.svg")
            .expect("Could not read input test file \"tests/SVG6.svg\"");

        #[cfg(feature = "print")]
        fs::write("tests-out/SVG6.svg", res);
        #[cfg(not(feature = "print"))]
        assert_eq!(res, expected);
    }

    #[test]
    fn can_override_fill() {
        let conf_file: fs::File =
            fs::File::open("tests/conf7.json").expect("Could not open \"tests/conf7.json\"");
        let mut configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf7.json\"");

        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let mut svg: SVG = SVG::try_from(&manycore).expect("Could not convert Manycore to SVG.");

        let _ = svg
            .update_configurable_information(&mut manycore, &mut configuration, &BASE_CONFIG)
            .expect("Could not generate SVG update");

        let res = String::try_from(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG7.svg")
            .expect("Could not read input test file \"tests/SVG6.svg\"");

        #[cfg(feature = "print")]
        fs::write("tests-out/SVG7.svg", res);
        #[cfg(not(feature = "print"))]
        assert_eq!(res, expected);
    }
}
