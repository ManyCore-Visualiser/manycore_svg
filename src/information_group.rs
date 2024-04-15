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
    /// It returns the serialised [`InformationGroup`] with the main `<g>` stripped off.
    pub(crate) fn update_string(&self) -> Result<String, DeError> {
        let dummy_xml = quick_xml::se::to_string_with_root("g", &self.groups)?;
        // 0-2+1...dummy_xml.len() - 4
        // <g>...</g>
        // e.g <g>hello</g> = 3..8
        // Start is inclusive, end is exclusive
        let dummy_len = dummy_xml.len();
        let inner_content;

        if dummy_len > 6 {
            inner_content = &dummy_xml[3..(dummy_xml.len() - 4)];
        } else {
            inner_content = "";
        }

        // We must return a string here because without allocation the string slice would be dropped.
        Ok(String::from(inner_content))
    }
}