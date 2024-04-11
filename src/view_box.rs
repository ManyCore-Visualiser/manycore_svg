use getset::{Getters, Setters};
use serde::Serialize;

use crate::{
    sinks_sources_layer::{I_SINKS_SOURCES_GROUP_OFFSET, SINKS_SOURCES_GROUP_OFFSET},
    BLOCK_LENGTH, FONT_SIZE_WITH_OFFSET,
};

#[derive(Getters, Setters, Clone, Copy)]
#[getset(get = "pub", set = "pub")]
pub struct ViewBox {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

impl ViewBox {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            x: 0,
            // Needed to fit upper text on links
            y: FONT_SIZE_WITH_OFFSET.wrapping_mul(-1),
            width,
            height,
        }
    }

    pub fn swap(&mut self, x: i16, y: i16, width: u16, height: u16) -> Self {
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

    pub fn reset(&mut self, width: u16, height: u16) {
        self.x = 0;
        self.y = FONT_SIZE_WITH_OFFSET.wrapping_mul(-1);
        self.width = width;
        self.height = height;
    }

    pub fn insert_edges(&mut self) {
        // This offset is greater than font offset
        self.x = -I_SINKS_SOURCES_GROUP_OFFSET;
        self.width = self
            .width
            .wrapping_sub(BLOCK_LENGTH / 2)
            .saturating_add(2 * SINKS_SOURCES_GROUP_OFFSET);
        self.y = -I_SINKS_SOURCES_GROUP_OFFSET;
        self.height = self
            .height
            .saturating_add_signed(-FONT_SIZE_WITH_OFFSET)
            .saturating_add(2 * SINKS_SOURCES_GROUP_OFFSET);
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
