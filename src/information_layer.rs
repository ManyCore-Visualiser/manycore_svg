use manycore_parser::{RoutingMap, SystemDimensionsT, WithID};
use serde::Serialize;

use crate::{
    ClipPath, Configuration, ConnectionsParentGroup, CoordinateT, Offsets,
    ProcessedBaseConfiguration, ProcessingGroup, RoutingConfiguration, SVGError, ROUTER_OFFSET,
    USE_FREEFORM_CLIP_PATH,
};

static OFFSET_FROM_BORDER: CoordinateT = 1;

/// Core or Router information SVG `<g>` wrapper.
#[derive(Serialize, Default)]
struct ProcessingInformation {
    #[serde(rename = "@clip-path")]
    clip_path: String,
    #[serde(rename = "text")]
    information: Vec<TextInformation>,
}

/// Object representation for an SVG `<g>` that wraps user configurable information for each core-derived group.
/// core-derived groups include channels and routers as all the calculations to generate those groups rely on
/// information provided by the [`manycore_parser::Core`] object.
#[derive(Serialize, Default)]
#[serde(rename = "g")]
pub(crate) struct InformationLayer {
    #[serde(rename = "@clip-path")]
    clip_path: &'static str,
    #[serde(rename = "g")]
    core_group: ProcessingInformation,
    #[serde(rename = "g")]
    router_group: ProcessingInformation,
    #[serde(rename = "text", skip_serializing_if = "Option::is_none")]
    coordinates: Option<TextInformation>,
    #[serde(rename = "text", skip_serializing_if = "Vec::is_empty")]
    links_load: Vec<TextInformation>,
}

mod utils;
use utils::*;
mod text_information;
pub(crate) use text_information::*;
mod coordinates;
use coordinates::make_coordinates;
mod channel_data;
use channel_data::*;

impl InformationLayer {
    /// Generates a new [`InformationLayer`] instance.
    pub(crate) fn new(
        rows: SystemDimensionsT,
        configuration: &mut Configuration,
        core: &manycore_parser::Core,
        links_with_load: Option<&RoutingMap>,
        css: &mut String,
        processing_group: &ProcessingGroup,
        connections_group: &ConnectionsParentGroup,
        routing_configuration: Option<&RoutingConfiguration>,
        offsets: &mut Offsets,
        processed_base_configuration: &ProcessedBaseConfiguration,
    ) -> Result<Self, SVGError> {
        let mut ret = InformationLayer::default();
        ret.clip_path = USE_FREEFORM_CLIP_PATH;

        let (r, c) = processing_group.coordinates();
        let (core_x, core_y) = processing_group.core().move_coordinates();

        // Coordinates are stored in the core config but apply to whole group
        make_coordinates(
            configuration.core_config(),
            core_x,
            core_y,
            rows,
            r,
            c,
            &mut ret,
            processed_base_configuration,
        )?;

        // Core
        generate_with_id(
            *core_x,
            *core_y,
            configuration.core_config(),
            configuration.core_fills(),
            core,
            &mut ret.core_group,
            "start",
            css,
            processed_base_configuration,
        )?;
        // Clip path id
        ret.core_group.clip_path = format!("url(#{})", ClipPath::make_core_id(core.id()));

        // Router
        let (router_x, router_y) = processing_group.router().move_coordinates();
        generate_with_id(
            *router_x,
            router_y - ROUTER_OFFSET,
            configuration.router_config(),
            configuration.router_fills(),
            core.router(),
            &mut ret.router_group,
            "start",
            css,
            processed_base_configuration,
        )?;
        // Clip path id
        ret.router_group.clip_path =
            format!("url(#{})", ClipPath::make_router_id(core.router().id()));

        // Channels
        generate_channel_data(
            configuration,
            core,
            links_with_load,
            connections_group,
            routing_configuration,
            offsets,
            &mut ret,
            processed_base_configuration,
        )?;

        Ok(ret)
    }
}
