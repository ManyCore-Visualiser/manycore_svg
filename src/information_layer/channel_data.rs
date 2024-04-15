use std::collections::{btree_map::Iter, BTreeMap, BTreeSet, HashMap};

use manycore_parser::{
    source::Source, BorderEntry, Channel, Core, Directions, SinkSourceDirection, WithID,
    WithXMLAttributes,
};

use crate::{
    Configuration, ConnectionType, ConnectionsParentGroup, CoordinateT, DirectionType,
    FieldConfiguration, InformationLayer, Offsets, RoutingConfiguration, SVGError, TextInformation,
};

use super::{get_connection_type, missing_channel, missing_connection, missing_source};

/// Utility to retrieve an SVG connection's coordinates and whether it is an edge or an inner connection.
fn channel_info_details<'a>(
    direction: &Directions,
    connections_group: &'a ConnectionsParentGroup,
    core: &manycore_parser::Core,
) -> Result<(&'a i32, &'a i32, bool), SVGError> {
    // We are looking for output only, source links are handled separately.
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

/// Generates an SVG channel secondary [`TextInformation`], if requested and present.
fn get_secondary_channel_attribute(
    x: &CoordinateT,
    y: &CoordinateT,
    edge: bool,
    configuration_iterator: &mut Iter<'_, String, FieldConfiguration>,
    channel: &Channel,
    direction: &Directions,
) -> Option<TextInformation> {
    if !edge {
        if let (Some((key, field_configuration)), Some(channel_attributes)) =
            (configuration_iterator.next(), channel.other_attributes())
        {
            return match channel_attributes.get(key) {
                Some(attribute_value) => {
                    let link_secondary_text = TextInformation::link_secondary(
                        direction,
                        x,
                        y,
                        attribute_value,
                        field_configuration,
                    );

                    Some(link_secondary_text)
                }
                None => {
                    // Not all attributes must be present on every channel I suppose.
                    // Do nothing if this channel does not have the requested one.
                    None
                }
            };
        }
    }

    None
}

pub(crate) fn generate_channel_data(
    rows: u8,
    columns: u8,
    configuration: &mut Configuration,
    core: &Core,
    border_routers_ids: Option<&HashMap<SinkSourceDirection, BorderEntry>>,
    sources: Option<&BTreeMap<u16, Source>>,
    core_loads: Option<&BTreeSet<Directions>>,
    connections_group: &ConnectionsParentGroup,
    routing_configuration: Option<&RoutingConfiguration>,
    offsets: &mut Offsets,
    ret: &mut InformationLayer,
) -> Result<(), SVGError> {
    // We use this set to keep track of directions we can add information to.
    let mut remaining_directions: BTreeSet<&Directions> =
        core.channels().channel().keys().collect();

    // Link loads
    if let (Some(directions), Some(routing_configuration)) = (core_loads, routing_configuration) {
        // For each direction in the current core_loads
        for direction in directions {
            // We explored this one, so we added all available information. Well, not yet, but we will soon.
            // We can remove from set
            remaining_directions.remove(direction);

            // Get channel details
            let (x, y, edge) = channel_info_details(direction, connections_group, core)?;

            // Grab the manycore_parser counterpart of the channel
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

            // This channel data might need the viewBox extended to be fully displayed.
            offsets.update(Offsets::try_from(&link_load_text)?);
            // Add the generated text to the result
            ret.links_load.push(link_load_text);

            // Additional parameter, if any, but only if this is not an edge connection.
            if let Some(link_secondary_text) = get_secondary_channel_attribute(
                x,
                y,
                edge,
                &mut configuration.channel_config().iter(),
                channel,
                direction,
            ) {
                // This channel data might need the viewBox extended to be fully displayed.
                offsets.update(Offsets::try_from(&link_secondary_text)?);
                ret.links_load.push(link_secondary_text);
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

        let (x, y, edge) = channel_info_details(direction, connections_group, core)?;

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
                            // This channel data might need the viewBox extended to be fully displayed.
                            offsets.update(Offsets::try_from(&link_text)?);
                            ret.links_load.push(link_text);
                        }
                        None => {
                            // Not all attributes must be present on every channel I suppose.
                            // Do nothing if this channel does not have the requested one.
                        }
                    }
                }
            }
        }

        // Second element, if any, but only if this is not an edge connection.
        if let Some(link_secondary_text) =
            get_secondary_channel_attribute(x, y, edge, &mut iter, channel, direction)
        {
            // This channel data might need the viewBox extended to be fully displayed.
            offsets.update(Offsets::try_from(&link_secondary_text)?);
            ret.links_load.push(link_secondary_text);
        }
    }

    // Source loads
    // Do we have the required data?
    if let (Some(sources), Some(border_routers_ids), Some(routing_configuration)) =
        (sources, border_routers_ids, routing_configuration)
    {
        // Is the core on the edge?
        if let Some(edge_position) = core.is_on_edge(columns, rows).as_ref() {
            let keys: Vec<SinkSourceDirection> = edge_position.into();

            // For each connection
            for source_direction in keys {
                // Is there a source here?
                if let Some(BorderEntry::Source(task_id)) =
                    border_routers_ids.get(&source_direction)
                {
                    let source = sources.get(&task_id).ok_or(missing_source(task_id))?;
                    let load = source.current_load();

                    let mut direction: Directions = (&source_direction).into();
                    let direction_type = DirectionType::Source(direction.clone());

                    // Get connection index
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

                        // Generate load text
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

    Ok(())
}
