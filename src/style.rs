use const_format::concatcp;
use getset::{Getters, MutGetters};
use serde::Serialize;

pub(crate) const DEFAULT_FILL: &str = "#e5e5e5";
pub(crate) const BASE_FILL_CLASS_NAME: &'static str = "baseFill";
pub(crate) const EDGE_DATA_CLASS_NAME: &'static str = "edgeData";

const FONT_FACE: &str = concatcp!(
    r#"
@font-face {
    font-family: "Roboto Mono";
    src: url(data:font/truetype;charset=utf-8;base64,"#,
    include_str!("assets/roboto_mono_base64.txt"),
    r#") format("truetype");
}"#
);

const BASE_STYLE: &str = concatcp!(
    FONT_FACE,
    "\n.",
    BASE_FILL_CLASS_NAME,
    "{fill: ",
    DEFAULT_FILL,
    ";}"
);

static DEFAULT_STYLE: &str = concatcp!(BASE_STYLE, "\n.", EDGE_DATA_CLASS_NAME, "{display: none;}");

/// Object representation of SVG `<style>`.
#[derive(Serialize, MutGetters, Getters)]
pub(crate) struct Style {
    #[serde(rename = "$text")]
    #[getset(get_mut = "pub", get = "pub")]
    css: String,
}

impl Style {
    /// Generates a new [`Style`] instance with only [`BASE_STYLE`].
    pub(crate) fn base() -> Self {
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
