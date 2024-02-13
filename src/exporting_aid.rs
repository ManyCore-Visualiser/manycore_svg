use serde::Serialize;

#[derive(Serialize)]
pub struct ExportingAid {
    #[serde(rename = "@width")]
    width: &'static str,
    #[serde(rename = "@height")]
    height: &'static str,
    #[serde(rename = "@fill")]
    fill: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl Default for ExportingAid {
    fn default() -> Self {
        Self {
            width: "100%",
            height: "100%",
            fill: "none",
            stroke: "#ff0000",
            stroke_width: "1",
        }
    }
}
