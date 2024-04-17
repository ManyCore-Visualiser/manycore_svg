use const_format::concatcp;
use getset::{Getters, MutGetters, Setters};
use serde::Serialize;

use crate::{
    style::BASE_FILL_CLASS_NAME, CoordinateT, FontSizeT, ProcessedBaseConfiguration, SVGError,
    TextInformation, TopLeft, CHAR_H_PADDING, CONNECTION_LENGTH, MARKER_HEIGHT,
};

pub(crate) const SIDE_LENGTH: CoordinateT = 100;
pub(crate) const ROUTER_OFFSET: CoordinateT = SIDE_LENGTH.saturating_div(4).saturating_mul(3);
pub(crate) static BLOCK_LENGTH: CoordinateT = SIDE_LENGTH + ROUTER_OFFSET;
pub(crate) static HALF_SIDE_LENGTH: CoordinateT = SIDE_LENGTH.saturating_div(2);
pub(crate) static HALF_ROUTER_OFFSET: CoordinateT = ROUTER_OFFSET.saturating_div(2);
pub(crate) static BLOCK_DISTANCE: CoordinateT = CONNECTION_LENGTH
    .saturating_sub(ROUTER_OFFSET)
    .saturating_add(MARKER_HEIGHT);
static TASK_RECT_X_OFFSET: CoordinateT = 10;

// Example after concatenation with SIDE_LENGTH = 100 -> ROUTER_OFFSET = 75
// l0,100 l100,0 l0,-75 l-25,-25 l-75,0 Z
static PROCESSOR_PATH: &'static str = concatcp!(
    "l0,",
    SIDE_LENGTH,
    " l",
    SIDE_LENGTH,
    ",0 l0,-",
    ROUTER_OFFSET,
    " l-",
    SIDE_LENGTH - ROUTER_OFFSET,
    ",-",
    SIDE_LENGTH - ROUTER_OFFSET,
    " l-",
    ROUTER_OFFSET,
    ",0 Z"
);

// Example after concatenation with SIDE_LENGTH = 100 -> ROUTER_OFFSET = 75
// l0,-75 l100,0 l0,100 l-75,0 Z
static ROUTER_PATH: &'static str = concatcp!(
    "l0,-",
    ROUTER_OFFSET,
    " l",
    SIDE_LENGTH,
    ",0 l0,",
    SIDE_LENGTH,
    " l-",
    ROUTER_OFFSET,
    ",0 Z"
);

pub(crate) const CORE_ROUTER_STROKE_WIDTH: CoordinateT = 1;
static CORE_ROUTER_STROKE_WIDTH_STR: &'static str = concatcp!(CORE_ROUTER_STROKE_WIDTH);

pub(crate) const DEFAULT_TASK_FONT_SIZE: FontSizeT = 22.0;
pub(crate) static TASK_RECT_STROKE: CoordinateT = 1;
// static TASK_RECT_HEIGHT: CoordinateT = CHAR_HEIGHT_AT_22_PX + CHAR_V_PADDING * 2;
// pub(crate) static HALF_TASK_RECT_HEIGHT: CoordinateT = TASK_RECT_HEIGHT.saturating_div(2);
// pub(crate) static TASK_RECT_CENTRE_OFFSET: CoordinateT = 10;
static TASK_RECT_FILL: &'static str = "#bfdbfe";
// pub(crate) static TASK_BOTTOM_OFFSET: CoordinateT = TASK_RECT_CENTRE_OFFSET
//     .saturating_add(HALF_TASK_RECT_HEIGHT)
//     .saturating_add(TASK_RECT_STROKE);

/// Wrapper around attributes shared by different elements.
#[derive(Serialize, Setters, Debug)]
pub(crate) struct CommonAttributes {
    #[serde(rename = "@class", skip_serializing_if = "Option::is_none")]
    class: Option<&'static str>,
    #[serde(rename = "@fill-rule")]
    fill_rule: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-linecap")]
    stroke_linecap: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl Default for CommonAttributes {
    fn default() -> Self {
        Self {
            class: Some(BASE_FILL_CLASS_NAME),
            fill_rule: "evenodd",
            stroke: "black",
            stroke_linecap: "butt",
            stroke_width: CORE_ROUTER_STROKE_WIDTH_STR,
        }
    }
}

impl CommonAttributes {
    /// Generates  a [`CommonAttributes`] instance with no class.
    pub(crate) fn with_no_class() -> Self {
        Self {
            class: None,
            fill_rule: "evenodd",
            stroke: "black",
            stroke_linecap: "butt",
            stroke_width: CORE_ROUTER_STROKE_WIDTH_STR,
        }
    }
}

/// Object representation of the SVG `<rect>` that wraps a task id.
#[derive(Serialize)]
struct TaskRect {
    #[serde(rename = "@x")]
    x: CoordinateT,
    #[serde(rename = "@y")]
    y: CoordinateT,
    #[serde(rename = "@width")]
    width: CoordinateT,
    #[serde(rename = "@height")]
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
        processed_base_configuration: &ProcessedBaseConfiguration,
    ) -> Self {
        Self {
            x: centre_x - (text_width.saturating_div(2)),
            y: centre_y - *processed_base_configuration.task_rect_half_height(),
            width: text_width,
            height: *processed_base_configuration.task_rect_height(),
            rx: "10",
            fill: TASK_RECT_FILL,
            stroke: "black",
            stroke_width: CORE_ROUTER_STROKE_WIDTH_STR,
        }
    }
}

/// Helper struct to group [`TaskRect`] and its corresponding [`TextInformation`] together, forms the task bubble in the SVG.
#[derive(Serialize)]
struct Task {
    rect: TaskRect,
    text: TextInformation,
}

impl Task {
    /// Generates a new [`Task`] instance from the given parameters.
    fn new(
        row: &CoordinateT,
        column: &CoordinateT,
        task_id: &Option<u16>,
        top_left: &TopLeft,
        processed_base_configuration: &ProcessedBaseConfiguration,
    ) -> Result<Option<Self>, SVGError> {
        match task_id {
            Some(task) => {
                let text = format!("T{}", task);

                // Get an approx text width
                let text_width = TextInformation::calculate_length_util(
                    *processed_base_configuration.task_font_size(),
                    text.len(),
                    Some(CHAR_H_PADDING),
                )?;

                // Get centre coordinates
                let (cx, cy) = Self::get_centre_coordinates(
                    row,
                    column,
                    text_width,
                    top_left,
                    processed_base_configuration,
                );

                Ok(Some(Self {
                    rect: TaskRect::new(cx, cy, text_width, processed_base_configuration),
                    text: TextInformation::new(
                        cx,
                        cy,
                        *processed_base_configuration.task_font_size(),
                        "middle",
                        "central",
                        None,
                        None,
                        text,
                    ),
                }))
            }
            None => Ok(None),
        }
    }

    /// Calculates centre coordinates of a [`Task`] group by leveraging the provided approximate text width.
    fn get_centre_coordinates(
        row: &CoordinateT,
        column: &CoordinateT,
        text_width: CoordinateT,
        top_left: &TopLeft,
        processed_base_configuration: &ProcessedBaseConfiguration,
    ) -> (CoordinateT, CoordinateT) {
        let cx = column * BLOCK_LENGTH + column * BLOCK_DISTANCE - (text_width.saturating_div(2))
            + TASK_RECT_X_OFFSET
            + top_left.x();
        let cy = processed_base_configuration.task_rect_centre_offset()
            + row * BLOCK_LENGTH
            + ROUTER_OFFSET
            + SIDE_LENGTH
            + row * BLOCK_DISTANCE
            + top_left.y();

        (cx, cy)
    }
}

/// Object representattion of the SVG `<path>` that makes up a router.
#[derive(Serialize, MutGetters, Getters)]
pub(crate) struct Router {
    /// Router coordinates, (x, y)
    #[serde(skip)]
    #[getset(get = "pub")]
    move_coordinates: (CoordinateT, CoordinateT),
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CommonAttributes,
}

impl Router {
    /// Generates a new [`Router`] instance from the given parameters.
    pub(crate) fn new(
        row: &CoordinateT,
        column: &CoordinateT,
        id: &u8,
        top_left: &TopLeft,
    ) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(row, column, top_left);

        Self {
            move_coordinates: (move_x, move_y),
            id: format!("r{}", id),
            d: format!("M{},{} {}", move_x, move_y, ROUTER_PATH),
            attributes: CommonAttributes::default(),
        }
    }

    /// Calculates the move coordinates for a [`Router`] path given current row, column and the viewBox [`TopLeft`].
    pub(crate) fn get_move_coordinates(
        row: &CoordinateT,
        column: &CoordinateT,
        top_left: &TopLeft,
    ) -> (CoordinateT, CoordinateT) {
        let move_x = (column * BLOCK_LENGTH)
            + ROUTER_OFFSET
            + column * BLOCK_DISTANCE
            + top_left.x()
            + CORE_ROUTER_STROKE_WIDTH;
        let move_y = row * BLOCK_LENGTH
            + ROUTER_OFFSET
            + row * BLOCK_DISTANCE
            + top_left.y()
            + CORE_ROUTER_STROKE_WIDTH;

        (move_x, move_y)
    }
}

/// Object representattion of the SVG `<path>` that makes up a core.
#[derive(Serialize, MutGetters, Getters)]
pub struct Core {
    /// Core coordinates, (x, y)
    #[serde(skip)]
    #[getset(get = "pub")]
    move_coordinates: (CoordinateT, CoordinateT),
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    #[getset(get_mut = "pub")]
    attributes: CommonAttributes,
}

impl Core {
    /// Calculates the move coordinates for a [`Core`] path given current row, column and the viewBox [`TopLeft`].
    pub(crate) fn get_move_coordinates(
        row: &CoordinateT,
        column: &CoordinateT,
        top_left: &TopLeft,
    ) -> (CoordinateT, CoordinateT) {
        let move_x = column * BLOCK_LENGTH
            + column * BLOCK_DISTANCE
            + top_left.x()
            + CORE_ROUTER_STROKE_WIDTH;
        let move_y = row * BLOCK_LENGTH
            + ROUTER_OFFSET
            + row * BLOCK_DISTANCE
            + top_left.y()
            + CORE_ROUTER_STROKE_WIDTH;

        (move_x, move_y)
    }

    /// Generates a new [`Core`] instance from the given parameters.
    fn new(row: &CoordinateT, column: &CoordinateT, id: &u8, top_left: &TopLeft) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(row, column, top_left);

        Self {
            move_coordinates: (move_x, move_y),
            id: format!("c{}", id),
            d: format!("M{},{} {}", move_x, move_y, PROCESSOR_PATH),
            attributes: CommonAttributes::default(),
        }
    }
}

/// Object representation of an SVG `<g>` that contains a [`Core`], [`Router`] and [`Task`].
#[derive(Serialize, MutGetters, Getters)]
pub(crate) struct ProcessingGroup {
    #[serde(skip)]
    #[getset(get = "pub")]
    /// Coordinates (row, column)
    coordinates: (CoordinateT, CoordinateT),
    #[serde(rename = "@id")]
    id: u8,
    #[serde(rename = "path")]
    #[getset(get = "pub")]
    core: Core,
    #[serde(rename = "path")]
    #[getset(get = "pub")]
    router: Router,
    #[serde(rename = "g", skip_serializing_if = "Option::is_none")]
    task: Option<Task>,
}

impl ProcessingGroup {
    /// Generates a new [`ProcessingGroup`] instance from the given parameters.
    pub(crate) fn new(
        row: &CoordinateT,
        column: &CoordinateT,
        id: &u8,
        allocated_task: &Option<u16>,
        top_left: &TopLeft,
        prrocessed_base_configuration: &ProcessedBaseConfiguration,
    ) -> Result<Self, SVGError> {
        Ok(Self {
            coordinates: (*row, *column),
            id: *id,
            core: Core::new(row, column, id, top_left),
            router: Router::new(row, column, id, top_left),
            task: Task::new(
                row,
                column,
                allocated_task,
                top_left,
                prrocessed_base_configuration,
            )?,
        })
    }

    /// Returns this [`ProcessingGroup`]'s [`Task`] `x` coordinate, if any.
    pub(crate) fn task_start(&self) -> Option<CoordinateT> {
        match &self.task {
            Some(task) => Some(task.rect.x),
            None => None,
        }
    }
}

/// An SVG `<g>` that wraps all [`ProcessingGroup`] instances.
#[derive(Serialize, MutGetters, Getters)]
#[getset(get_mut = "pub", get = "pub")]
pub(crate) struct ProcessingParentGroup {
    #[serde(rename = "@id")]
    id: &'static str,
    g: Vec<ProcessingGroup>,
}

impl ProcessingParentGroup {
    /// Generates a new [`ProcessingParentGroup`] with capacity for `number_of_cores` instances of [`ProcessingGroup`]s.
    pub(crate) fn new(number_of_cores: &usize) -> Self {
        Self {
            id: "processingGroup",
            g: Vec::with_capacity(*number_of_cores),
        }
    }
}
