use getset::{Getters, Setters};
use serde::Serialize;

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
            y: 0,
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
        self.y = 0;
        self.width = width;
        self.height = height;
    }

    pub fn extend_left_by(&mut self, dx: i16) {
        self.x -= dx;
        self.width = self.width.wrapping_add_signed(dx);
    }

    pub fn extend_top_by(&mut self, dy: i16) {
        self.y -= dy;
        self.height = self.height.wrapping_add_signed(dy);
    }

    pub fn extend_right_by(&mut self, dx: u16) {
        self.width += dx;
    }

    pub fn extend_bottom_by(&mut self, dy: u16) {
        self.height += dy;
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
