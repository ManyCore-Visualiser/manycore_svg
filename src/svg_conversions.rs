/// [`SVG`] conversion utilities.
use manycore_parser::{ManycoreSystem, WithID};
use quick_xml::DeError;
use std::cmp::min;

use crate::{
    CoordinateT, ProcessingGroup, SVGError, TopLeft, BLOCK_DISTANCE, BLOCK_LENGTH,
    CORE_ROUTER_STROKE_WIDTH, SVG, TASK_BOTTOM_OFFSET, TASK_RECT_STROKE,
};

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
        let mut ret = SVG::new(
            &manycore.cores().list().len(),
            rows,
            columns,
            width,
            height,
            top_left,
        );

        // Row tracker for iteration
        let mut r: u8 = 0;

        let cores = manycore.cores().list();
        let borders = manycore.borders();

        let mut min_task_start = None;
        let mut has_bottom_task = false;

        for (i, core) in cores.iter().enumerate() {
            // Realistically this conversion should never fail
            // Calculate current column from iteration index
            let c = u8::try_from(i % usize::try_from(columns).expect("8 bits must fit in a usize. I have no idea what you're trying to run this on, TI TMS 1000?")).expect(
                "Somehow, modulus on an 8 bit number gave a number that does not fit in 8 bits (your ALU re-invented mathematics).",
            );

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
                core.allocated_task(),
                &top_left,
            )?;

            // Check if viewBox needs to be extended left
            if c == 0 {
                if let Some(task_start) = processing_group.task_start() {
                    if let Some(min_task_start_value) = min_task_start {
                        // Get the minimum start, these are negative coordinates
                        min_task_start = Some(min(min_task_start_value, task_start));
                    } else {
                        min_task_start = Some(task_start);
                    }
                }
            }

            // Check if viewBox needs to be extended bottom
            if r == (rows - 1) {
                if let Some(_) = core.allocated_task() {
                    // We don't need an actual value as the offset is derived from the font size
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

                // Remember that index always corresponts to core ID (collections is sorted when converting manycore into SVG).
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
