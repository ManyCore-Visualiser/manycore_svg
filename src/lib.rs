mod connections_group;
mod error;
mod exporting_aid;
mod information_layer;
mod marker;
mod processing_group;
mod render_settings;
mod sinks_sources_layer;
mod style;
mod text_background;
mod view_box;

use std::collections::BTreeMap;

use connections_group::*;
pub use error::*;
use exporting_aid::*;
use getset::Getters;
use information_layer::*;
use marker::*;
use processing_group::*;
pub use render_settings::*;
use sinks_sources_layer::{
    SinksSourcesGroup, I_SINKS_SOURCES_GROUP_OFFSET, SINKS_SOURCES_GROUP_OFFSET,
};
pub use view_box::*;

use manycore_parser::{ManycoreSystem, RoutingTarget, WithXMLAttributes};

use quick_xml::DeError;
use serde::{Serialize, Serializer};
use style::Style;
use text_background::TextBackground;

static PROCESSOR_PATH: &str = "l0,100 l100,0 l0,-75 l-25,-25 l-75,0 Z";
static ROUTER_PATH: &str = "l0,-75 l100,0 l0,100 l-75,0 Z";
static UNIT_LENGTH: u16 = 175;
static SIDE_LENGTH: u16 = 100;
static HALF_SIDE_LENGTH: u16 = 50;
static OUTPUT_LINK_OFFSET: u16 = 25;
static ROUTER_OFFSET: u16 = 75;
static HALF_ROUTER_OFFSET: u16 = ROUTER_OFFSET.div_ceil(2);
// static GROUP_DISTANCE: u16 = 120;
static MARKER_PATH: &str = "M0,0 M0,0 V8 L8,4 Z";
static MARKER_REFERENCE: &str = "url(#arrowHead)";
// static CONNECTION_LENGTH: u16 = 187;
static CONNECTION_LENGTH: u16 = ROUTER_OFFSET.saturating_mul(4);
static GROUP_DISTANCE: u16 = CONNECTION_LENGTH
    .saturating_sub(ROUTER_OFFSET)
    .saturating_add(MARKER_HEIGHT);
static MARKER_HEIGHT: u16 = 8;
static HALF_CONNECTION_LENGTH: u16 = (CONNECTION_LENGTH + MARKER_HEIGHT) / 2;
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

impl InformationGroup {
    fn new(number_of_cores: &usize) -> Self {
        Self {
            groups: Vec::with_capacity(*number_of_cores),
            id: "information",
        }
    }
    fn should_serialise(&self) -> bool {
        self.groups.is_empty()
    }
}

#[derive(Serialize)]
struct Root {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "g")]
    processing_group: ProcessingParentGroup,
    #[serde(rename = "g")]
    connections_group: ConnectionsParentGroup,
    #[serde(
        rename = "g",
        skip_serializing_if = "InformationGroup::should_serialise"
    )]
    information_group: InformationGroup,
    #[serde(rename = "g")]
    sinks_sources_group: SinksSourcesGroup,
}

#[derive(Serialize, Getters)]
#[serde(rename = "svg")]
pub struct SVG {
    #[serde(skip)]
    #[getset(get = "pub")]
    width: u16,
    #[serde(skip)]
    #[getset(get = "pub")]
    height: u16,
    #[serde(rename = "@xmlns:svg")]
    xmlns_svg: &'static str,
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "@preserveAspectRation")]
    preserve_aspect_ratio: &'static str,
    #[serde(rename = "@class")]
    class: String,
    #[serde(rename = "@viewBox")]
    #[getset(get = "pub")]
    view_box: ViewBox,
    defs: Defs,
    style: Style,
    #[serde(rename = "g")]
    root: Root,
    #[serde(rename = "rect")]
    exporting_aid: ExportingAid,
    #[serde(skip)]
    rows: u8,
    #[serde(skip)]
    columns: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    style: String,
    information_group: String,
    sinks_sources_group: String,
    view_box: String,
}

impl TryFrom<&SVG> for String {
    type Error = DeError;

    fn try_from(svg: &SVG) -> Result<Self, Self::Error> {
        quick_xml::se::to_string(svg)
    }
}

impl From<&ManycoreSystem> for SVG {
    fn from(manycore: &ManycoreSystem) -> Self {
        let columns = *manycore.columns();
        let rows = *manycore.rows();

        let columns_u16 = u16::from(columns);
        let rows_u16 = u16::from(rows);
        let width = (columns_u16 * UNIT_LENGTH)
            + ((columns_u16 - 1) * GROUP_DISTANCE)
            + TASK_CIRCLE_TOTAL_OFFSET;
        let height = (rows_u16 * UNIT_LENGTH)
            + ((rows_u16 - 1) * GROUP_DISTANCE)
            + TASK_CIRCLE_TOTAL_OFFSET
            + FONT_SIZE_WITH_OFFSET;

        let mut ret = SVG::new(&manycore.cores().list().len(), rows, columns, width, height);

        let mut r: u8 = 0;

        let cores = manycore.cores().list();

        for (i, core) in cores.iter().enumerate() {
            // This cast here might look a bit iffy as the result of the mod
            // might not fit in 8 bits. However, since manycore.columns is 8 bits,
            // that should never happen.
            let c = (i % usize::from(columns)) as u8;

            if i > 0 && c == 0 {
                r += 1;
            }

            let r16 = u16::from(r);
            let c16 = u16::from(c);

            // Generate processing group
            let processing_group =
                ProcessingGroup::new(&r16, &c16, core.id(), core.allocated_task());

            // Generate connections group
            ret.root
                .connections_group
                .add_connections(core, &r16, &c16, columns, rows);

            // Generate borders
            if let Some(edge_position) = core.is_on_edge(columns, rows) {
                let (router_x, router_y) = processing_group.router().move_coordinates();

                ret.root
                    .sinks_sources_group
                    .insert(edge_position, router_x, router_y);
            }

            // Store processing group
            ret.root
                .processing_group
                .g_mut()
                .insert(*core.id(), processing_group);
        }

        ret
    }
}

impl SVG {
    fn new(number_of_cores: &usize, rows: u8, columns: u8, width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            xmlns_svg: "http://www.w3.org/2000/svg",
            xmlns: "http://www.w3.org/2000/svg",
            preserve_aspect_ratio: "xMidYMid meet",
            class: String::from("w-full max-h-full"),
            view_box: ViewBox::new(width, height),
            defs: Defs {
                marker: Marker::default(),
                text_background: TextBackground::default(),
            },
            style: Style::default(),
            root: Root {
                id: "mainGroup",
                processing_group: ProcessingParentGroup::new(),
                connections_group: ConnectionsParentGroup::default(),
                information_group: InformationGroup::new(number_of_cores),
                sinks_sources_group: SinksSourcesGroup::new(rows, columns),
            },
            exporting_aid: ExportingAid::default(),
            rows,
            columns,
        }
    }

    pub fn update_configurable_information(
        &mut self,
        manycore: &mut ManycoreSystem,
        configuration: &Configuration,
    ) -> Result<UpdateResult, SVGError> {
        let show_sinks_sources = configuration.sinks_sources().is_some_and(|is_true| is_true);
        let not_empty_configuration = !configuration.core_config().is_empty()
            || !configuration.router_config().is_empty()
            || configuration.routing_config().is_some()
            || show_sinks_sources;

        // Compute routing if requested
        let links_with_load = match configuration.routing_config() {
            Some(algorithm) => Some(manycore.route(algorithm)?),
            None => None,
        };

        // Clear information groups. Clear will keep memory allocated, hopefully less heap allocation penalties.
        self.root.information_group.groups.clear();
        // Reset viewbox
        self.view_box.reset(self.width, self.height);

        // Expand viewBox and adjust css if required (Sinks and Sources)
        // Always reset CSS. If user deselects all options and clicks apply, they expect the base render to show.
        if show_sinks_sources {
            self.style = Style::base(); // CSS

            self.view_box.extend_left_by(I_SINKS_SOURCES_GROUP_OFFSET);
            self.view_box.extend_right_by(SINKS_SOURCES_GROUP_OFFSET);
            self.view_box.extend_top_by(I_SINKS_SOURCES_GROUP_OFFSET);
            self.view_box.extend_bottom_by(SINKS_SOURCES_GROUP_OFFSET);
        } else {
            self.style = Style::default(); // CSS
        }

        // Closure to get core loads
        let get_core_loads = |i: &usize| {
            if let Some(links_loads) = links_with_load.as_ref() {
                let mut ret = Vec::new();

                let core_key = RoutingTarget::Core(*i);
                let sink_key = RoutingTarget::Sink(*i);

                if let Some(core_loads) = links_loads.get(&core_key) {
                    ret.extend(core_loads);
                }

                if let Some(sink_loads) = links_loads.get(&sink_key) {
                    ret.extend(sink_loads);
                }

                if ret.len() > 0 {
                    return Some(ret);
                }
            }

            None
        };

        if not_empty_configuration {
            for (i, core) in manycore.cores().list().iter().enumerate() {
                let core_loads = get_core_loads(&i);

                // TODO: Handle unwraps
                let processing_group = self.root.processing_group.g().get(core.id()).unwrap();

                self.root
                    .information_group
                    .groups
                    .push(InformationLayer::new(
                        self.rows,
                        self.columns,
                        configuration,
                        core,
                        manycore.borders().core_source_map().get(&i),
                        manycore.borders().sources(),
                        self.style.css_mut(),
                        core_loads.as_ref(),
                        processing_group,
                        &self.root.connections_group,
                    )?);
            }
        }

        Ok(UpdateResult {
            style: self.style.css().clone(),
            information_group: quick_xml::se::to_string_with_root(
                "g",
                &self.root.information_group,
            )?,
            sinks_sources_group: quick_xml::se::to_string_with_root(
                "g",
                &self.root.sinks_sources_group,
            )?,
            view_box: String::from(&self.view_box),
        })
    }

    pub fn serialise_btreemap<S: Serializer, K, V: Serialize>(
        map: &BTreeMap<K, V>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(map.values())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use manycore_parser::ManycoreSystem;

    use super::SVG;

    // #[test]
    // fn can_convert_from() {
    //     let manycore: ManycoreSystem = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
    //         .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

    //     let svg: SVG = (&manycore).into();

    //     let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

    //     let expected = read_to_string("tests/SVG1.svg")
    //         .expect("Could not read input test file \"tests/SVG1.svg\"");

    //     // assert_eq!(res, expected)
    //     // println!("{res}")
    // }
}
