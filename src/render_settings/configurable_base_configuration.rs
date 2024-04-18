use serde::Serialize;

use crate::{
    FontSizeT, DEFAULT_ATTRIBUTE_FONT_SIZE, DEFAULT_TASK_FONT_SIZE, MAXIMUM_ATTRIBUTE_FONT_SIZE,
    MAXIMUM_TASK_FONT_SIZE, MINIMUM_ATTRIBUTE_FONT_SIZE, MINIMUM_TASK_FONT_SIZE,
};

/// Enum whose variants represents specific attribute configuration details for
/// [`ConfigurableBaseConfiguration`] attributes.
#[derive(Serialize, Clone, Copy)]
#[serde(tag = "type")]
pub(crate) enum ConfigurableBaseConfigurationAttributeSpecifics {
    FontSize {
        default: FontSizeT,
        display: &'static str,
        min: FontSizeT,
        max: FontSizeT,
    },
}

/// This struct is used to inform the front-end of what fields are part
/// of the [`BaseConfiguration`].
/// We serialise fields in snake_case because we process them on the frontend.
#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub struct ConfigurableBaseConfiguration {
    attribute_font_size: ConfigurableBaseConfigurationAttributeSpecifics,
    task_font_size: ConfigurableBaseConfigurationAttributeSpecifics,
}

pub static CONFIGURABLE_BASE_CONFIGURATION: ConfigurableBaseConfiguration =
    ConfigurableBaseConfiguration {
        attribute_font_size: ConfigurableBaseConfigurationAttributeSpecifics::FontSize {
            default: DEFAULT_ATTRIBUTE_FONT_SIZE,
            display: "Attribute font size",
            min: MINIMUM_ATTRIBUTE_FONT_SIZE,
            max: MAXIMUM_ATTRIBUTE_FONT_SIZE,
        },
        task_font_size: ConfigurableBaseConfigurationAttributeSpecifics::FontSize {
            default: DEFAULT_TASK_FONT_SIZE,
            display: "Task font size",
            min: MINIMUM_TASK_FONT_SIZE,
            max: MAXIMUM_TASK_FONT_SIZE,
        },
    };
