use std::collections::BTreeMap;

use manycore_parser::COORDINATES_KEY;

use crate::{
    CoordinateT, CoordinatesOrientation, FieldConfiguration, InformationLayer, TextInformation,
    DEFAULT_FONT_SIZE, HALF_SIDE_LENGTH, SIDE_LENGTH,
};

/// Generates coordinates text.
pub(crate) fn make_coordinates(
    core_config: &BTreeMap<String, FieldConfiguration>,
    core_x: &CoordinateT,
    core_y: &CoordinateT,
    rows: u8,
    r: &CoordinateT,
    c: &CoordinateT,
    ret: &mut InformationLayer,
) {
    if let Some(order_config) = core_config.get(COORDINATES_KEY) {
        // Text coordinates
        let x = core_x + HALF_SIDE_LENGTH;
        let y = core_y + SIDE_LENGTH;

        // (X, Y) text repesentation
        // TODO: This is actually (Y, X), we want (X, Y)
        let (cx, cy) = match order_config {
            FieldConfiguration::Coordinates(order) => match order {
                CoordinatesOrientation::B => (CoordinateT::from(rows) - r, c + 1),
                CoordinatesOrientation::T => (r + 1, c + 1),
            },
            _ => (r + 1, c + 1), // Don't know what happened. Wrong enum variant, default to top left.
        };

        ret.coordinates = Some(TextInformation::new(
            x,
            y,
            DEFAULT_FONT_SIZE,
            "middle",
            "text-before-edge",
            None,
            None,
            format!("({},{})", cx, cy),
        ));
    }
}
