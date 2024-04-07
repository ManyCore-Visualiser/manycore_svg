use std::collections::BTreeMap;

use manycore_parser::COORDINATES_KEY;

use crate::{
    CoordinatesOrientation, FieldConfiguration, InformationLayer, TextInformation,
    HALF_SIDE_LENGTH, SIDE_LENGTH,
};

pub fn make_coordinates(
    core_config: &BTreeMap<String, FieldConfiguration>,
    core_x: &u16,
    core_y: &u16,
    rows: u16,
    r: &u16,
    c: &u16,
    ret: &mut InformationLayer,
) {
    if let Some(order_config) = core_config.get(COORDINATES_KEY) {
        let x = core_x + HALF_SIDE_LENGTH;
        let y = core_y + SIDE_LENGTH;

        let (cx, cy) = match order_config {
            FieldConfiguration::Coordinates(order) => match order {
                CoordinatesOrientation::B => (u16::from(rows) - r, c + 1),
                CoordinatesOrientation::T => (r + 1, c + 1),
            },
            _ => (r + 1, c + 1), // Don't know what happened. Wrong enum variant, default to top left.
        };

        ret.coordinates = Some(TextInformation::new(
            x,
            y,
            "middle",
            "text-before-edge",
            None,
            None,
            format!("({},{})", cx, cy),
        ));
    }
}
