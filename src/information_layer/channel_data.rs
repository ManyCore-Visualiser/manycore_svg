use std::collections::{btree_map::Iter, BTreeSet};

use manycore_parser::{
    Channel, Core, Directions, RoutingMap, RoutingType, WithID, WithXMLAttributes,
};

use crate::{
    Configuration, ConnectionType, ConnectionsParentGroup, CoordinateT, DirectionType,
    FieldConfiguration, InformationLayer, Offsets, ProcessedBaseConfiguration,
    RoutingConfiguration, SVGError, TextInformation,
};

use super::{
    get_connection_type, missing_channel, missing_connection, missing_source_load,
    missing_source_loads,
};

/// Utility to retrieve an SVG connection's coordinates and whether it is an edge or an inner connection.
fn channel_info_details<'a>(
    direction: &Directions,
    connections_group: &'a ConnectionsParentGroup,
    core: &manycore_parser::Core,
    routing_type: &RoutingType,
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
            let connection = match routing_type {
                // Get sink connection
                RoutingType::OutputChannel => connections_group
                    .edge_connections()
                    .sink()
                    .get(*idx)
                    .ok_or(missing_connection(idx))?,
                // Get source connection
                RoutingType::SourceChannel => connections_group
                    .edge_connections()
                    .source()
                    .get(*idx)
                    .ok_or(missing_connection(idx))?,
            };

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
    prrocessed_base_configuration: &ProcessedBaseConfiguration,
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
                        prrocessed_base_configuration,
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
    configuration: &mut Configuration,
    core: &Core,
    links_with_load: Option<&RoutingMap>,
    connections_group: &ConnectionsParentGroup,
    routing_configuration: Option<&RoutingConfiguration>,
    offsets: &mut Offsets,
    ret: &mut InformationLayer,
    processed_base_configuration: &ProcessedBaseConfiguration,
) -> Result<(), SVGError> {
    // We use this set to keep track of directions we can add information to.
    let mut remaining_directions: BTreeSet<&Directions> =
        core.channels().channel().keys().collect();

    if let (Some(links_with_load), Some(routing_configuration)) =
        (links_with_load, routing_configuration)
    {
        if let Some(routed_channels) = links_with_load.get(core.id()) {
            for (target, directions) in routed_channels {
                for direction in directions {
                    // We explored this one, so we added all available information. Well, not yet, but we will soon.
                    // We can remove from set
                    remaining_directions.remove(direction);

                    // Get channel details
                    let (x, y, edge) =
                        channel_info_details(direction, connections_group, core, target)?;

                    // Grab the manycore_parser counterpart of the channel
                    let channel = core
                        .channels()
                        .channel()
                        .get(direction)
                        .ok_or(missing_channel(core.id(), &direction))?;

                    // Generate load text
                    let link_load_text = match target {
                        RoutingType::OutputChannel => TextInformation::link_load(
                            direction,
                            x,
                            y,
                            channel.current_load(),
                            channel.bandwidth(),
                            edge,
                            routing_configuration,
                            processed_base_configuration,
                        ),
                        RoutingType::SourceChannel => {
                            // Grab load from core source channels
                            let load = core
                                .source_loads()
                                .as_ref()
                                .ok_or(missing_source_loads(core.id()))?
                                .get(direction)
                                .ok_or(missing_source_load(core.id(), direction))?;

                            // Flip direction. The rendering logic assumes direction from the source
                            // point of view, not the core's.
                            let flipped_direction = match direction {
                                Directions::North => Directions::South,
                                Directions::South => Directions::North,
                                Directions::East => Directions::West,
                                Directions::West => Directions::East,
                            };

                            TextInformation::source_load(
                                &flipped_direction,
                                x,
                                y,
                                load,
                                channel.bandwidth(),
                                routing_configuration,
                                processed_base_configuration,
                            )
                        }
                    };

                    // This channel data might need the viewBox extended to be fully displayed.
                    offsets.update(Offsets::try_from_channel(&link_load_text, direction)?);
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
                        processed_base_configuration,
                    ) {
                        // This channel data might need the viewBox extended to be fully displayed.
                        offsets.update(Offsets::try_from_channel(&link_secondary_text, direction)?);
                        ret.links_load.push(link_secondary_text);
                    }
                }
            }
        }
    }

    let edge_directions = match core.matrix_edge() {
        Some(edge_position) => BTreeSet::from(edge_position),
        None => BTreeSet::new(),
    };

    // Render additional parameter(s) if requested for non-routed directions, ignoring
    // edge connections, if any. Note that direction has type &&Direrctions but get coerced
    // into &Directions.
    // See https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/deref-coercions.html#deref-and-method-calls
    for direction in remaining_directions.difference(&edge_directions) {
        let channel = core
            .channels()
            .channel()
            .get(direction)
            .ok_or(missing_channel(core.id(), direction))?;

        let mut iter = configuration.channel_config().iter();

        let (x, y, _) = channel_info_details(
            direction,
            connections_group,
            core,
            &RoutingType::OutputChannel,
        )?;

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
                            // Edge is always false, we are not walking over any edge direction.
                            // Set difference iterator removes them.
                            false,
                            field_configuration,
                            processed_base_configuration,
                        );
                        // This channel data might need the viewBox extended to be fully displayed.
                        offsets.update(Offsets::try_from_channel(&link_text, direction)?);
                        ret.links_load.push(link_text);
                    }
                    None => {
                        // Not all attributes must be present on every channel I suppose.
                        // Do nothing if this channel does not have the requested one.
                    }
                }
            }
        }

        // Second element, if any.
        if let Some(link_secondary_text) = get_secondary_channel_attribute(
            x,
            y,
            // Edge is always false, we are not walking over any edge direction.
            // Set difference iterator removes them.
            false,
            &mut iter,
            channel,
            direction,
            processed_base_configuration,
        ) {
            // This channel data might need the viewBox extended to be fully displayed.
            offsets.update(Offsets::try_from_channel(&link_secondary_text, direction)?);
            ret.links_load.push(link_secondary_text);
        }
    }

    Ok(())
}
