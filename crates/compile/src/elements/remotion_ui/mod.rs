mod background;
mod data;
mod device;
mod overlay;
mod scene;
mod text;
mod transition;
mod ui;

use crate::elements::ElementMeta;

pub fn elements() -> Vec<ElementMeta> {
    [
        transition::elements(),
        data::elements(),
        scene::elements(),
        ui::elements(),
        overlay::elements(),
        device::elements(),
        text::elements(),
        background::elements(),
    ]
    .concat()
}
