use serde::Serialize;

use crate::{Configuration, Core, HALF_SIDE_LENGTH, SIDE_LENGTH};

enum TextVariant {
    Router,
    Core,
}

#[derive(Serialize)]
struct TextInformation {
    #[serde(rename = "@x")]
    x: u16,
    #[serde(rename = "@y")]
    y: u16,
    #[serde(rename = "@font-size")]
    font_size: &'static str,
    #[serde(rename = "@font-family")]
    font_family: &'static str,
    #[serde(rename = "@text-anchor")]
    text_anchor: &'static str,
    #[serde(rename = "@dominant-baseline")]
    dominant_baseline: &'static str,
    #[serde(rename = "$text")]
    value: String,
}

impl TextInformation {
    fn get_coordinates_from_core(core_x: &u16, core_y: &u16) -> (u16, u16) {
        (core_x + HALF_SIDE_LENGTH, core_y + SIDE_LENGTH)
    }

    fn new(
        x: u16,
        y: u16,
        text_anchor: &'static str,
        dominant_baseline: &'static str,
        value: String,
    ) -> Self {
        Self {
            x,
            y,
            font_size: "16px",
            font_family: "Roboto Mono",
            text_anchor,
            dominant_baseline,
            value,
        }
    }
}

#[derive(Serialize, Default)]
struct ProcessingInformation {
    #[serde(rename = "text")]
    core_id: Option<TextInformation>,
    #[serde(rename = "text")]
    core_temperature: Option<TextInformation>,
    #[serde(rename = "text")]
    core_age: Option<TextInformation>,
    #[serde(rename = "text")]
    computation_time: Option<TextInformation>,
    #[serde(rename = "text")]
    router_temperature: Option<TextInformation>,
    #[serde(rename = "text")]
    router_age: Option<TextInformation>,
    #[serde(rename = "text")]
    coordinates: Option<TextInformation>,
}

#[derive(Serialize, Default)]
#[serde(rename = "g")]
pub struct InformationLayer {
    g: ProcessingInformation,
}

impl InformationLayer {
    pub fn new(r: &u16, c: &u16, configuration: &Configuration) -> Self {
        let ret = InformationLayer::default();

        if *configuration.core_ids() {

        }

        ret
    }
}