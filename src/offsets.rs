use getset::Getters;
use manycore_parser::Directions;
use std::{
    cmp::{max, min},
    ops::Sub,
};

use crate::{
    sinks_sources_layer::{
        SinkSource, SINKS_SOURCES_SHORT_SIDE_LENGTH, SINKS_SOURCES_STROKE_WIDTH,
    },
    CoordinateT, SVGError, TextInformation, CHAR_V_PADDING,
};

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
        // East link is the only one that affects top
        let top = match direction {
            Directions::East => value
                .y()
                .sub(CHAR_V_PADDING.saturating_add(value.font_size().px().round() as CoordinateT)),
            _ => *value.y(),
        };

        // For left and right we only care about South and North directions respectively.
        // Remaining directions wouldn't affect viewBox.
        let left = match direction {
            Directions::South => value.x().saturating_sub(value.calculate_length(None)?),
            _ => *value.x(),
        };

        let right = match direction {
            Directions::North => value.x().saturating_add(value.calculate_length(None)?),
            _ => *value.x(),
        };

        Ok(Offsets::new(
            left,
            top,
            right,
            value
                .y()
                .saturating_add(*value.font_size().px() as CoordinateT),
        ))
    }

    /// Utility to generate viewBox offset from a [`SinkSource`] instance.
    pub(crate) fn from_sinksource(value: &SinkSource) -> Self {
        let top = value.rect().y();
        let left = value.rect().x();
        let right = left
            .saturating_add(*value.rect().width())
            .saturating_add(SINKS_SOURCES_STROKE_WIDTH);
        let bottom = top
            .saturating_add(SINKS_SOURCES_SHORT_SIDE_LENGTH)
            .saturating_add(SINKS_SOURCES_STROKE_WIDTH);

        Self {
            left: left.saturating_sub(SINKS_SOURCES_STROKE_WIDTH),
            top: top.saturating_sub(SINKS_SOURCES_STROKE_WIDTH),
            right,
            bottom,
        }
    }
}
