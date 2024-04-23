use getset::MutGetters;
use quick_xml::DeError;
use serde::Serialize;

use crate::InformationLayer;

#[derive(Serialize, MutGetters)]
pub(crate) struct InformationGroup {
    #[serde(rename = "g", skip_serializing_if = "Vec::is_empty")]
    #[getset(get_mut = "pub")]
    groups: Vec<InformationLayer>,
    #[serde(rename = "@id")]
    id: &'static str,
}

impl InformationGroup {
    /// Creates a new [`InformationGroup`] with capacity for each [`ProcessingGroup`].
    pub(crate) fn new(number_of_cores: &usize) -> Self {
        Self {
            groups: Vec::with_capacity(*number_of_cores),
            id: "information",
        }
    }

    /// Generates a String to include in an SVG update by serialising [`InformationGroup`].
    /// It returns the serialised [`InformationGroup`] without the main `<g>`.
    pub(crate) fn update_string(&self) -> Result<String, DeError> {
        let groups = quick_xml::se::to_string_with_root("g", &self.groups)?;

        Ok(groups)
    }
}
