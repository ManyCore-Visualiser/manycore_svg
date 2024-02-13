use serde::Serialize;

#[derive(Serialize)]
pub struct EventsRect {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "@width")]
    width: &'static str,
    #[serde(rename = "@height")]
    height: &'static str,
    #[serde(rename = "@fill")]
    fill: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
}

impl Default for EventsRect {
    fn default() -> Self {
        Self {
            id: "events",
            width: "100%",
            height: "100%",
            fill: "none",
            stroke: "none",
        }
    }
}
