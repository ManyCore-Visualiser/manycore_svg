use getset::{Getters, Setters};
use serde::Serialize;

use crate::{CoordinateT, Offsets, TopLeft};

/// Object representation of the SVG `viewBox` attribute. Allows for maths operations.
#[derive(Getters, Setters, Clone, Copy, Debug, PartialEq, Eq)]
#[getset(get = "pub", set = "pub")]
pub struct ViewBox {
    pub(crate) x: CoordinateT,
    pub(crate) y: CoordinateT,
    pub(crate) width: CoordinateT,
    pub(crate) height: CoordinateT,
}

impl ViewBox {
    /// Generates a new [`ViewBox`] instance from the given parameters.
    pub(crate) fn new(width: CoordinateT, height: CoordinateT, top_left: &TopLeft) -> Self {
        Self {
            x: *top_left.x(),
            y: *top_left.y(),
            width,
            height,
        }
    }

    /// Swaps every field of the [`ViewBox`] with the provided ones and returns a clone of the instance prior to modification.
    pub fn swap(
        &mut self,
        x: CoordinateT,
        y: CoordinateT,
        width: CoordinateT,
        height: CoordinateT,
    ) -> Self {
        let old = self.clone();

        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;

        old
    }

    /// Restores a [`ViewBox`] instance from another one by replacing every field.
    pub fn restore_from(&mut self, from: &ViewBox) {
        *self = *from;
    }

    /// Utility function to extend the [`ViewBox`] to the left and adjust the width accordingly.
    pub(crate) fn extend_left(&mut self, left: CoordinateT) {
        self.x = self.x.saturating_sub(left);
        self.width = self.width.saturating_add(left);
    }

    /// Utility function to extend the [`ViewBox`] to the right (width).
    pub(crate) fn extend_right(&mut self, right: CoordinateT) {
        self.width = self.width.saturating_add(right);
    }

    /// Utility function to extend the bottom (height) of the [`ViewBox`].
    pub(crate) fn extend_bottom(&mut self, bottom: CoordinateT) {
        self.height = self.height.saturating_add(bottom);
    }

    /// Utility function to extend the top of the [`ViewBox`] and adjust the height accordingly.
    pub(crate) fn extend_top(&mut self, top: CoordinateT) {
        self.y = self.y.saturating_sub(top);
        self.height = self.height.saturating_add(top);
    }

    /// Utility function to extend whole [`ViewBox`] to fit provided [`Offsets`], if required.
    pub(crate) fn fit_offsets(&mut self, offsets: &Offsets) {
        let mut updated_view_box = *self;

        // Left
        if self.x > *offsets.left() {
            updated_view_box.extend_left(offsets.left().abs().saturating_sub(self.x.abs()));
        }

        // Right
        let far_end = self.width.saturating_sub(self.x.abs());
        if far_end < *offsets.right() {
            updated_view_box.extend_right(offsets.right().saturating_sub(far_end));
        }

        // Top
        if self.y > *offsets.top() {
            updated_view_box.extend_top(offsets.top().abs().saturating_sub(self.y.abs()));
        }

        // Bottom
        let far_bottom = self.height.saturating_sub(self.y.abs());
        if far_bottom < *offsets.bottom() {
            updated_view_box.extend_bottom(offsets.bottom().saturating_sub(far_bottom))
        }

        if *self != updated_view_box {
            self.restore_from(&updated_view_box);
        }
    }
}

impl From<&ViewBox> for String {
    fn from(view_box: &ViewBox) -> Self {
        format!(
            "{} {} {} {}",
            view_box.x, view_box.y, view_box.width, view_box.height
        )
    }
}

impl From<&Offsets> for ViewBox {
    fn from(offsets: &Offsets) -> Self {
        let left = *offsets.left();
        let top = *offsets.top();
        Self {
            x: left,
            y: top,
            width: offsets.right().saturating_add(left.abs()),
            height: offsets.bottom().saturating_add(top.abs()),
        }
    }
}

impl Serialize for ViewBox {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(String::from(self).as_str())
    }
}
