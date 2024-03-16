mod connections_group;
mod exporting_aid;
mod information_layer;
mod marker;
mod processing_group;
mod render_settings;
mod sinks_sources_layer;
mod style;
mod text_background;
mod view_box;

use std::{collections::BTreeSet, error::Error};

use connections_group::*;
use exporting_aid::*;
use getset::Getters;
use information_layer::*;
use marker::*;
use processing_group::*;
pub use render_settings::*;
use sinks_sources_layer::{
    SinkSource, SinkSourceVariant, SinksSourcesGroup, I_SINKS_SOURCES_GROUP_OFFSET,
    SINKS_SOURCES_GROUP_OFFSET,
};
pub use view_box::*;

use manycore_parser::{
    ConnectionUpdateError, ManycoreSystem, SinkSourceDirection, WithXMLAttributes,
};
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
static CONNECTION_LENGTH: u16 = 187;
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
    #[serde(
        rename = "g",
        skip_serializing_if = "SinksSourcesGroup::should_serialise"
    )]
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
    view_box: ViewBox,
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
    #[serde(skip)]
    columns: u16,
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
        let columns = u16::from(*manycore.columns());
        let rows = u16::from(*manycore.rows());
        let width =
            (columns * UNIT_LENGTH) + ((columns - 1) * GROUP_DISTANCE) + TASK_CIRCLE_TOTAL_OFFSET;
        let height = (rows * UNIT_LENGTH)
            + ((rows - 1) * GROUP_DISTANCE)
            + TASK_CIRCLE_TOTAL_OFFSET
            + FONT_SIZE_WITH_OFFSET;

        let mut ret = SVG::new(&manycore.cores().list().len(), rows, columns, width, height);

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

            ret.root.processing_group.g_mut().push(ProcessingGroup::new(
                &r16,
                &c16,
                core.id(),
                core.allocated_task(),
            ));

            ret.root.connections_group.add_neighbours(
                i,
                manycore.connections().get(&i),
                &r16,
                &c16,
            );

            ret.coordinates_pairs.push((r16, c16));
        }

        ret
    }
}

impl SVG {
    fn new(number_of_cores: &usize, rows: u16, columns: u16, width: u16, height: u16) -> Self {
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
                processing_group: ProcessingParentGroup::new(number_of_cores),
                connections_group: ConnectionsParentGroup::new(),
                information_group: InformationGroup::new(number_of_cores),
                sinks_sources_group: SinksSourcesGroup::new(rows, columns),
            },
            exporting_aid: ExportingAid::default(),
            coordinates_pairs: Vec::with_capacity(*number_of_cores),
            rows,
            columns,
        }
    }
    pub fn update_configurable_information(
        &mut self,
        manycore: &mut ManycoreSystem,
        configuration: &Configuration,
    ) -> Result<UpdateResult, Box<dyn Error>> {
        let show_sinks_sources = configuration.sinks_sources().is_some_and(|is_true| is_true);
        let not_empty_configuration = !configuration.core_config().is_empty()
            || !configuration.router_config().is_empty()
            || configuration.routing_config().is_some()
            || show_sinks_sources;

        // Compute routing if requested
        let mut links_with_load = None;
        if let Some(algorithm) = configuration.routing_config() {
            links_with_load = Some(manycore.route(algorithm)?)
        }

        // Always reset CSS. If user deselects all options and clicks apply, they expect the base render to show.
        self.style = Style::default();
        // Clear information groups. Clear will keep memory allocated, hopefully less heap allocation penalties.
        self.root.information_group.groups.clear();
        // Reset viewbox
        self.view_box.reset(self.width, self.height);
        // Clear sinks/sources group
        self.root.sinks_sources_group.clear();

        // Expand viewBox if required (Sinks and Sources)
        if show_sinks_sources {
            self.view_box.extend_left_by(I_SINKS_SOURCES_GROUP_OFFSET);
            self.view_box.extend_right_by(SINKS_SOURCES_GROUP_OFFSET);
            self.view_box.extend_top_by(I_SINKS_SOURCES_GROUP_OFFSET);
            self.view_box.extend_bottom_by(SINKS_SOURCES_GROUP_OFFSET);
        }

        if not_empty_configuration {
            for (i, core) in manycore.cores().list().iter().enumerate() {
                // Information group
                let (r, c) = &self.coordinates_pairs.get(i).ok_or(ConnectionUpdateError)?;

                let core_loads = match links_with_load.as_ref() {
                    Some(links) => links.get(&i),
                    None => None,
                };

                let channels = manycore.cores().list()[i].channels().as_ref();

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
                        channels,
                    )?);

                // Sinks/Sources group
                let is_on_the_edge =
                    (*r == 0 || *r == (self.rows - 1)) || (*c == 0 || *c == (self.columns - 1));
                if show_sinks_sources && is_on_the_edge {
                    let (router_x, mut router_y) = Router::get_move_coordinates(r, c);
                    router_y -= ROUTER_OFFSET;

                    let mut edge_routers_expected_directions: BTreeSet<SinkSourceDirection>;

                    // Determine set of expected directions
                    if *c % (self.columns - 1) == 0 {
                        // Top edge
                        if *r == 0 {
                            // Top Left corner
                            if *c == 0 {
                                edge_routers_expected_directions = BTreeSet::from([
                                    SinkSourceDirection::North,
                                    SinkSourceDirection::West,
                                ]);
                            }
                            // Top right corner
                            else {
                                edge_routers_expected_directions = BTreeSet::from([
                                    SinkSourceDirection::North,
                                    SinkSourceDirection::East,
                                ]);
                            }
                        }
                        // Bottom edge
                        else if *r == (self.rows - 1) {
                            // Bottom left corner
                            if *c == 0 {
                                edge_routers_expected_directions = BTreeSet::from([
                                    SinkSourceDirection::South,
                                    SinkSourceDirection::West,
                                ]);
                            }
                            // Bottom right corner
                            else {
                                edge_routers_expected_directions = BTreeSet::from([
                                    SinkSourceDirection::South,
                                    SinkSourceDirection::East,
                                ]);
                            }
                        }
                        // Left side
                        else if *c == 0 {
                            edge_routers_expected_directions =
                                BTreeSet::from([SinkSourceDirection::West]);
                        }
                        // Right side
                        else {
                            edge_routers_expected_directions =
                                BTreeSet::from([SinkSourceDirection::East]);
                        }
                    }
                    // Top edge
                    else if *r == 0 {
                        edge_routers_expected_directions =
                            BTreeSet::from([SinkSourceDirection::North]);
                    }
                    // Bottom edge
                    else {
                        edge_routers_expected_directions =
                            BTreeSet::from([SinkSourceDirection::South]);
                    }

                    if let Some(sink) = manycore.sinks().get(&i) {
                        self.root.sinks_sources_group.push(SinkSource::new(
                            &router_x,
                            &router_y,
                            sink.direction(),
                            SinkSourceVariant::Sink,
                        ));

                        edge_routers_expected_directions.remove(sink.direction());
                    }

                    if let Some(source) = manycore.sources().get(&i) {
                        self.root.sinks_sources_group.push(SinkSource::new(
                            &router_x,
                            &router_y,
                            source.direction(),
                            SinkSourceVariant::Source,
                        ));

                        edge_routers_expected_directions.remove(source.direction());
                    }

                    for direction in edge_routers_expected_directions.iter() {
                        self.root.sinks_sources_group.push(SinkSource::new(
                            &router_x,
                            &router_y,
                            direction,
                            SinkSourceVariant::None,
                        ));
                    }
                }
            }
        }

        Ok(UpdateResult {
            style: quick_xml::se::to_string_with_root("style", &self.style)?,
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
    }
}
