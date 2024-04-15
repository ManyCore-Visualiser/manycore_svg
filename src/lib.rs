//! SVG generation library for ManyCore systems.
//!
//! Provides utilities to generate and customise an SVG file rerpresenting a ManyCore system.

mod clip_path;
mod connections_group;
mod defs;
mod error;
mod information_group;
mod information_layer;
mod marker;
mod offsets;
mod processing_group;
mod render_settings;
mod sinks_sources_layer;
mod style;
mod svg_conversions;
mod text_background;
mod view_box;

use std::collections::BTreeSet;

pub use clip_path::*;
use connections_group::*;
use defs::*;
pub use error::*;
use getset::{Getters, MutGetters, Setters};
use information_group::*;
use information_layer::*;
use marker::*;
use offsets::*;
use processing_group::*;
pub use render_settings::*;
use sinks_sources_layer::SinksSourcesGroup;
pub use view_box::*;

use manycore_parser::{ManycoreSystem, RoutingTarget, BORDER_ROUTERS_KEY, ROUTING_KEY};

use serde::Serialize;
use style::Style;

/// Type alias for SVG elements coordinates.
pub type CoordinateT = i32;

/// Object representation of the [`SVG`] main group. Everything goes in here.
/// Cores, (border) routers, channels and information are all inner groups of this group.
#[derive(Serialize, Setters)]
struct Root {
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

/// An Object representation of the [`ViewBox`] top left coordinate.
/// Primarily used to derive paths coordinates.
#[derive(Getters, Clone, Copy)]
#[getset(get = "pub")]
struct TopLeft {
    x: CoordinateT,
    y: CoordinateT,
}

/// Object representation of the generated SVGs.
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

/// This struct is provided as a result of requesting an [`SVG`] update based on a particular [`Configuration`].
/// Fields are kept private as no modification of this is ever expected. It's just a convenient wrapper for serialisation.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResult {
    style: String,
    information_group: String,
    view_box: String,
}

/// Error thrown when we can't get to the requested processing group.
/// Realistically, it should never happen, unless an invalid [`ManycoreSystem`] is provided.
/// However, the manycore_parser library should guard against this.
fn no_processing_group(index: usize) -> SVGError {
    SVGError::new(SVGErrorKind::GenerationError(format!("Could not retrieve SVG group for core with ID {}. Something weent wrong generating the SVG, please try again.", index)))
}

impl SVG {
    /// Creates a new [`SVG`] instance with the given parameters.
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

    /// Extends the [`SVG`]'s base and current viewBox left coordinate and adjusts width accordingly.
    fn extend_base_view_box_left(&mut self, left: CoordinateT) {
        self.view_box.extend_left(left);
        self.base_view_box.extend_left(left);
        self.width = self.width.saturating_add(left);
    }

    /// Extends the [`SVG`]'s base and current viewBox left height.
    fn extend_base_view_box_bottom(&mut self, bottom: CoordinateT) {
        self.view_box.extend_bottom(bottom);
        self.base_view_box.extend_bottom(bottom);
        self.height = self.height.saturating_add(bottom);
    }

    /// Generates an [`UpdateResult`] based on a provided [`Configuration`] and a reference [`ManycoreSystem`].
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
        self.root.information_group.groups_mut().clear();
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

        let mut offsets = Offsets::default();
        // Compute all requested attributes at information layer
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
                    .groups_mut()
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
            if *self.view_box.x() > *offsets.left() {
                updated_view_box
                    .extend_left(offsets.left().abs().saturating_sub(self.view_box.x().abs()));
            }

            let far_end = self
                .view_box
                .width()
                .saturating_sub(self.view_box.x().abs());
            if far_end < *offsets.right() {
                updated_view_box.extend_right(offsets.right().saturating_sub(far_end));
            }

            if *self.view_box.y() > *offsets.top() {
                updated_view_box
                    .extend_top(offsets.top().abs().saturating_sub(self.view_box.y().abs()));
            }

            let far_bottom = self
                .view_box
                .height()
                .saturating_sub(self.view_box.y().abs());
            if far_bottom < *offsets.bottom() {
                updated_view_box.extend_bottom(offsets.bottom().saturating_sub(far_bottom))
            }

            self.view_box.restore_from(&updated_view_box);
        }

        Ok(UpdateResult {
            style: self.style.css().clone(),
            information_group: self.root.information_group.update_string()?,
            view_box: String::from(&self.view_box),
        })
    }

    /// Adds a [`ClipPath`] to the [`SVG`]. Used in FreeForm exporting.
    pub fn add_clip_path(&mut self, polygon_points: String) {
        self.clip_path = Some(ClipPath::new(polygon_points));
        self.root.clip_path = Some(USE_CLIP_PATH);
    }

    /// Removes [`ClipPath`] from the [`SVG`].
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
