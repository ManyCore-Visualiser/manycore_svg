use std::collections::HashMap;

use manycore_parser::{ManycoreSystem, Neighbours};
use quick_xml::DeError;
use serde::{Serialize, Serializer};

static PROCESSOR_PATH: &str = "l0,100 l100,0 l0,-75 l-25,-25 l-75,0 Z";
static ROUTER_PATH: &str = "l0,-75 l100,0 l0,100 l-75,0 Z";
static UNIT_LENGTH: u16 = 175;
static SIDE_LENGTH: u16 = 100;
static HALF_SIDE_LENGTH: u16 = 50;
static OUTPUT_LINK_OFFSET: u16 = 25;
static ROUTER_OFFSET: u16 = 75;
static GROUP_DISTANCE: u16 = 120;
static MARKER_PATH: &str = "M0,0 M0,0 V8 L8,4 Z";
static MARKER_REFERENCE: &str = "url(#arrowHead)";
static CONNECTION_LENGTH: u8 = 187;

#[derive(Serialize)]
pub struct ExportingAid {
    #[serde(rename = "@width")]
    width: &'static str,
    #[serde(rename = "@height")]
    height: &'static str,
    #[serde(rename = "@fill")]
    fill: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl Default for ExportingAid {
    fn default() -> Self {
        Self {
            width: "100%",
            height: "100%",
            fill: "none",
            stroke: "#ff0000",
            stroke_width: "1",
        }
    }
}

#[derive(Serialize)]
pub struct CoreRouterCommon {
    #[serde(rename = "@fill")]
    fill: &'static str,
    #[serde(rename = "@fill-rule")]
    fill_rule: &'static str,
    #[serde(rename = "@stroke")]
    stroke: &'static str,
    #[serde(rename = "@stroke-linecap")]
    stroke_linecap: &'static str,
    #[serde(rename = "@stroke-width")]
    stroke_width: &'static str,
}

impl Default for CoreRouterCommon {
    fn default() -> Self {
        Self {
            fill: "none",
            fill_rule: "evenodd",
            stroke: "black",
            stroke_linecap: "butt",
            stroke_width: "1",
        }
    }
}

#[derive(Serialize)]
pub struct Router {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    attributes: CoreRouterCommon,
}

impl Router {
    fn new(r: &u16, c: &u16, group_id: &String) -> Self {
        let (move_x, move_y) = Self::get_move_coordinates(r, c);

        Self {
            id: format!("{}r", group_id),
            d: format!("M{},{} {}", move_x, move_y, ROUTER_PATH),
            attributes: CoreRouterCommon::default(),
        }
    }

    fn get_move_coordinates(r: &u16, c: &u16) -> (u16, u16) {
        let move_x =
            (c * UNIT_LENGTH) + ROUTER_OFFSET + if *c == 0 { 0 } else { c * GROUP_DISTANCE };
        let move_y = r * UNIT_LENGTH + ROUTER_OFFSET + if *r == 0 { 0 } else { r * GROUP_DISTANCE };

        (move_x, move_y)
    }
}

#[derive(Serialize)]
pub struct Core {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    attributes: CoreRouterCommon,
}

impl Core {
    fn new(r: &u16, c: &u16, group_id: &String) -> Self {
        let move_x = c * UNIT_LENGTH + if *c == 0 { 0 } else { c * GROUP_DISTANCE };
        let move_y = r * UNIT_LENGTH + ROUTER_OFFSET + if *r == 0 { 0 } else { r * GROUP_DISTANCE };

        Self {
            id: format!("{}c", group_id),
            d: format!("M{},{} {}", move_x, move_y, PROCESSOR_PATH),
            attributes: CoreRouterCommon::default(),
        }
    }
}

#[derive(Serialize)]
pub struct ProcessingGroup {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "path")]
    core: Core,
    #[serde(rename = "path")]
    router: Router,
}

impl ProcessingGroup {
    fn new(r: &u16, c: &u16, group_id: &String) -> Self {
        Self {
            id: group_id.clone(),
            core: Core::new(r, c, &group_id),
            router: Router::new(r, c, &group_id),
        }
    }
}

#[derive(Serialize)]
pub struct ProcessingParentGroup {
    g: Vec<ProcessingGroup>,
}

#[derive(Serialize)]
pub struct Connection {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@d")]
    d: String,
    #[serde(flatten)]
    attributes: CoreRouterCommon,
    #[serde(rename = "@marker-end")]
    marker_end: &'static str,
}

enum ConnectionDirection {
    TOP,
    RIGHT,
    BOTTOM,
    LEFT,
}

impl Connection {
    fn get_path(direction: ConnectionDirection, r: &u16, c: &u16) -> String {
        let (router_x, router_y) = Router::get_move_coordinates(r, c);
        let router_centre_x = router_x + HALF_SIDE_LENGTH;
        let router_centre_y = router_y + (SIDE_LENGTH - ROUTER_OFFSET) - HALF_SIDE_LENGTH;

        match direction {
            ConnectionDirection::TOP => format!(
                "M{},{} v-{}",
                router_centre_x + OUTPUT_LINK_OFFSET,
                router_centre_y - HALF_SIDE_LENGTH,
                CONNECTION_LENGTH
            ),
            ConnectionDirection::RIGHT => format!(
                "M{},{} h{}",
                router_centre_x + HALF_SIDE_LENGTH,
                router_centre_y - OUTPUT_LINK_OFFSET,
                CONNECTION_LENGTH
            ),
            ConnectionDirection::BOTTOM => format!(
                "M{},{} v{}",
                router_centre_x,
                router_centre_y + HALF_SIDE_LENGTH,
                CONNECTION_LENGTH
            ),
            ConnectionDirection::LEFT => format!(
                "M{},{} h-{}",
                router_centre_x - HALF_SIDE_LENGTH,
                router_centre_y,
                CONNECTION_LENGTH
            ),
        }
    }
}

#[derive(Serialize)]
pub struct ConnectionsParentGroup {
    #[serde(serialize_with = "serialise_map")]
    path: HashMap<String, Connection>,
}

fn serialise_map<S>(map: &HashMap<String, Connection>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();

    // TODO: Clean this up without unwrap
    let entries: Vec<&Connection> = keys.iter().map(|k| map.get(*k).unwrap()).collect();

    entries.serialize(serializer)
}

impl ConnectionsParentGroup {
    fn add_neighbour(
        &mut self,
        i: usize,
        neighbour: usize,
        direction: ConnectionDirection,
        r: &u16,
        c: &u16,
    ) {
        let connection_id = format!("{}-{}", i, neighbour);
        self.path.insert(
            connection_id.clone(),
            Connection {
                id: connection_id,
                d: Connection::get_path(direction, &r, &c),
                attributes: CoreRouterCommon::default(),
                marker_end: MARKER_REFERENCE,
            },
        );
    }

    pub fn add_neighbours(
        &mut self,
        i: usize,
        opt_neighbours: Option<&Neighbours>,
        r: &u16,
        c: &u16,
    ) {
        if let Some(neighbours) = opt_neighbours {
            if let Some(top) = neighbours.top() {
                self.add_neighbour(i, top, ConnectionDirection::TOP, r, c);
            }

            if let Some(right) = neighbours.right() {
                self.add_neighbour(i, right, ConnectionDirection::RIGHT, r, c);
            }

            if let Some(bottom) = neighbours.bottom() {
                self.add_neighbour(i, bottom, ConnectionDirection::BOTTOM, r, c);
            }

            if let Some(left) = neighbours.left() {
                self.add_neighbour(i, left, ConnectionDirection::LEFT, r, c);
            }
        }
    }
}

#[derive(Serialize)]
struct MarkerPath {
    #[serde(rename = "@d")]
    d: &'static str,
    #[serde(flatten)]
    attributes: CoreRouterCommon,
}

impl Default for MarkerPath {
    fn default() -> Self {
        let mut attributes = CoreRouterCommon::default();
        attributes.fill = "black";

        Self {
            d: MARKER_PATH,
            attributes,
        }
    }
}

#[derive(Serialize)]
struct Marker {
    #[serde(rename = "@id")]
    id: &'static str,
    #[serde(rename = "@orient")]
    orient: &'static str,
    #[serde(rename = "@markerWidth")]
    marker_width: &'static str,
    #[serde(rename = "@markerHeight")]
    marker_height: &'static str,
    #[serde(rename = "@refY")]
    ref_y: &'static str,
    path: MarkerPath,
}

impl Default for Marker {
    fn default() -> Self {
        Self {
            id: "arrowHead",
            orient: "auto",
            marker_width: "8",
            marker_height: "8",
            ref_y: "4",
            path: MarkerPath::default(),
        }
    }
}

#[derive(Serialize)]
struct Defs {
    marker: Marker,
}

#[derive(Serialize)]
#[serde(rename = "svg")]
pub struct SVG {
    #[serde(rename = "@xmlns:svg")]
    xmlns_svg: &'static str,
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "@preserveAspectRation")]
    preserve_aspect_ratio: &'static str,
    #[serde(rename = "@class")]
    class: String,
    #[serde(rename = "@viewBox")]
    view_box: String,
    defs: Defs,
    #[serde(rename = "g")]
    processing_group: ProcessingParentGroup,
    #[serde(rename = "g")]
    connections_group: ConnectionsParentGroup,
    #[serde(rename = "rect")]
    exporting_aid: ExportingAid,
}

impl TryFrom<&SVG> for String {
    type Error = DeError;

    fn try_from(svg: &SVG) -> Result<Self, Self::Error> {
        quick_xml::se::to_string(svg)
    }
}

impl Default for SVG {
    fn default() -> Self {
        Self {
            xmlns_svg: "http://www.w3.org/2000/svg",
            xmlns: "http://www.w3.org/2000/svg",
            preserve_aspect_ratio: "xMidYMid meet",
            class: String::from("w-full max-h-full"),
            view_box: String::new(),
            defs: Defs {
                marker: Marker::default(),
            },
            processing_group: ProcessingParentGroup { g: vec![] },
            connections_group: ConnectionsParentGroup {
                path: HashMap::new(),
            },
            exporting_aid: ExportingAid::default(),
        }
    }
}

impl From<&ManycoreSystem> for SVG {
    fn from(manycore: &ManycoreSystem) -> Self {
        let mut ret = SVG::default();

        let columns = u16::from(manycore.columns);
        let rows = u16::from(manycore.rows);

        let width = (columns * UNIT_LENGTH) + ((columns - 1) * GROUP_DISTANCE);
        let height = (rows * UNIT_LENGTH) + ((rows - 1) * GROUP_DISTANCE);
        ret.view_box.push_str(&format!("0 0 {} {}", width, height));

        let mut r: u8 = 0;

        for i in 0..manycore.cores.list.len() {
            // This cast here might look a bit iffy as the result of the mod
            // might not fit in 8 bits. However, since manycore.columns is 8 bits,
            // that should never happen.
            let c = (i % usize::from(manycore.columns)) as u8;

            if i > 0 && c == 0 {
                r += 1;
            }

            let group_id = format!("{},{}", r + 1, c + 1);
            let r16 = u16::from(r);
            let c16 = u16::from(c);

            ret.processing_group
                .g
                .push(ProcessingGroup::new(&r16, &c16, &group_id));

            ret.connections_group
                .add_neighbours(i, manycore.connections.get(&i), &r16, &c16);
        }

        return ret;
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use manycore_parser::ManycoreSystem;

    use super::SVG;

    #[test]
    fn can_convert_from() {
        let manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        let svg = SVG::from(&manycore);

        let res =
            quick_xml::se::to_string(&svg).expect("Could not convert from ManycoreSystem to SVG");

        let expected = read_to_string("tests/SVG1.svg")
            .expect("Could not read input test file \"tests/SVG1.svg\"");

        assert_eq!(res, expected)
    }
}
