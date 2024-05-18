/// [`SVG`] conversion utilities.
use manycore_parser::{ManycoreSystem, SystemDimensionsT, WithID};
use quick_xml::DeError;
use serde::Serialize;
use std::cmp::min;

use crate::{
    tasks_group::{missing_task, TASK_RECT_STROKE},
    BaseConfiguration, CoordinateT, Offsets, ProcessingGroup, SVGError, TopLeft, ViewBox,
    BLOCK_DISTANCE, BLOCK_LENGTH, CORE_ROUTER_STROKE_WIDTH, SVG, UNSUPPORTED_PLATFORM,
};

impl TryFrom<&SVG> for String {
    type Error = DeError;

    fn try_from(svg: &SVG) -> Result<Self, Self::Error> {
        let mut buf = String::new();
        let mut serialiser = quick_xml::se::Serializer::new(&mut buf);
        serialiser.indent(' ', 4);
        serialiser.set_quote_level(quick_xml::se::QuoteLevel::Minimal);

        svg.serialize(serialiser)?;

        Ok(buf)
    }
}

impl SVG {
    fn shared_try_from(
        manycore: &ManycoreSystem,
        base_configuration: BaseConfiguration,
    ) -> Result<Self, SVGError> {
        let columns = *manycore.columns();
        let rows = *manycore.rows();

        let columns_coord: CoordinateT = columns.into();
        let rows_coord: CoordinateT = rows.into();

        // Each column * each block + the distance between blocks + the stroke
        let width = (columns_coord * BLOCK_LENGTH)
            + ((columns_coord - 1) * BLOCK_DISTANCE)
            + CORE_ROUTER_STROKE_WIDTH.saturating_mul(2);
        // Each row * each block + the distance between blocks + the stroke
        let height = (rows_coord * BLOCK_LENGTH)
            + ((rows_coord - 1) * BLOCK_DISTANCE)
            + CORE_ROUTER_STROKE_WIDTH.saturating_mul(2);

        // viewBox top left
        let top_left = TopLeft {
            x: width.saturating_div(2).saturating_mul(-1),
            y: height.saturating_div(2).saturating_mul(-1),
        };

        // The SVG we'll return
        let mut ret = SVG::new(manycore, width, height, top_left, base_configuration);

        // Row tracker for iteration
        let mut r: SystemDimensionsT = 0;

        let cores = manycore.cores().list();
        let borders = manycore.borders();

        let mut min_task_start = None;
        let mut has_bottom_task = false;

        let mut borders_offsets = Offsets::new(0, 0, 0, 0);

        for (i, core) in cores.iter().enumerate() {
            // Realistically this conversion should never fail
            // Calculate current column from iteration index
            let c = SystemDimensionsT::try_from(
                i % usize::try_from(columns).expect(UNSUPPORTED_PLATFORM),
            )?;

            // Increment row when we wrap onto a new row
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
                &ret.top_left,
                ret.defs.clip_paths_mut(),
            )?;

            // Add task
            if let Some(task_id) = core.allocated_task() {
                let allocated_task = manycore
                    .task_graph()
                    .tasks()
                    .get(task_id)
                    .ok_or_else(|| missing_task(core.id(), task_id))?;

                let task = ret.root.tasks_group.add_task(
                    &r_coord,
                    &c_coord,
                    allocated_task,
                    &top_left,
                    &ret.processed_base_configuration,
                )?;

                // Check if viewBox needs to be extended left
                if c == 0 {
                    if let Some(min_task_start_value) = min_task_start {
                        // Get the minimum start, these are negative coordinates
                        min_task_start = Some(min(min_task_start_value, *task.rect().x()));
                    } else {
                        min_task_start = Some(*task.rect().x());
                    }
                }

                // Check if viewBox needs to be extended bottom
                if r == (rows - 1) {
                    has_bottom_task = true;
                }
            }

            // Generate connections group
            ret.root
                .connections_group
                .add_connections(core, &r_coord, &c_coord, &ret.top_left);

            // Generate borders
            if let Some(edge_position) = core.matrix_edge() {
                let (router_x, router_y) = processing_group.router().move_coordinates();

                // Remember that index always corresponts to core ID (collections is sorted when converting manycore into SVG).
                ret.root.sinks_sources_group.insert(
                    edge_position,
                    router_x,
                    router_y,
                    match borders {
                        Some(borders) => borders.core_border_map().get(&i),
                        None => None,
                    },
                    &ret.processed_base_configuration,
                    &mut borders_offsets,
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
            ret.extend_base_view_box_bottom(
                *ret.processed_base_configuration
                    .task_rect()
                    .task_rect_bottom_padding(),
            );
        }

        // Calculate borders viewBox.
        let borders_view_box = ViewBox::from(&borders_offsets);
        ret.borders_view_box = borders_view_box;

        Ok(ret)
    }

    pub(crate) fn try_from_manycore_with_base_config(
        manycore: &ManycoreSystem,
        base_configuration: &BaseConfiguration,
    ) -> Result<Self, SVGError> {
        Ok(SVG::shared_try_from(manycore, *base_configuration)?)
    }
}

impl TryFrom<&ManycoreSystem> for SVG {
    type Error = SVGError;
    fn try_from(manycore: &ManycoreSystem) -> Result<Self, Self::Error> {
        Ok(SVG::shared_try_from(
            manycore,
            BaseConfiguration::default(),
        )?)
    }
}
