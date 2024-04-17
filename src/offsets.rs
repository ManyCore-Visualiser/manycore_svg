use getset::Getters;
use manycore_parser::Directions;
use std::{
    cmp::{max, min},
    ops::Sub,
};

use crate::{CoordinateT, SVGError, TextInformation};

/// Helper struct to calculate viewBox offsets.
#[derive(Getters, Clone, Copy, Debug, Default)]
#[getset(get = "pub")]
pub(crate) struct Offsets {
    left: CoordinateT,
    top: CoordinateT,
    right: CoordinateT,
    bottom: CoordinateT,
}

impl Offsets {
    /// Creates a new [`Offsets`] instance given each offset.
    pub(crate) fn new(
        left: CoordinateT,
        top: CoordinateT,
        right: CoordinateT,
        bottom: CoordinateT,
    ) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Updates an [`Offsets`] instance by comparring it to another to maximise viewBox size.
    pub(crate) fn update(&mut self, other: Offsets) {
        self.left = min(self.left, other.left);
        self.top = min(self.top, other.top);
        self.right = max(self.right, other.right);
        self.bottom = max(self.bottom, other.bottom);
    }

    /// Utility to generate channel text offset from a [`TextInformation`] instance.
    pub(crate) fn try_from_channel(
        value: &TextInformation,
        direction: &Directions,
    ) -> Result<Self, SVGError> {
        let top = match direction {
            Directions::North | Directions::South => *value.y(),
            _ => value.y().sub(*value.font_size().px() as CoordinateT),
        };

        Ok(Offsets::new(
            *value.x(),
            top,
            value.x().saturating_add(value.calculate_length(None)?),
            value
                .y()
                .saturating_add(*value.font_size().px() as CoordinateT),
        ))
    }
}
