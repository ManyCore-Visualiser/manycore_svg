use quick_xml::DeError;

pub(crate) trait PartialUpdate {
    /// Generates a String to include in an SVG update by serialising [`Self`].
    /// It returns the serialised [`Self`] without the main `<g>`.
    fn update_string(&self) -> Result<String, DeError>;
}
