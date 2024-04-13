use getset::{Getters, Setters};
use serde::Serialize;

use crate::{
    sinks_sources_layer::SINKS_SOURCES_GROUP_OFFSET, CoordinateT, TopLeft, FONT_SIZE_WITH_OFFSET,
};

#[derive(Getters, Setters, Clone, Copy, Debug)]
#[getset(get = "pub", set = "pub")]
pub struct ViewBox {
    x: CoordinateT,
    y: CoordinateT,
    width: CoordinateT,
    height: CoordinateT,
}

impl ViewBox {
    pub fn new(width: CoordinateT, height: CoordinateT, top_left: &TopLeft) -> Self {
        Self {
            x: *top_left.x(),
            y: *top_left.y(),
            width,
            height,
        }
    }

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

    pub fn restore_from(&mut self, from: &ViewBox) {
        *self = *from;
    }

    pub fn reset(&mut self, width: CoordinateT, height: CoordinateT) {
        self.x = 0;
        self.y = FONT_SIZE_WITH_OFFSET.wrapping_mul(-1);
        self.width = width;
        self.height = height;
    }

    pub fn insert_edges(&mut self) {
        self.x = self.x.saturating_sub(SINKS_SOURCES_GROUP_OFFSET);
        self.width = self
            .width
            .saturating_add(SINKS_SOURCES_GROUP_OFFSET.saturating_mul(2));

        self.y = self.y.saturating_sub(SINKS_SOURCES_GROUP_OFFSET);
        self.height = self
            .height
            .saturating_add(SINKS_SOURCES_GROUP_OFFSET.saturating_mul(2));
    }

    pub fn extend_left(&mut self, left: CoordinateT) {
        self.x = self.x.saturating_sub(left);
        self.width = self.width.saturating_add(left);
    }

    pub fn extend_right(&mut self, right: CoordinateT) {
        self.width = self.width.saturating_add(right);
    }

    pub fn extend_bottom(&mut self, bottom: CoordinateT) {
        self.height = self.height.saturating_add(bottom);
    }

    pub fn extend_top(&mut self, top: CoordinateT) {
        self.y = self.y.saturating_sub(top);
        self.height = self.height.saturating_add(top);
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

impl Serialize for ViewBox {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(String::from(self).as_str())
    }
}
