use getset::Getters;
use std::cmp::{max, min};

use crate::CoordinateT;

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
}
