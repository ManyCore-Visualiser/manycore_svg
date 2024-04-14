use std::collections::{BTreeMap, BTreeSet, HashMap};

use const_format::concatcp;
use manycore_parser::{
    source::Source, BorderEntry, Directions, SinkSourceDirection, WithID, WithXMLAttributes,
};
use serde::Serialize;

use crate::{
    text_background::TEXT_BACKGROUND_ID, Configuration, ConnectionType, ConnectionsParentGroup,
    CoordinateT, DirectionType, Offsets, ProcessingGroup, RoutingConfiguration, SVGError,
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

#[derive(Serialize, Default)]
struct ProcessingInformation {
    #[serde(rename = "@filter", skip_serializing_if = "Option::is_none")]
    filter: Option<&'static str>,
    #[serde(rename = "@clip-path")]
    clip_path: &'static str,
    #[serde(rename = "text")]
    information: Vec<TextInformation>,
}

#[derive(Serialize, Default)]
#[serde(rename = "g")]
pub struct InformationLayer {
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

impl InformationLayer {
    fn link_info_details<'a>(
        direction: &Directions,
        connections_group: &'a ConnectionsParentGroup,
        core: &manycore_parser::Core,
    ) -> Result<(&'a i32, &'a i32, bool), SVGError> {
        let direction_type = DirectionType::Out(*direction);

        let connection_type = get_connection_type(connections_group, &direction_type, core.id())?;

        match connection_type {
            ConnectionType::Connection(idx) => {
                let connection = connections_group
                    .connections()
                    .path()
                    .get(*idx)
                    .ok_or(missing_connection(idx))?;

                Ok((connection.x(), connection.y(), false))
            }
            ConnectionType::EdgeConnection(idx) => {
                let connection = connections_group
                    .edge_connections()
                    .sink()
                    .get(*idx)
                    .ok_or(missing_connection(idx))?;

                Ok((connection.x(), connection.y(), true))
            }
        }
    }

    pub fn new(
        rows: u8,
        columns: u8,
        configuration: &Configuration,
        core: &manycore_parser::Core,
        border_routers_ids: Option<&HashMap<SinkSourceDirection, BorderEntry>>,
        sources: Option<&BTreeMap<u16, Source>>,
        css: &mut String,
        core_loads: Option<&BTreeSet<Directions>>,
        processing_group: &ProcessingGroup,
        connections_group: &ConnectionsParentGroup,
        routing_configuration: Option<&RoutingConfiguration>,
        offsets: &mut Offsets,
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
        );
        ret.router_group.clip_path = ROUTER_CLIP;

        let mut remaining_directions: BTreeSet<&Directions> =
            core.channels().channel().keys().collect();

        // Link loads
        if let (Some(directions), Some(routing_configuration)) = (core_loads, routing_configuration)
        {
            for direction in directions {
                remaining_directions.remove(direction);

                let (x, y, edge) =
                    InformationLayer::link_info_details(direction, connections_group, core)?;

                let channel = core
                    .channels()
                    .channel()
                    .get(direction)
                    .ok_or(missing_channel(core.id(), &direction))?;

                let load = channel.current_load();

                let bandwidth = channel.bandwidth();

                let link_load_text = TextInformation::link_load(
                    direction,
                    x,
                    y,
                    load,
                    bandwidth,
                    edge,
                    routing_configuration,
                );
                offsets.update(Offsets::try_from(&link_load_text)?);
                ret.links_load.push(link_load_text);

                if !edge {
                    // Additional parameter, if any
                    if let (Some((key, field_configuration)), Some(channel_attributes)) = (
                        configuration.channel_config().iter().next(),
                        channel.other_attributes(),
                    ) {
                        match channel_attributes.get(key) {
                            Some(attribute_value) => {
                                let link_secondary_text = TextInformation::link_secondary(
                                    direction,
                                    x,
                                    y,
                                    attribute_value,
                                    field_configuration,
                                );
                                offsets.update(Offsets::try_from(&link_secondary_text)?);
                                ret.links_load.push(link_secondary_text);
                            }
                            None => {
                                panic!("Missing attribute requested")
                            }
                        }
                    }
                }
            }
        }

        // Render additional parameter(s) if requested for non-routed directions
        for direction in remaining_directions {
            let channel = core
                .channels()
                .channel()
                .get(direction)
                .ok_or(missing_channel(core.id(), &direction))?;

            let mut iter = configuration.channel_config().iter();

            let (x, y, edge) =
                InformationLayer::link_info_details(direction, connections_group, core)?;

            if !edge {
                if let Some(channel_attributes) = channel.other_attributes() {
                    // First element
                    if let Some((key, field_configuration)) = iter.next() {
                        match channel_attributes.get(key) {
                            Some(attribute_value) => {
                                let link_text = TextInformation::link_primary(
                                    direction,
                                    x,
                                    y,
                                    attribute_value,
                                    false,
                                    field_configuration,
                                );
                                offsets.update(Offsets::try_from(&link_text)?);
                                ret.links_load.push(link_text);
                            }
                            None => {
                                panic!("Missing attribute requested")
                            }
                        }
                    }

                    // Second element
                    if let Some((key, field_configuration)) = iter.next() {
                        match channel_attributes.get(key) {
                            Some(attribute_value) => {
                                let link_text = TextInformation::link_secondary(
                                    direction,
                                    x,
                                    y,
                                    attribute_value,
                                    field_configuration,
                                );
                                offsets.update(Offsets::try_from(&link_text)?);
                                ret.links_load.push(link_text);
                            }
                            None => {
                                panic!("Missing attribute requested")
                            }
                        }
                    }
                }
            }
        }

        // Source loads
        if let (Some(sources), Some(border_routers_ids), Some(routing_configuration)) =
            (sources, border_routers_ids, routing_configuration)
        {
            if let Some(edge_position) = core.is_on_edge(columns, rows).as_ref() {
                let keys: Vec<SinkSourceDirection> = edge_position.into();

                for source_direction in keys {
                    if let Some(BorderEntry::Source(task_id)) =
                        border_routers_ids.get(&source_direction)
                    {
                        let source = sources.get(&task_id).ok_or(missing_source(task_id))?;
                        let load = source.current_load();

                        let mut direction: Directions = (&source_direction).into();
                        let direction_type = DirectionType::Source(direction.clone());
                        let connection_type =
                            get_connection_type(connections_group, &direction_type, core.id())?;

                        if let ConnectionType::EdgeConnection(idx) = connection_type {
                            let connection = connections_group
                                .edge_connections()
                                .source()
                                .get(*idx)
                                .ok_or(missing_connection(idx))?;

                            let channel = core
                                .channels()
                                .channel()
                                .get(&direction)
                                .ok_or(missing_channel(core.id(), &direction))?;

                            // Flip direction, source notation is inverted
                            direction = match direction {
                                Directions::North => Directions::South,
                                Directions::South => Directions::North,
                                Directions::East => Directions::West,
                                Directions::West => Directions::East,
                            };

                            ret.links_load.push(TextInformation::source_load(
                                &direction,
                                connection.x(),
                                connection.y(),
                                &load,
                                channel.bandwidth(),
                                routing_configuration,
                            ));
                        } else {
                            panic!("Not supposed to be this");
                        }
                    }
                }
            } else {
                panic!("Core must be on edge for source ids")
            }
        }

        Ok(ret)
    }
}
