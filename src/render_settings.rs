use std::collections::BTreeMap;

use getset::Getters;
use manycore_parser::RoutingAlgorithms;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Getters, PartialEq, Debug)]
#[getset(get = "pub")]
pub struct ColourSettings {
    bounds: [u64; 4],
    colours: [String; 4],
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum FieldConfiguration {
    Text(String),
    ColouredText(String, ColourSettings),
    Fill(ColourSettings),
}

#[derive(Serialize, Deserialize, Getters, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Configuration {
    core_config: BTreeMap<String, FieldConfiguration>,
    router_config: BTreeMap<String, FieldConfiguration>,
    routing_config: Option<RoutingAlgorithms>,
    sinks_sources: Option<bool>,
}

#[cfg(test)]
mod tests {
    use ::lazy_static::lazy_static;
    use manycore_parser::ManycoreSystem;

    use std::{
        collections::BTreeMap,
        fs::{self, read_to_string},
    };

    use crate::{ColourSettings, Configuration, FieldConfiguration, SVG};

    lazy_static! {
        static ref EXPECTED_CONFIGURATION: Configuration = Configuration {
            core_config: BTreeMap::from([
                (
                    "@id".to_string(),
                    FieldConfiguration::Text("ID".to_string()),
                ),
                (
                    "@coordinates".to_string(),
                    FieldConfiguration::Text("T".to_string()),
                ),
                (
                    "@age".to_string(),
                    FieldConfiguration::Fill(ColourSettings {
                        bounds: [30, 100, 200, 300],
                        colours: [
                            "#22c55e".to_string(),
                            "#eab308".to_string(),
                            "#f97316".to_string(),
                            "#dc2626".to_string(),
                        ],
                    }),
                ),
                (
                    "@temperature".to_string(),
                    FieldConfiguration::ColouredText(
                        "Temp".to_string(),
                        ColourSettings {
                            bounds: [30, 31, 50, 75],
                            colours: [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        },
                    ),
                ),
            ]),
            router_config: BTreeMap::from([
                (
                    "@age".to_string(),
                    FieldConfiguration::Fill(ColourSettings {
                        bounds: [30, 100, 200, 300],
                        colours: [
                            "#22c55e".to_string(),
                            "#eab308".to_string(),
                            "#f97316".to_string(),
                            "#dc2626".to_string(),
                        ],
                    }),
                ),
                (
                    "@temperature".to_string(),
                    FieldConfiguration::ColouredText(
                        "Temp".to_string(),
                        ColourSettings {
                            bounds: [30, 31, 50, 75],
                            colours: [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        },
                    ),
                ),
            ]),
            routing_config: Some(manycore_parser::RoutingAlgorithms::RowFirst),
            sinks_sources: None
        };
    }

    #[test]
    fn can_parse() {
        let conf_file =
            fs::File::open("tests/conf2.json").expect("Could not open \"tests/conf2.json\"");
        let configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf2.json\"");

        assert_eq!(configuration, *EXPECTED_CONFIGURATION)
    }

    #[test]
    fn can_generate_according_to_conf() {
        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let mut svg: SVG = (&manycore).into();
        let _ = svg
            .update_configurable_information(&mut manycore, &EXPECTED_CONFIGURATION)
            .expect("Could not generate SVG update.");

        let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG2.svg")
            .expect("Could not read input test file \"tests/SVG2.svg\"");

        // assert_eq!(res, expected)
    }

    #[test]
    fn can_serialise_configuration_update() {
        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let mut svg: SVG = (&manycore).into();
        let update = svg
            .update_configurable_information(&mut manycore, &EXPECTED_CONFIGURATION)
            .expect("Could not generate update based on configuration.");

        let expected_style = read_to_string("tests/style_update.xml")
            .expect("Could not open \"tests/style_update.xml\"");
        let expected_information = read_to_string("tests/information_update.xml")
            .expect("Could not open \"tests/information_update.xml\"");
        let expected_sinks_source = read_to_string("tests/sinks_sources_update.xml")
            .expect("Could not open \"tests/sinks_sources_update.xml\"");
        let expected_view_box = read_to_string("tests/view_box_update.txt")
            .expect("Could not open \"tests/view_box_update.txt\"");

        // assert_eq!(update.style, expected_style);
        // assert_eq!(update.information_group, expected_information);
        // assert_eq!(update.sinks_sources_group, expected_sinks_source);
        // assert_eq!(update.view_box, expected_view_box);
    }

    #[test]
    fn can_flip_coordinates() {
        let conf_file =
            fs::File::open("tests/conf3.json").expect("Could not open \"tests/conf3.json\"");
        let configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf3.json\"");

        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let mut svg: SVG = (&manycore).into();
        let _ = svg
            .update_configurable_information(&mut manycore, &configuration)
            .expect("Could not generate SVG update");

        let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG3.svg")
            .expect("Could not read input test file \"tests/SVG3.svg\"");

        // assert_eq!(res, expected)
    }

    // Routing needs to be fixed in manycore_parrser
    // #[test]
    // fn all_links_are_correct() {
    //     let conf_file =
    //         fs::File::open("tests/conf4.json").expect("Could not open \"tests/conf4.json\"");
    //     let configuration: Configuration =
    //         serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf4.json\"");

    //     let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
    //         .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

    //     let mut svg: SVG = (&manycore).into();
    //     let _ = svg
    //         .update_configurable_information(&mut manycore, &configuration)
    //         .expect("Could not generate SVG update");

    //     let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

    //     let expected = read_to_string("tests/SVG4.svg")
    //         .expect("Could not read input test file \"tests/SVG4.svg\"");

    //     // assert_eq!(res, expected)
    // }
}
