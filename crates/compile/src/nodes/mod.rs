pub mod audio;
pub mod browser;
pub mod camera;
pub mod capture;
pub mod cursor;
pub mod effects;
pub mod elements;
pub mod grade;
pub mod image;
pub mod media;
pub mod output;
pub mod text;
pub mod timeline;
pub mod transitions;
pub mod utility;

use crate::node::NodeDef;

pub fn all() -> Vec<NodeDef> {
    [
        browser::nodes(),
        capture::nodes(),
        camera::nodes(),
        cursor::nodes(),
        text::nodes(),
        effects::nodes(),
        grade::nodes(),
        elements::nodes(),
        transitions::nodes(),
        audio::nodes(),
        timeline::nodes(),
        media::nodes(),
        image::nodes(),
        output::nodes(),
        utility::nodes(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
