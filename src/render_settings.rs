use std::collections::HashMap;

use getset::Getters;
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
    core_config: HashMap<String, FieldConfiguration>,
    router_config: HashMap<String, FieldConfiguration>,
}

#[cfg(test)]
mod tests {
    use ::lazy_static::lazy_static;
    use manycore_parser::ManycoreSystem;

    use std::{collections::HashMap, fs::{self, read_to_string}};

    use crate::{ColourSettings, Configuration, FieldConfiguration, SVG};

    lazy_static! {
        static ref EXPECTED_CONFIGURATION: Configuration = Configuration {
            core_config: HashMap::from([
                (
                    "@id".to_string(),
                    FieldConfiguration::Text("ID".to_string()),
                ),
                (
                    "@coordinates".to_string(),
                    FieldConfiguration::Text("".to_string()),
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
            router_config: HashMap::from([
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
        };
    }

    #[test]
    fn can_parse() {
        let conf_file =
            fs::File::open("tests/conf2.json").expect("Could not open \"tests/conf1.json\"");
        let configuration: Configuration =
            serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf1.json\"");
        
        assert_eq!(configuration, *EXPECTED_CONFIGURATION)
    }

    #[test]
    fn can_generate_according_to_conf() {
        let manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let svg = SVG::from_manycore_with_configuration(&manycore, &EXPECTED_CONFIGURATION);

        let res =
            quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");
        
        let expected = read_to_string("tests/SVG2.svg")
            .expect("Could not read input test file \"tests/SVG1.svg\"");

        assert_eq!(res, expected)
    }
}
