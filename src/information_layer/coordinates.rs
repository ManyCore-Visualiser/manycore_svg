use std::collections::BTreeMap;

use manycore_parser::{SystemDimensionsT, COORDINATES_KEY};

use crate::{
    CoordinateT, CoordinatesOrientation, FieldConfiguration, InformationLayer,
    ProcessedBaseConfiguration, SVGError, TextInformation, HALF_SIDE_LENGTH, SIDE_LENGTH,
};

/// Generates coordinates text.
pub(crate) fn make_coordinates(
    core_config: &BTreeMap<String, FieldConfiguration>,
    core_x: &CoordinateT,
    core_y: &CoordinateT,
    rows: SystemDimensionsT,
    r: &CoordinateT,
    c: &CoordinateT,
    ret: &mut InformationLayer,
    processed_base_configuration: &ProcessedBaseConfiguration,
) -> Result<(), SVGError> {
    if let Some(order_config) = core_config.get(COORDINATES_KEY) {
        // Text coordinates
        let x = core_x + HALF_SIDE_LENGTH;
        let y = core_y + SIDE_LENGTH;

        // (X, Y) text repesentation
        let (cx, cy) = match order_config {
            FieldConfiguration::Coordinates { orientation } => match orientation {
                CoordinatesOrientation::B => Ok((c + 1, CoordinateT::from(rows) - r)),
                CoordinatesOrientation::T => Ok((c + 1, r + 1)),
            },
            fc => Err(SVGError::new(crate::SVGErrorKind::GenerationError(
                format!(
                    "Unsupported configuration for coordinates: {}",
                    fc.type_str()
                ),
            ))),
        }?;

        ret.coordinates = Some(TextInformation::new(
            x,
            y,
            *processed_base_configuration.attribute_font_size(),
            "middle",
            "text-before-edge",
            None,
            None,
            format!("({},{})", cx, cy),
        ));
    }

    Ok(())
}
