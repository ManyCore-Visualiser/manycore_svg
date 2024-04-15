use std::collections::BTreeMap;

use getset::{Getters, MutGetters};
use manycore_parser::RoutingAlgorithms;
use serde::{Deserialize, Serialize};

/// Configuration colour settings
/// * `bounds`: Numerical boundaries. Used to determine colour.
/// * `colours`: List of colours (hexadecimal) corresponding to each boundary.
///
/// Example, given:
/// ```ignore
/// let colour_settings = ColourSettings {
///     bounds: [10, 20, 30, 40],
///     colours: ["#22c55e", "#eab308", "#f97316", "#dc2626"],
/// };
/// ```
/// **Warning:** Above is pseudo-code, all those fields are private and `colours` expects [`String`] not [`str`].
///
/// We would have:
///
/// | Attribute value | Colour    |
/// |-----------------|-----------|
/// | `9`             | `#22c55e` |
/// | `11`            | `#eab308` |
/// | `35`            | `#f97316` |
/// | `50`            | `#dc2626` |
#[derive(Serialize, Deserialize, Getters, PartialEq, Debug, PartialOrd, Eq, Ord)]
#[getset(get = "pub")]
pub struct ColourSettings {
    bounds: [u64; 4],
    colours: [String; 4],
}

/// Configuration coordinates orientation settins.
/// * [`T`][`CoordinatesOrientation::T`]: Top to bottom
/// * [`B`][`CoordinatesOrientation::B`]: Bottom to top
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum CoordinatesOrientation {
    T,
    B,
}

/// Object representation of requested routing configuration.
/// * `algorithm`: [`RoutingAlgorithms`]
/// * `load_configuration`: [`LoadConfiguration`]
/// * `load_colours`: [`ColourSettings`]
/// * `display`: [`String`], the display key of channel loads.
#[derive(Serialize, Deserialize, Getters, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
#[getset(get = "pub")]
pub struct RoutingConfiguration {
    algorithm: RoutingAlgorithms,
    load_configuration: LoadConfiguration,
    load_colours: ColourSettings,
    display: String,
}

/// Possible ways a field can be configured.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum FieldConfiguration {
    /// Text only.
    Text(String),
    /// Coloured Text according to provided [`ColourSettings`].
    ColouredText(String, ColourSettings),
    /// Fill colour of associated element, according to provided [`ColourSettings`].
    Fill(ColourSettings),
    /// This variant can be used to configure coordinates display only.
    Coordinates(CoordinatesOrientation),
    /// This variant can be used to configure routing only.
    Routing(RoutingConfiguration),
    /// This variant can be used to configure boolean properties, e.g. displaying border routers.
    Boolean(bool),
}

/// Channel load configuration.
#[derive(Serialize, Deserialize, PartialEq, Debug, PartialOrd, Eq, Ord)]
pub enum LoadConfiguration {
    /// Display loads as percentage of bandwidth, e.g. 5%.
    Percentage,
    /// Display loads as frraction of bandwith, e.g. 20/400.
    Fraction,
}

#[cfg(doc)]
use manycore_parser::{Channel, Core, Router};

/// Object representation of user-defined configurration.
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

#[cfg(test)]
mod tests {
    use manycore_parser::{ManycoreSystem, RoutingAlgorithms, BORDER_ROUTERS_KEY, ROUTING_KEY};

    use std::{
        collections::BTreeMap,
        fs::{self, read_to_string},
    };

    use crate::{
        ColourSettings, Configuration, CoordinatesOrientation, FieldConfiguration,
        LoadConfiguration, RoutingConfiguration, SVG,
    };

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
            channel_config: BTreeMap::from([
                (
                    "@age".to_string(),
                    FieldConfiguration::ColouredText(
                        "Age".to_string(),
                        ColourSettings {
                            bounds: [30, 100, 200, 300],
                            colours: [
                                "#22c55e".to_string(),
                                "#eab308".to_string(),
                                "#f97316".to_string(),
                                "#dc2626".to_string(),
                            ],
                        },
                    ),
                ),
                (
                    BORDER_ROUTERS_KEY.to_string(),
                    FieldConfiguration::Boolean(true),
                ),
                (
                    ROUTING_KEY.to_string(),
                    FieldConfiguration::Routing(RoutingConfiguration {
                        algorithm: RoutingAlgorithms::RowFirst,
                        load_configuration: LoadConfiguration::Percentage,
                        load_colours: ColourSettings {
                            bounds: [20, 50, 70, 90],
                            colours: [
                                "#1a5fb4".to_string(),
                                "#26a269".to_string(),
                                "#c64600".to_string(),
                                "#a51d2d".to_string(),
                            ],
                        },
                        display: String::from("L"),
                    }),
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
            .update_configurable_information(&mut manycore, &mut configuration)
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
            .update_configurable_information(&mut manycore, &mut configuration)
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
            .update_configurable_information(&mut manycore, &mut configuration)
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
            .update_configurable_information(&mut manycore, &mut configuration)
            .expect("Could not generate SVG update");

        let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG4.svg")
            .expect("Could not read input test file \"tests/SVG4.svg\"");

        assert_eq!(res, expected)
        // println!("SVG4: {res}\n\n")
    }
}
