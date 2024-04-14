//! SVG generation library for ManyCore systems.
//!
//! Provides utilities to generate and customise an SVG file rerpresenting a ManyCore system.

mod clip_path;
mod connections_group;
mod defs;
mod error;
mod information_layer;
mod marker;
mod processing_group;
mod render_settings;
mod sinks_sources_layer;
mod style;
mod text_background;
mod view_box;

use std::{
    cmp::{max, min},
    collections::BTreeSet,
};

pub use clip_path::*;
use connections_group::*;
use defs::*;
pub use error::*;
use getset::{Getters, MutGetters, Setters};
use information_layer::*;
use marker::*;
use processing_group::*;
pub use render_settings::*;
use sinks_sources_layer::SinksSourcesGroup;
pub use view_box::*;

use manycore_parser::{ManycoreSystem, RoutingTarget, WithID, BORDER_ROUTERS_KEY, ROUTING_KEY};

use quick_xml::DeError;
use serde::Serialize;
use style::Style;

pub type CoordinateT = i32;

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

    pub fn update_string(&self) -> Result<String, DeError> {
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

#[derive(Serialize, Setters)]
pub struct Root {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "@clip-path", skip_serializing_if = "Option::is_none")]
    clip_path: Option<&'static str>,
    #[serde(rename = "g")]
    processing_group: ProcessingParentGroup,
    #[serde(rename = "g")]
    connections_group: ConnectionsParentGroup,
    #[serde(rename = "g")]
    information_group: InformationGroup,
    #[serde(rename = "g")]
    sinks_sources_group: SinksSourcesGroup,
}

#[derive(Getters, Clone, Copy)]
#[getset(get = "pub")]
struct TopLeft {
    x: CoordinateT,
    y: CoordinateT,
}

#[derive(Getters, Clone, Copy, Debug)]
#[getset(get = "pub")]
struct Offsets {
    left: CoordinateT,
    top: CoordinateT,
    right: CoordinateT,
    bottom: CoordinateT,
}

impl Offsets {
    fn new() -> Self {
        Self {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        }
    }

    pub fn update(&mut self, other: Offsets) {
        self.left = min(self.left, other.left);
        self.top = min(self.top, other.top);
        self.right = max(self.right, other.right);
        self.bottom = max(self.bottom, other.bottom);
    }
}

#[derive(Serialize, Getters, MutGetters)]
#[serde(rename = "svg")]
pub struct SVG {
    #[serde(rename = "@xmlns:svg")]
    xmlns_svg: &'static str,
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "@preserveAspectRation")]
    preserve_aspect_ratio: &'static str,
    #[serde(rename = "@class")]
    class: &'static str,
    #[serde(rename = "@viewBox")]
    #[getset(get = "pub", get_mut = "pub")]
    view_box: ViewBox,
    defs: Defs,
    style: Style,
    #[serde(rename = "clipPath", skip_serializing_if = "Option::is_none")]
    clip_path: Option<ClipPath>,
    #[serde(rename = "g")]
    #[getset(get_mut = "pub")]
    root: Root,
    #[serde(skip)]
    rows: u8,
    #[serde(skip)]
    columns: u8,
    #[serde(skip)]
    #[getset(get = "pub")]
    width: CoordinateT,
    #[serde(skip)]
    #[getset(get = "pub")]
    height: CoordinateT,
    #[serde(skip)]
    top_left: TopLeft,
    #[serde(skip)]
    base_view_box: ViewBox,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    style: String,
    information_group: String,
    view_box: String,
}

impl TryFrom<&SVG> for String {
    type Error = DeError;

    fn try_from(svg: &SVG) -> Result<Self, Self::Error> {
        quick_xml::se::to_string(svg)
    }
}

impl TryFrom<&ManycoreSystem> for SVG {
    type Error = SVGError;
    fn try_from(manycore: &ManycoreSystem) -> Result<Self, Self::Error> {
        let columns = *manycore.columns();
        let rows = *manycore.rows();

        let columns_coord: CoordinateT = columns.into();
        let rows_coord: CoordinateT = rows.into();

        let width = (columns_coord * BLOCK_LENGTH)
            + ((columns_coord - 1) * BLOCK_DISTANCE)
            + CORE_ROUTER_STROKE_WIDTH.saturating_mul(2);
        let height = (rows_coord * BLOCK_LENGTH)
            + ((rows_coord - 1) * BLOCK_DISTANCE)
            + CORE_ROUTER_STROKE_WIDTH.saturating_mul(2);

        let top_left = TopLeft {
            x: width.saturating_div(2).saturating_mul(-1),
            y: height.saturating_div(2).saturating_mul(-1),
        };

        let mut ret = SVG::new(
            &manycore.cores().list().len(),
            rows,
            columns,
            width,
            height,
            top_left,
        );

        let mut r: u8 = 0;

        let cores = manycore.cores().list();
        let borders = manycore.borders();

        let mut min_task_start = None;
        let mut has_bottom_task = false;
        for (i, core) in cores.iter().enumerate() {
            let c = u8::try_from(i % usize::try_from(columns).expect("8 bits must fit in a usize. I have no idea what you're trying to run this on, TI TMS 1000?")).expect(
                "Somehow, modulus on an 8 bit number gave a number that does not fit in 8 bits (your ALU re-invented mathematics).",
            );

            if i > 0 && c == 0 {
                r += 1;
            }

            let r_coord: CoordinateT = r.into();
            let c_coord: CoordinateT = c.into();

            // Generate processing group
            let processing_group = ProcessingGroup::new(
                &r_coord,
                &c_coord,
                core.id(),
                core.allocated_task(),
                &top_left,
            )?;

            // Check if viewBox needs to be extended left
            if c == 0 {
                if let Some(task_start) = processing_group.task_start() {
                    if let Some(min_task_start_value) = min_task_start {
                        min_task_start = Some(min(min_task_start_value, task_start));
                    } else {
                        min_task_start = Some(task_start);
                    }
                }
            }

            // Check if viewBox needs to be extended bottom
            if r == (rows - 1) {
                if let Some(_) = core.allocated_task() {
                    has_bottom_task = true;
                }
            }

            // Generate connections group
            ret.root
                .connections_group
                .add_connections(core, &r_coord, &c_coord, columns, rows, &top_left);

            // Generate borders
            if let Some(edge_position) = core.is_on_edge(columns, rows) {
                let (router_x, router_y) = processing_group.router().move_coordinates();

                // Remember that index always corresponts to core ID.
                ret.root.sinks_sources_group.insert(
                    edge_position,
                    router_x,
                    router_y,
                    match borders {
                        Some(borders) => borders.core_border_map().get(&i),
                        None => None,
                    },
                );
            }

            // Store processing group
            ret.root.processing_group.g_mut().push(processing_group);
        }

        // Extend viewBox
        if let Some(min_task_start) = min_task_start {
            ret.extend_base_view_box_left(
                min_task_start
                    .abs()
                    .saturating_sub(ret.top_left.x.abs())
                    .saturating_add(TASK_RECT_STROKE),
            );
        }
        if has_bottom_task {
            ret.extend_base_view_box_bottom(TASK_BOTTOM_OFFSET);
        }

        Ok(ret)
    }
}

fn no_processing_group(index: usize) -> SVGError {
    SVGError::new(SVGErrorKind::GenerationError(format!("Could not retrieve SVG group for core with ID {}. Something weent wrong generating the SVG, please try again.", index)))
}

impl SVG {
    fn new(
        number_of_cores: &usize,
        rows: u8,
        columns: u8,
        width: CoordinateT,
        height: CoordinateT,
        top_left: TopLeft,
    ) -> Self {
        let view_box = ViewBox::new(width, height, &top_left);

        Self {
            width,
            height,
            xmlns_svg: "http://www.w3.org/2000/svg",
            xmlns: "http://www.w3.org/2000/svg",
            preserve_aspect_ratio: "xMidYMid meet",
            class: "mx-auto",
            view_box,
            defs: Defs::default(),
            style: Style::default(),
            clip_path: None,
            root: Root {
                id: "mainGroup",
                clip_path: None,
                processing_group: ProcessingParentGroup::new(number_of_cores),
                connections_group: ConnectionsParentGroup::default(),
                information_group: InformationGroup::new(number_of_cores),
                sinks_sources_group: SinksSourcesGroup::new(rows, columns),
            },
            rows,
            columns,
            top_left,
            base_view_box: view_box,
        }
    }

    fn extend_base_view_box_left(&mut self, left: CoordinateT) {
        self.view_box.extend_left(left);
        self.base_view_box.extend_left(left);
        self.width = self.width.saturating_add(left);
    }

    fn extend_base_view_box_bottom(&mut self, bottom: CoordinateT) {
        self.view_box.extend_bottom(bottom);
        self.base_view_box.extend_bottom(bottom);
        self.height = self.height.saturating_add(bottom);
    }

    pub fn update_configurable_information(
        &mut self,
        manycore: &mut ManycoreSystem,
        configuration: &mut Configuration,
    ) -> Result<UpdateResult, SVGError> {
        // let show_sinks_sources = configuration.sinks_sources().is_some_and(|is_true| is_true);
        let not_empty_configuration = !configuration.core_config().is_empty()
            || !configuration.router_config().is_empty()
            || !configuration.channel_config().is_empty();

        // Compute routing if requested
        let (links_with_load, routing_configuration) =
            match configuration.channel_config_mut().remove(ROUTING_KEY) {
                Some(configuration) => match configuration {
                    FieldConfiguration::Routing(routing_configuration) => (
                        Some(manycore.route(routing_configuration.algorithm())?),
                        Some(routing_configuration),
                    ),
                    _ => (None, None), // Invalid configuration option
                },
                None => (None, None),
            };

        // Clear information groups. Clear will keep memory allocated, hopefully less heap allocation penalties.
        self.root.information_group.groups.clear();
        // Reset viewbox
        self.view_box.restore_from(&self.base_view_box);

        // Expand viewBox and adjust css if required (Sinks and Sources)
        // Always reset CSS. If user deselects all options and clicks apply, they expect the base render to show.
        let mut has_edge_routers = false;
        if let Some(border_routers_configuration) = configuration
            .channel_config_mut()
            .remove(BORDER_ROUTERS_KEY)
        {
            match border_routers_configuration {
                FieldConfiguration::Boolean(show_border_routers) => {
                    if show_border_routers {
                        has_edge_routers = true;

                        self.style = Style::base(); // CSS

                        // Expand viewBox for edges
                        self.view_box.insert_edges();
                    } else {
                        self.style = Style::default(); // CSS
                    }
                }
                _ => {
                    self.style = Style::default(); // CSS
                }
            }
        } else {
            self.style = Style::default(); // CSS
        }

        // Closure to get core loads
        let get_core_loads = |i: &usize| {
            if let Some(links_loads) = links_with_load.as_ref() {
                let mut ret = BTreeSet::new();

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

        let mut offsets = Offsets::new();
        if not_empty_configuration {
            let borders = manycore.borders();
            let sources = match borders {
                Some(borders) => Some(borders.sources()),
                None => None,
            };

            for (i, core) in manycore.cores().list().iter().enumerate() {
                let core_loads = get_core_loads(&i);

                let processing_group = self
                    .root
                    .processing_group
                    .g()
                    .get(i)
                    .ok_or(no_processing_group(i))?;

                self.root
                    .information_group
                    .groups
                    .push(InformationLayer::new(
                        self.rows,
                        self.columns,
                        configuration,
                        core,
                        match borders {
                            Some(borders) => borders.core_border_map().get(&i),
                            None => None,
                        },
                        sources,
                        self.style.css_mut(),
                        core_loads.as_ref(),
                        processing_group,
                        &self.root.connections_group,
                        routing_configuration.as_ref(),
                        &mut offsets,
                    )?);
            }
        }

        // Extend viewBox to fit channel text iff no edge routers.
        // If edge routers are displayed any channel text is bound to fit.
        if !has_edge_routers {
            let mut updated_view_box = self.view_box;
            if *self.view_box.x() > offsets.left {
                updated_view_box
                    .extend_left(offsets.left.abs().saturating_sub(self.view_box.x().abs()));
            }

            let far_end = self
                .view_box
                .width()
                .saturating_sub(self.view_box.x().abs());
            if far_end < offsets.right {
                updated_view_box.extend_right(offsets.right.saturating_sub(far_end));
            }

            if *self.view_box.y() > offsets.top {
                updated_view_box
                    .extend_top(offsets.top.abs().saturating_sub(self.view_box.y().abs()));
            }

            let far_bottom = self
                .view_box
                .height()
                .saturating_sub(self.view_box.y().abs());
            if far_bottom < offsets.bottom {
                updated_view_box.extend_bottom(offsets.bottom.saturating_sub(far_bottom))
            }

            self.view_box.restore_from(&updated_view_box);
        }

        Ok(UpdateResult {
            style: self.style.css().clone(),
            information_group: self.root.information_group.update_string()?,
            view_box: String::from(&self.view_box),
        })
    }

    pub fn add_clip_path(&mut self, polygon_points: String) {
        self.clip_path = Some(ClipPath::new(polygon_points));
        self.root.clip_path = Some(USE_CLIP_PATH);
    }

    pub fn clear_clip_path(&mut self) {
        self.clip_path = None;
        self.root.clip_path = None;
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

        let svg: SVG = (&manycore)
            .try_into()
            .expect("Could not convert Manycorer to SVG.");

        let res = quick_xml::se::to_string(&svg).expect("Could not convert from SVG to string");

        let expected = read_to_string("tests/SVG1.svg")
            .expect("Could not read input test file \"tests/SVG1.svg\"");

        assert_eq!(res, expected)
        // println!("SVG1: {res}\n\n")
    }
}
