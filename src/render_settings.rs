use std::collections::{btree_map::Iter, BTreeMap, BTreeSet};

use getset::Getters;
use manycore_parser::{AttributeType, ManycoreSystem, RoutingAlgorithms};
use serde::{Deserialize, Serialize};

use crate::SVG;

#[derive(Serialize, Deserialize, Getters, PartialEq, Debug, PartialOrd, Eq, Ord)]
#[getset(get = "pub")]
pub struct ColourSettings {
    bounds: [u64; 4],
    colours: [String; 4],
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum CoordinatesOrientation {
    T,
    B,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum FieldConfiguration {
    Text(String),
    ColouredText(String, ColourSettings),
    Fill(ColourSettings),
    Coordinates(CoordinatesOrientation),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, PartialOrd, Eq, Ord)]
pub enum ChannelConfiguration {
    Load(LoadConfiguration, ColourSettings),
    Attribute(String, Option<ColourSettings>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, PartialOrd, Eq, Ord)]
pub enum LoadConfiguration {
    Percentage,
    Fraction,
}

#[derive(Serialize, Deserialize, Getters, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct Configuration {
    core_config: BTreeMap<String, FieldConfiguration>,
    router_config: BTreeMap<String, FieldConfiguration>,
    routing_config: Option<RoutingAlgorithms>,
    sinks_sources: Option<bool>,
    channel_config: BTreeMap<String, FieldConfiguration>,
}

#[derive(Serialize, PartialEq, Debug)]
pub struct ProcessedAttribute {
    _type: AttributeType,
    display: String,
}

impl ProcessedAttribute {
    fn format_display(key: &String) -> String {
        // Uppercase chars indices (true = uppercase, false = lowercase)
        let upper_i = key.chars().map(|c| c.is_uppercase()).collect::<Vec<bool>>();
        // Last iterable item
        let last = upper_i.len() - 1;

        // Previous split index
        let mut prev = 0usize;

        let mut ret = String::new();

        // Last is exclusive here because we always
        // want to be able to grab the current and next char descriptors.
        for i in 0..last {
            // Char at i descriptor
            let first = upper_i[i];
            // Following char
            let second = upper_i[i + 1];

            if first && !second && prev != i {
                // This condition is met for something like Ab.
                // Useful to catch multiple uppercase chars that form a
                // block and are then followed by another word.
                // e.g. helloCAMELCase -> hello camel case
                ret.push_str(&key[prev..=(i - 1)].to_lowercase());
                ret.push(' ');
                prev = i;
            } else if !first && second {
                // This condition is met for something like aB.
                // e.g. camelCase -> camel case
                ret.push_str(&key[prev..=i].to_lowercase());
                ret.push(' ');
                prev = i + 1;
            }
        }
        // Append remaining string, if any
        ret.push_str(&key[prev..].to_lowercase());

        // Trim any excess space
        let mut result = ret.trim_end().to_string();

        // Uppercase first char
        result.replace_range(0..1, &result[0..1].to_uppercase());

        result
    }

    fn new(key: &String, _type: AttributeType) -> Self {
        Self {
            _type,
            display: Self::format_display(key),
        }
    }
}

#[derive(Serialize, Getters, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SVGConfigurableAttributes {
    core: BTreeMap<String, ProcessedAttribute>,
    router: BTreeMap<String, ProcessedAttribute>,
    algorithms: Vec<RoutingAlgorithms>,
    observed_algorithm: Option<String>,
    channel: BTreeMap<String, ProcessedAttribute>,
}

impl SVG {
    fn populate_config_btreemap(
        map: &mut BTreeMap<String, ProcessedAttribute>,
        iter: Iter<String, AttributeType>,
    ) {
        for (key, attribute_type) in iter {
            map.insert(
                key.clone(),
                ProcessedAttribute::new(key, attribute_type.clone()),
            );
        }
    }
    pub fn derive_configurable_attributes(manycore: &ManycoreSystem) -> SVGConfigurableAttributes {
        let manycore_configurable_attributes = manycore.configurable_attributes();

        let mut config = SVGConfigurableAttributes {
            core: BTreeMap::new(),
            router: BTreeMap::new(),
            algorithms: manycore_configurable_attributes.algorithms().clone(),
            observed_algorithm: manycore_configurable_attributes
                .observed_algorithm()
                .clone(),
            channel: BTreeMap::new(),
        };

        SVG::populate_config_btreemap(
            &mut config.core,
            manycore_configurable_attributes.core().iter(),
        );
        SVG::populate_config_btreemap(
            &mut config.router,
            manycore_configurable_attributes.router().iter(),
        );
        SVG::populate_config_btreemap(
            &mut config.channel,
            manycore_configurable_attributes.channel().iter(),
        );

        config
    }
}

#[cfg(test)]
mod tests {
    use ::lazy_static::lazy_static;
    use manycore_parser::ManycoreSystem;

    use std::{collections::BTreeMap, fs::read_to_string};

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
            sinks_sources: None,
            channel_config: BTreeMap::new(),
        };
    }

    // #[test]
    // fn can_parse() {
    //     let conf_file =
    //         fs::File::open("tests/conf2.json").expect("Could not open \"tests/conf2.json\"");
    //     let configuration: Configuration =
    //         serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf2.json\"");

    //     assert_eq!(configuration, *EXPECTED_CONFIGURATION)
    // }

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
        println!("{res}")
    }

    // #[test]
    // fn can_serialise_configuration_update() {
    //     let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
    //         .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

    //     let mut svg: SVG = (&manycore).into();
    //     let update = svg
    //         .update_configurable_information(&mut manycore, &EXPECTED_CONFIGURATION)
    //         .expect("Could not generate update based on configuration.");

    //     let expected_style = read_to_string("tests/style_update.xml")
    //         .expect("Could not open \"tests/style_update.xml\"");
    //     let expected_information = read_to_string("tests/information_update.xml")
    //         .expect("Could not open \"tests/information_update.xml\"");
    //     let expected_sinks_source = read_to_string("tests/sinks_sources_update.xml")
    //         .expect("Could not open \"tests/sinks_sources_update.xml\"");
    //     let expected_view_box = read_to_string("tests/view_box_update.txt")
    //         .expect("Could not open \"tests/view_box_update.txt\"");

    //     // assert_eq!(update.style, expected_style);
    //     // assert_eq!(update.information_group, expected_information);
    //     // assert_eq!(update.sinks_sources_group, expected_sinks_source);
    //     // assert_eq!(update.view_box, expected_view_box);
    // }

    // #[test]
    // fn can_flip_coordinates() {
    //     let conf_file =
    //         fs::File::open("tests/conf3.json").expect("Could not open \"tests/conf3.json\"");
    //     let configuration: Configuration =
    //         serde_json::from_reader(conf_file).expect("Could not parse \"tests/conf3.json\"");

    //     let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
    //         .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

    //     let mut svg: SVG = (&manycore).into();
    //     let _ = svg
    //         .update_configurable_information(&mut manycore, &configuration)
    //         .expect("Could not generate SVG update");

    //     let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

    //     let expected = read_to_string("tests/SVG3.svg")
    //         .expect("Could not read input test file \"tests/SVG3.svg\"");

    //     // assert_eq!(res, expected)
    // }

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
