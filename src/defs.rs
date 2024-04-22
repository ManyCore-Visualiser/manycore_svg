use serde::Serialize;

use crate::Marker;

/// Object representation of SVG `<defs>`.
/// Includes a default [`Marker`].
#[derive(Serialize, Default)]
pub(crate) struct Defs {
    marker: Marker,
}
