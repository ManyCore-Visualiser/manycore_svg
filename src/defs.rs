use std::ops::Mul;

use getset::MutGetters;
use serde::Serialize;

use crate::{ClipPath, Marker};

/// Object representation of SVG `<defs>`.
/// Includes a default [`Marker`] and the required [`ClipPath`]s for core and router information layer.
#[derive(Serialize, MutGetters)]
pub(crate) struct Defs {
    marker: Marker,
    #[serde(rename = "clipPath")]
    #[getset(get_mut = "pub")]
    clip_paths: Vec<ClipPath>,
}

impl Defs {
    /// Creates a new [`Defs`] instance with the required capacity for [`ClipPath`]s.
    pub(crate) fn new(number_of_cores: &usize) -> Self {
        Self {
            marker: Default::default(),
            // We need capacity for twice the number of cores to fit
            // both cores and routers' clip paths.
            // We add one to potentially store freeform clip path.
            clip_paths: Vec::with_capacity(number_of_cores.mul(2).saturating_add(1)),
        }
    }
}
