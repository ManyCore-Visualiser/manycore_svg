use const_format::concatcp;
use manycore_parser::RoutingMap;
use serde::Serialize;

use crate::{
    text_background::TEXT_BACKGROUND_ID, Configuration, ConnectionsParentGroup, CoordinateT,
    Offsets, ProcessedBaseConfiguration, ProcessingGroup, RoutingConfiguration, SVGError,
    ROUTER_OFFSET, SIDE_LENGTH,
};

static OFFSET_FROM_BORDER: CoordinateT = 1;
static TEXT_GROUP_FILTER: &str = concatcp!("url(#", TEXT_BACKGROUND_ID, ")");

// Example after concatenation with SIDE_LENGTH = 100 -> ROUTER_OFFSET = 75
// path('m0,0 l0,100 l98,0 l0,-75 l-25,-25 l-75,0 Z')
static PROCESSOR_CLIP: &'static str = concatcp!(
    "path('m0,0 l0,",
    SIDE_LENGTH,
    " l",
    SIDE_LENGTH - 2,
    ",0 l0,-",
    ROUTER_OFFSET,
    " l-",
    SIDE_LENGTH - ROUTER_OFFSET,
    ",-",
    SIDE_LENGTH - ROUTER_OFFSET,
    " l-",
    ROUTER_OFFSET,
    ",0 Z')"
);

// Example after concatenation with SIDE_LENGTH = 100 -> ROUTER_OFFSET = 75
// path('m0,0 l0,74 l25,25 l73,0 l0,-100 Z')
static ROUTER_CLIP: &'static str = concatcp!(
    "path('m0,0 l0,",
    ROUTER_OFFSET - 1,
    " l",
    SIDE_LENGTH - ROUTER_OFFSET,
    ",",
    SIDE_LENGTH - ROUTER_OFFSET,
    " l",
    ROUTER_OFFSET - 2,
    ",0 l0,-",
    SIDE_LENGTH,
    " Z')"
);

/// Core or Router information SVG `<g>` wrapper.
#[derive(Serialize, Default)]
struct ProcessingInformation {
    #[serde(rename = "@filter", skip_serializing_if = "Option::is_none")]
    filter: Option<&'static str>,
    #[serde(rename = "@clip-path")]
    clip_path: &'static str,
    #[serde(rename = "text")]
    information: Vec<TextInformation>,
}

/// Object representation for an SVG `<g>` that wraps user configurable information for each core-derived group.
/// core-derived groups include channels and routers as all the calculations to generate those groups rely on
/// information provided by the [`manycore_parser::Core`] object.
#[derive(Serialize, Default)]
#[serde(rename = "g")]
pub(crate) struct InformationLayer {
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
pub use text_information::*;
mod coordinates;
use coordinates::make_coordinates;
mod channel_data;
use channel_data::*;

impl InformationLayer {
    /// Generates a new [`InformationLayer`] instance.
    pub(crate) fn new(
        rows: u8,
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
        );

        // Core
        generate_with_id(
            *core_x,
            *core_y,
            configuration.core_config(),
            core,
            &mut ret.core_group,
            "start",
            css,
            processed_base_configuration,
        );
        ret.core_group.clip_path = PROCESSOR_CLIP;

        // Router
        let (router_x, router_y) = processing_group.router().move_coordinates();
        generate_with_id(
            *router_x,
            router_y - ROUTER_OFFSET,
            configuration.router_config(),
            core.router(),
            &mut ret.router_group,
            "start",
            css,
            processed_base_configuration,
        );
        ret.router_group.clip_path = ROUTER_CLIP;

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
