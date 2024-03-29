use const_format::concatcp;
use getset::MutGetters;
use serde::Serialize;

use crate::{sinks_sources_layer::SINK_SOURCES_ID, EDGE_CONNECTIONS_ID};

pub const DEFAULT_FILL: &str = "#e5e5e5";
pub const BASE_FILL_CLASS_NAME: &'static str = "baseFill";
static DEFAULT_STYLE: &str = concatcp!(
    ".",
    BASE_FILL_CLASS_NAME,
    "{fill: ",
    DEFAULT_FILL,
    ";}",
    "#",
    EDGE_CONNECTIONS_ID,
    ", #",
    SINK_SOURCES_ID,
    "{display: none;}"
);

static BASE_STYLE: &str = concatcp!(".", BASE_FILL_CLASS_NAME, "{fill: ", DEFAULT_FILL, ";}");

#[derive(Serialize, MutGetters)]
pub struct Style {
    #[serde(rename = "$text")]
    #[getset(get_mut = "pub")]
    css: String,
}

impl Style {
    pub fn base() -> Self {
        Self {
            css: BASE_STYLE.into(),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            css: DEFAULT_STYLE.into(),
        }
    }
}
