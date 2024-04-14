/// SVG defs.
/// Includes a default [`Marker`] and [`TextBackground`]

use serde::Serialize;

use crate::{text_background::TextBackground, Marker};

#[derive(Serialize, Default)]
pub(crate) struct Defs {
    marker: Marker,
    #[serde(rename = "filter")]
    text_background: TextBackground,
}