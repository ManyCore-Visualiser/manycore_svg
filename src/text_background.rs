use serde::Serialize;

use crate::style::DEFAULT_FILL;

pub(crate) const TEXT_BACKGROUND_ID: &'static str = "textBackground";

/// Object representation of SVG `<feComposite>`.
#[derive(Serialize)]
struct Composite {
    #[serde(rename = "@in")]
    input: &'static str,
    #[serde(rename = "@operator")]
    operator: &'static str,
}

impl Default for Composite {
    fn default() -> Self {
        Self {
            input: "SourceGraphic",
            operator: "or",
        }
    }
}

/// Object representation of SVG `<feFlood>`.
#[derive(Serialize)]
struct Flood {
    #[serde(rename = "@flood-color")]
    flood_color: &'static str,
}

impl Default for Flood {
    fn default() -> Self {
        Self {
            flood_color: DEFAULT_FILL,
        }
    }
}

/// Object representation of SVG `<filter>` for text background.
#[derive(Serialize)]
pub(crate) struct TextBackground {
    #[serde(rename = "@x")]
    x: u8,
    #[serde(rename = "@y")]
    y: u8,
    #[serde(rename = "@width")]
    width: u8,
    #[serde(rename = "@height")]
    height: u8,
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "feFlood")]
    flood: Flood,
    #[serde(rename = "feComposite")]
    composite: Composite,
}

impl Default for TextBackground {
    fn default() -> Self {
        Self {
            id: TEXT_BACKGROUND_ID,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            flood: Flood::default(),
            composite: Composite::default(),
        }
    }
}
