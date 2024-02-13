mod connections_group;
mod events_rect;
mod exporting_aid;
mod marker;
mod processing_group;

use connections_group::*;
use events_rect::EventsRect;
use exporting_aid::*;
use marker::*;
use processing_group::*;

use manycore_parser::ManycoreSystem;
use quick_xml::DeError;
use serde::Serialize;

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
struct Defs {
    marker: Marker,
}

#[derive(Serialize)]
struct Root {
    #[serde(rename = "g")]
    processing_group: ProcessingParentGroup,
    #[serde(rename = "g")]
    connections_group: ConnectionsParentGroup,
    rect: EventsRect,
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
    root: Root,
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
            root: Root {
                processing_group: ProcessingParentGroup::new(),
                connections_group: ConnectionsParentGroup::new(),
                rect: EventsRect::default(),
            },
            exporting_aid: ExportingAid::default(),
        }
    }
}

impl From<&ManycoreSystem> for SVG {
    fn from(manycore: &ManycoreSystem) -> Self {
        let mut ret = SVG::default();

        let columns = u16::from(*manycore.columns());
        let rows = u16::from(*manycore.rows());

        let width = (columns * UNIT_LENGTH) + ((columns - 1) * GROUP_DISTANCE);
        let height = (rows * UNIT_LENGTH) + ((rows - 1) * GROUP_DISTANCE);
        ret.view_box.push_str(&format!("0 0 {} {}", width, height));

        let mut r: u8 = 0;

        for i in 0..manycore.cores().list().len() {
            // This cast here might look a bit iffy as the result of the mod
            // might not fit in 8 bits. However, since manycore.columns is 8 bits,
            // that should never happen.
            let c = (i % usize::from(*manycore.columns())) as u8;

            if i > 0 && c == 0 {
                r += 1;
            }

            let group_id = format!("{},{}", r + 1, c + 1);
            let r16 = u16::from(r);
            let c16 = u16::from(c);

            ret.root
                .processing_group
                .g_mut()
                .push(ProcessingGroup::new(&r16, &c16, &group_id));

            ret.root.connections_group.add_neighbours(
                i,
                manycore.connections().get(&i),
                &r16,
                &c16,
            );
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
