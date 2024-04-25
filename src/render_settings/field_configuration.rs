use getset::Getters;
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

impl ColourSettings {
    #[cfg(test)]
    /// Generates a new [`ColourSettings`] from the given parameters.
    pub(crate) fn new(bounds: [u64; 4], colours: [String; 4]) -> Self {
        Self { bounds, colours }
    }
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
    #[serde(flatten)]
    load_colours: ColourSettings,
    display: String,
}

impl RoutingConfiguration {
    #[cfg(test)]
    pub(crate) fn new(
        algorithm: RoutingAlgorithms,
        load_configuration: LoadConfiguration,
        load_colours: ColourSettings,
        display: String,
    ) -> Self {
        Self {
            algorithm,
            load_configuration,
            load_colours,
            display,
        }
    }
}

/// Channel load configuration.
#[derive(Serialize, Deserialize, PartialEq, Debug, PartialOrd, Eq, Ord)]
pub enum LoadConfiguration {
    /// Display loads as percentage of bandwidth, e.g. 5%.
    Percentage,
    /// Display loads as fraction of bandwith, e.g. 20/400.
    Fraction,
}

/// Possible ways a field can be configured.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum FieldConfiguration {
    /// Text only.
    Text {
        display: String,
        colour: Option<String>,
    },
    /// Coloured Text according to provided [`ColourSettings`].
    ColouredText {
        display: String,
        #[serde(rename = "colourSettings", flatten)]
        colour_settings: ColourSettings,
    },
    /// Fill colour of associated element, according to provided [`ColourSettings`].
    Fill {
        #[serde(rename = "colourSettings", flatten)]
        colour_settings: ColourSettings,
    },
    /// This variant can be used to configure coordinates display only.
    Coordinates { orientation: CoordinatesOrientation },
    /// This variant can be used to configure routing only.
    Routing {
        #[serde(flatten)]
        configuration: RoutingConfiguration,
    },
    /// This variant can be used to configure boolean properties, e.g. displaying border routers.
    Boolean { value: bool },
}

impl FieldConfiguration {
    pub(crate) fn type_str(&self) -> &'static str {
        match self {
            FieldConfiguration::Boolean { .. } => "Boolean",
            FieldConfiguration::ColouredText { .. } => "ColouredText",
            FieldConfiguration::Coordinates { .. } => "Coordinates",
            FieldConfiguration::Fill { .. } => "Fill",
            FieldConfiguration::Routing { .. } => "Routing",
            FieldConfiguration::Text { .. } => "Text",
        }
    }
}
