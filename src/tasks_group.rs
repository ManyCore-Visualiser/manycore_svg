use std::{
    cmp::max,
    collections::BTreeMap,
    ops::{Add, Sub},
};

use getset::Getters;
use manycore_parser::ElementIDT;
use manycore_utils::serialise_btreemap;
use quick_xml::DeError;
use serde::Serialize;

use crate::{
    generation_error, partial_update::PartialUpdate, CoordinateT, FontSizeT,
    ProcessedBaseConfiguration, ProcessingGroup, SVGError, SVGErrorKind, TaskRectConfiguration,
    TextInformation, TopLeft, BLOCK_DISTANCE, BLOCK_LENGTH, CHAR_H_PADDING,
    CORE_ROUTER_STROKE_WIDTH_STR, HALF_CHAR_V_PADDING, ROUTER_OFFSET, SIDE_LENGTH,
};

pub(crate) const DEFAULT_TASK_FONT_SIZE: FontSizeT = 22.0;
pub(crate) static MINIMUM_TASK_FONT_SIZE: FontSizeT = 16.0;
pub(crate) static MAXIMUM_TASK_FONT_SIZE: FontSizeT = 32.0;
pub(crate) static TASK_RECT_STROKE: CoordinateT = 1;
static TASK_RECT_FILL: &'static str = "#bfdbfe";
static TASK_RECT_X_OFFSET: CoordinateT = 10;

/// Object representation of the SVG `<rect>` that wraps a task id.
#[derive(Serialize, Getters)]
pub(crate) struct TaskRect {
    #[serde(rename = "@x")]
    #[getset(get = "pub")]
    x: CoordinateT,
    #[serde(rename = "@y")]
    #[getset(get = "pub")]
    y: CoordinateT,
    #[serde(rename = "@width")]
    #[getset(get = "pub")]
    width: CoordinateT,
    #[serde(rename = "@height")]
    #[getset(get = "pub")]
    height: CoordinateT,
    #[serde(rename = "@rx")]
    rx: &'static str,
    #[serde(rename = "@fill")]
    fill: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl TaskRect {
    /// Generates a new [`TaskRect`] instance from the given parameters.
    fn new(
        centre_x: CoordinateT,
        centre_y: CoordinateT,
        text_width: CoordinateT,
        task_rect_configuration: &TaskRectConfiguration,
    ) -> Self {
        Self {
            x: centre_x - (text_width.saturating_div(2)),
            y: centre_y - *task_rect_configuration.task_rect_half_height(),
            width: text_width,
            height: *task_rect_configuration.task_rect_height(),
            rx: "10",
            fill: TASK_RECT_FILL,
            stroke: "black",
            stroke_width: CORE_ROUTER_STROKE_WIDTH_STR,
        }
    }

    /// Toggles [`TaskRect`] between cost inclusive and base variant depending on provided config.
    fn toggle_variant(
        &mut self,
        centre_x: CoordinateT,
        centre_y: CoordinateT,
        text_width: CoordinateT,
        task_rect_configuration: &TaskRectConfiguration,
    ) {
        self.x = centre_x - (text_width.saturating_div(2));
        self.y = centre_y - *task_rect_configuration.task_rect_half_height();
        self.width = text_width;
        self.height = *task_rect_configuration.task_rect_height();
    }
}

/// Struct to differentiate what state the [`Task`]s are currently in.
/// True -> Base state.
/// False -> With cost state.
/// I thought an enum with the actual states would not be very ergonomic for the operations performed.
struct BaseVariant(bool);

/// Helper struct to group [`TaskRect`] and its corresponding [`TextInformation`] together, forms the task bubble in the SVG.
#[derive(Serialize, Getters)]
pub(crate) struct Task {
    #[getset(get = "pub")]
    rect: TaskRect,
    #[serde(rename = "text")]
    task_text: TextInformation,
    #[serde(rename = "text", skip_serializing_if = "Option::is_none")]
    cost_text: Option<TextInformation>,
    #[serde(skip)]
    cost_string: String,
    #[serde(skip)]
    task_width: CoordinateT,
    #[serde(skip)]
    cost_width: CoordinateT,
}

impl Task {
    /// Calculates centre coordinates of a [`Task`] group by leveraging the provided approximate text width.
    fn get_centre_coordinates(
        row: &CoordinateT,
        column: &CoordinateT,
        text_width: CoordinateT,
        top_left: &TopLeft,
        task_rect_configuration: &TaskRectConfiguration,
    ) -> (CoordinateT, CoordinateT) {
        let cx = column * BLOCK_LENGTH + column * BLOCK_DISTANCE - (text_width.saturating_div(2))
            + TASK_RECT_X_OFFSET
            + top_left.x();
        let cy = task_rect_configuration
            .task_rect_centre_offset()
            .add(row.saturating_mul(BLOCK_LENGTH.add(BLOCK_DISTANCE)))
            .add(ROUTER_OFFSET)
            .add(SIDE_LENGTH)
            .add(top_left.y);

        (cx, cy)
    }

    /// Generates a new [`Task`] instance from the given parameters.
    pub(crate) fn new(
        row: &CoordinateT,
        column: &CoordinateT,
        task: &manycore_parser::Task,
        top_left: &TopLeft,
        processed_base_configuration: &ProcessedBaseConfiguration,
    ) -> Result<Self, SVGError> {
        let task_text = format!("T{}", task.id());
        let cost_text = format!("[{}]", task.computation_cost());

        // Get an approx text width
        let task_text_width = TextInformation::calculate_length_util(
            *processed_base_configuration.task_font_size(),
            task_text.len(),
            Some(CHAR_H_PADDING),
        )?;
        let cost_text_width = TextInformation::calculate_length_util(
            *processed_base_configuration.task_font_size(),
            cost_text.len(),
            Some(CHAR_H_PADDING),
        )?;

        // Get centre coordinates
        let (cx, cy) = Self::get_centre_coordinates(
            row,
            column,
            task_text_width,
            top_left,
            processed_base_configuration.task_rect(),
        );

        Ok(Self {
            rect: TaskRect::new(
                cx,
                cy,
                task_text_width,
                processed_base_configuration.task_rect(),
            ),
            task_text: TextInformation::new(
                cx,
                cy,
                *processed_base_configuration.task_font_size(),
                "middle",
                "central",
                None,
                None,
                task_text,
            ),
            cost_text: None,
            // These three variables are computed here so we don't have to do it whenever the user requests the change.
            // They are relatively cheap to store. Realistically, number of tasks should be low compared to overall memory
            // footprint so the string should be okay to keep.
            // The widths are just 4 bytes currently (strings are unlikely to be significantly larger).
            cost_string: cost_text,
            task_width: task_text_width,
            cost_width: cost_text_width,
        })
    }

    /// Transforms a [`Task`] into the cost inclusive variant.
    fn make_with_cost(
        &mut self,
        processing_group: &ProcessingGroup,
        processed_base_configuration: &ProcessedBaseConfiguration,
        top_left: &TopLeft,
    ) {
        let (row, column) = processing_group.coordinates();
        let text_width = max(self.task_width, self.cost_width);

        // Get centre coordinates
        let (cx, cy) = Self::get_centre_coordinates(
            row,
            column,
            text_width,
            top_left,
            processed_base_configuration.task_rect(),
        );

        self.rect.toggle_variant(
            cx,
            cy,
            text_width,
            processed_base_configuration.task_rect_with_cost(),
        );

        self.task_text.set_x(cx);
        self.task_text.set_y(
            cy.sub(processed_base_configuration.task_half_font_size_coord())
                .sub(HALF_CHAR_V_PADDING),
        );

        self.cost_text = Some(TextInformation::new(
            cx,
            cy.add(processed_base_configuration.task_half_font_size_coord())
                .add(HALF_CHAR_V_PADDING),
            *processed_base_configuration.task_font_size(),
            "middle",
            "central",
            None,
            None,
            self.cost_string.clone(),
        ));
    }

    /// Transforms a [`Task`] into the base variant.
    fn make_base(
        &mut self,
        processing_group: &ProcessingGroup,
        processed_base_configuration: &ProcessedBaseConfiguration,
        top_left: &TopLeft,
    ) {
        let (row, column) = processing_group.coordinates();

        // Get centre coordinates
        let (cx, cy) = Self::get_centre_coordinates(
            row,
            column,
            self.task_width,
            top_left,
            processed_base_configuration.task_rect(),
        );

        self.rect.toggle_variant(
            cx,
            cy,
            self.task_width,
            processed_base_configuration.task_rect(),
        );

        self.task_text.set_x(cx);
        self.task_text.set_y(cy);
        self.cost_text = None;
    }
}

#[derive(Serialize)]
pub(crate) struct TasksGroup {
    #[serde(rename = "g", serialize_with = "serialise_btreemap")]
    tasks: BTreeMap<u16, Task>,
    #[serde(skip)]
    variant: BaseVariant,
}

impl TasksGroup {
    /// Creates a new [`TasksGroup`] instance with enough capacity for the provided number of tasks.
    pub(crate) fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            variant: BaseVariant(true),
        }
    }

    /// Creates a new [`Task`] and inserts it in the group from the provided [`manycore_parser::Task`]
    pub(crate) fn add_task(
        &mut self,
        row: &CoordinateT,
        column: &CoordinateT,
        task: &manycore_parser::Task,
        top_left: &TopLeft,
        processed_base_configuration: &ProcessedBaseConfiguration,
    ) -> Result<&Task, SVGError> {
        let task_g = Task::new(row, column, task, top_left, processed_base_configuration)?;

        Ok(self.tasks.entry(*task.id()).or_insert(task_g))
    }

    /// Returns whether the [`TasksGroup`] is in the base variant.
    pub(crate) fn is_base(&self) -> bool {
        self.variant.0
    }

    /// Toggles the [`TasksGroup`] variant.
    pub(crate) fn toggle_variant(&mut self) {
        self.variant.0 = !self.variant.0;
    }

    /// Toggles the requested task.
    pub(crate) fn toggle_task(
        &mut self,
        task_id: &u16,
        processing_group: &ProcessingGroup,
        processed_base_configuration: &ProcessedBaseConfiguration,
        top_left: &TopLeft,
    ) -> Result<&Task, SVGError> {
        let is_base = self.is_base();

        let task = self.tasks.get_mut(task_id).ok_or_else(|| {
            generation_error(format!("Could not find Task {task_id} in TasksGroup."))
        })?;

        if is_base {
            task.make_with_cost(processing_group, processed_base_configuration, top_left);
        } else {
            task.make_base(processing_group, processed_base_configuration, top_left);
        }

        Ok(task)
    }
}

impl PartialUpdate for TasksGroup {
    fn update_string(&self) -> Result<String, DeError> {
        let tasks: Vec<&Task> = self.tasks.values().collect();
        let tasks = quick_xml::se::to_string_with_root("g", &tasks)?;

        Ok(tasks)
    }
}

/// Utility to generate an error when a task is not found in the task graph.
pub(crate) fn missing_task(core_id: &ElementIDT, task_id: &u16) -> SVGError {
    SVGError::new(SVGErrorKind::ManycoreMismatch(format!(
        "Core {core_id} has Task {task_id} allocated but the task is not in the TaskGraph."
    )))
}
