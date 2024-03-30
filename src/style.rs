use const_format::concatcp;
use getset::MutGetters;
use serde::Serialize;

pub const DEFAULT_FILL: &str = "#e5e5e5";
pub const BASE_FILL_CLASS_NAME: &'static str = "baseFill";
pub const EDGE_DATA_CLASS_NAME: &'static str = "edgeData";
static DEFAULT_STYLE: &str = concatcp!(
    ".",
    BASE_FILL_CLASS_NAME,
    "{fill: ",
    DEFAULT_FILL,
    ";}",
    ".",
    EDGE_DATA_CLASS_NAME,
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
