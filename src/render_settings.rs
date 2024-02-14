use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct TemperatureColourSettings {
    bounds: [u8; 4],
    colours: [String; 4],
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TemperatureSettings {
    Hide,
    Text,
    Colour(TemperatureColourSettings),
}

#[derive(Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    core_temperature: TemperatureSettings,
    core_age: bool,
    core_ids: bool,
    coordinates: bool,
    computation_time: bool,
    router_temperature: TemperatureSettings,
    router_age: bool,
    link_load: bool,
    task_graph: bool,
    task_ids: bool,
    algorithm: String,
}

#[cfg(test)]
mod tests {
    use crate::{Configuration, TemperatureSettings};

    #[test]
    fn can_parse() {
        let expected_configuration = Configuration {
            core_temperature: TemperatureSettings::Hide,
            core_age: true,
            core_ids: true,
            coordinates: true,
            computation_time: false,
            router_temperature: TemperatureSettings::Hide,
            router_age: true,
            link_load: false,
            task_graph: false,
            task_ids: false,
            algorithm: String::from("RowFirst"),
        };

        println!(
            "{}",
            serde_json::to_string(&expected_configuration).unwrap()
        );
    }
}
