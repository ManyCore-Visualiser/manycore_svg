use serde::Serialize;

use crate::{text_background::TextBackground, Marker};

/// Object representation of SVG `<defs>`.
/// Includes a default [`Marker`] and [`TextBackground`]
#[derive(Serialize, Default)]
pub(crate) struct Defs {
    marker: Marker,
    #[serde(rename = "filter")]
    text_background: TextBackground,
}
