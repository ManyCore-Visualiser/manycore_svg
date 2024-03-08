mod connections_group;
mod exporting_aid;
mod information_layer;
mod marker;
mod processing_group;
mod render_settings;
mod style;
mod text_background;

use std::error::Error;

use connections_group::*;
use exporting_aid::*;
use information_layer::*;
use marker::*;
use processing_group::*;
pub use render_settings::*;

use manycore_parser::{ConnectionUpdateError, ManycoreSystem, WithXMLAttributes};
use quick_xml::DeError;
use serde::Serialize;
use style::Style;
use text_background::TextBackground;

static PROCESSOR_PATH: &str = "l0,100 l100,0 l0,-75 l-25,-25 l-75,0 Z";
static ROUTER_PATH: &str = "l0,-75 l100,0 l0,100 l-75,0 Z";
static UNIT_LENGTH: u16 = 175;
static SIDE_LENGTH: u16 = 100;
static HALF_SIDE_LENGTH: u16 = 50;
static OUTPUT_LINK_OFFSET: u16 = 25;
static ROUTER_OFFSET: u16 = 75;
static GROUP_DISTANCE: u16 = 120;
static MARKER_PATH: &str = "M0,0 M0,0 V8 L8,4 Z";
static MARKER_REFERENCE: &str = "url(#arrowHead)";
static CONNECTION_LENGTH: u8 = 187;
static FONT_SIZE_WITH_OFFSET: u16 = 18;

#[derive(Serialize)]
struct Defs {
    marker: Marker,
    #[serde(rename = "filter")]
    text_background: TextBackground,
}

#[derive(Serialize)]
struct InformationGroup {
    #[serde(rename = "g", skip_serializing_if = "Vec::is_empty")]
    groups: Vec<InformationLayer>,
    #[serde(rename = "@id")]
    id: &'static str,
}

impl Default for InformationGroup {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            id: "information",
        }
    }
}

impl InformationGroup {
    fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}

#[derive(Serialize)]
struct Root {
    #[serde(rename = "g")]
    processing_group: ProcessingParentGroup,
    #[serde(rename = "g")]
    connections_group: ConnectionsParentGroup,
    #[serde(rename = "g", skip_serializing_if = "InformationGroup::is_empty")]
    information_group: InformationGroup,
}

#[derive(Serialize)]
#[serde(rename = "svg")]
pub struct SVG {
    #[serde(rename = "@xmlns:svg")]
    xmlns_svg: &'static str,
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "@preserveAspectRation")]
    preserve_aspect_ratio: &'static str,
    #[serde(rename = "@class")]
    class: String,
    #[serde(rename = "@viewBox")]
    view_box: String,
    defs: Defs,
    style: Style,
    #[serde(rename = "g")]
    root: Root,
    #[serde(rename = "rect")]
    exporting_aid: ExportingAid,
    #[serde(skip)]
    coordinates_pairs: Vec<(u16, u16)>,
    #[serde(skip)]
    rows: u16,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    style: String,
    information_group: String,
}

impl TryFrom<&SVG> for String {
    type Error = DeError;

    fn try_from(svg: &SVG) -> Result<Self, Self::Error> {
        quick_xml::se::to_string(svg)
    }
}

impl Default for SVG {
    fn default() -> Self {
        Self {
            xmlns_svg: "http://www.w3.org/2000/svg",
            xmlns: "http://www.w3.org/2000/svg",
            preserve_aspect_ratio: "xMidYMid meet",
            class: String::from("w-full max-h-full"),
            view_box: String::new(),
            defs: Defs {
                marker: Marker::default(),
                text_background: TextBackground::default(),
            },
            style: Style::default(),
            root: Root {
                processing_group: ProcessingParentGroup::new(),
                connections_group: ConnectionsParentGroup::new(),
                information_group: InformationGroup::default(),
            },
            exporting_aid: ExportingAid::default(),
            coordinates_pairs: Vec::new(),
            rows: 0,
        }
    }
}

impl From<&ManycoreSystem> for SVG {
    fn from(manycore: &ManycoreSystem) -> Self {
        let mut ret = SVG::default();

        let columns = u16::from(*manycore.columns());
        ret.rows = u16::from(*manycore.rows());

        let width = (columns * UNIT_LENGTH) + ((columns - 1) * GROUP_DISTANCE);
        let height = (ret.rows * UNIT_LENGTH) + ((ret.rows - 1) * GROUP_DISTANCE);
        ret.view_box
            .push_str(&format!("0 0 {} {}", width, height + FONT_SIZE_WITH_OFFSET));

        let mut r: u8 = 0;

        let cores = manycore.cores().list();

        for (i, core) in cores.iter().enumerate() {
            // This cast here might look a bit iffy as the result of the mod
            // might not fit in 8 bits. However, since manycore.columns is 8 bits,
            // that should never happen.
            let c = (i % usize::from(*manycore.columns())) as u8;

            if i > 0 && c == 0 {
                r += 1;
            }

            let r16 = u16::from(r);
            let c16 = u16::from(c);

            ret.root
                .processing_group
                .g_mut()
                .push(ProcessingGroup::new(&r16, &c16, core.id()));

            ret.root.connections_group.add_neighbours(
                i,
                manycore.connections().get(&i),
                &r16,
                &c16,
                // &configuration.routing_config().as_ref(),
            );

            ret.coordinates_pairs.push((r16, c16));
        }

        ret
    }
}

impl SVG {
    pub fn update_configurable_information(
        &mut self,
        manycore: &mut ManycoreSystem,
        configuration: &Configuration,
    ) -> Result<UpdateResult, Box<dyn Error>> {
        let not_empty_configuration =
            !configuration.core_config().is_empty() || !configuration.router_config().is_empty();

        // Compute routing if requested
        let mut links_with_load = None;
        if let Some(algorithm) = configuration.routing_config() {
            links_with_load = match algorithm {
                manycore_parser::RoutingAlgorithms::Observed => Some(manycore.observed_route()?),
                _ => Some(manycore.task_graph_route(algorithm)?),
            }
        }

        if not_empty_configuration {
            // Reset CSS
            self.style = Style::default();
            for (i, core) in manycore.cores().list().iter().enumerate() {
                let (r, c) = &self.coordinates_pairs.get(i).ok_or(ConnectionUpdateError)?;

                let core_loads = match links_with_load.as_ref() {
                    Some(links) => links.get(&i),
                    None => None,
                };

                self.root
                    .information_group
                    .groups
                    .push(InformationLayer::new(
                        &self.rows,
                        r,
                        c,
                        configuration,
                        core,
                        &i,
                        manycore.connections(),
                        self.style.css_mut(),
                        core_loads,
                    )?);
            }
        }

        Ok(UpdateResult {
            style: quick_xml::se::to_string_with_root("style", &self.style)?,
            information_group: quick_xml::se::to_string_with_root(
                "g",
                &self.root.information_group,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use manycore_parser::ManycoreSystem;

    use super::SVG;

    #[test]
    fn can_convert_from() {
        let manycore: ManycoreSystem = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let svg: SVG = (&manycore).into();

        let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG1.svg")
            .expect("Could not read input test file \"tests/SVG1.svg\"");

        assert_eq!(res, expected)
        // println!("{}", res)
    }
}
