pub mod background;
pub mod data;
pub mod default_props;
pub mod device;
pub mod effect;
pub mod entry;
pub mod introspect;
pub mod length_cases;
pub mod lengths;
pub mod media;
pub mod overlay;
pub mod prepared;
pub mod props;
pub mod scene;
pub mod shared;
pub mod text;
pub mod theme;
pub mod transition;
pub mod ui;

use std::collections::HashMap;
use std::sync::LazyLock;

use indexmap::IndexMap;

use opsis_types::looks::NodeLook;
use opsis_types::timeline::Design;
use serde::{Deserialize, Serialize};

use crate::params::{ParamSchema, Values};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElementCategory {
    Media,
    Text,
    Overlay,
    Transition,
    Background,
    Effect,
    Ui,
    Data,
    Scene,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElementKind {
    Component,
    Wrapper,
    Presentation,
    Sequencer,
}

#[derive(Debug, Clone)]
pub struct ElementMeta {
    pub key: String,
    pub title: String,
    pub description: Option<String>,
    pub category: ElementCategory,
    pub kind: ElementKind,
    pub schema: ParamSchema,
    pub default_props: Values,
    pub fixed_props: Values,
    pub kind_props: IndexMap<String, String>,
    pub default_duration_in_frames: u32,
    pub design: Option<Design>,
    pub optional_clip: bool,
    pub thumbnail: Option<String>,
    pub look: Option<NodeLook>,
    pub authored_clock: bool,
    pub fonts: Vec<String>,
}

struct Elements {
    list: Vec<ElementMeta>,
    by_key: HashMap<String, usize>,
}

static ELEMENTS: LazyLock<Elements> = LazyLock::new(|| {
    let list: Vec<ElementMeta> = vec![
        text::title::definition(),
        text::kinetic_headline::definition(),
        overlay::lower_third::definition(),
        overlay::callout::definition(),
        overlay::captions::definition(),
        media::image::definition(),
        effect::mosaic::definition(),
        overlay::lottie::definition(),
        media::video::definition(),
        background::solid::definition(),
        background::gradient::definition(),
        background::noise::definition(),
        background::starburst::definition(),
        background::paper::definition(),
        background::backdrop::definition(),
        background::video_loop::definition(),
        overlay::light_leak::definition(),
        scene::logo_3d::definition(),
        scene::card_3d::definition(),
        text::text_3d::definition(),
        scene::model_3d::definition(),
        device::screen_3d::definition(),
        background::shapes_3d::definition(),
        transition::fade::definition(),
        transition::slide::definition(),
        transition::wipe::definition(),
        transition::flip::definition(),
        transition::clock_wipe::definition(),
        transition::iris::definition(),
        text::soft_blur_in::definition(),
        text::typewriter::definition(),
        text::shimmer_sweep::definition(),
        text::matrix_decode::definition(),
        text::slot_machine_roll::definition(),
        text::staggered_fade_up::definition(),
        text::kinetic_center_build::definition(),
        text::mask_reveal_up::definition(),
        text::tracking_in::definition(),
        text::word_push::definition(),
        text::rgb_glitch_text::definition(),
        text::gradient_scale_cut_text::definition(),
        text::strikethrough_replace::definition(),
        text::line_by_line_slide::definition(),
        overlay::marker_highlight::definition(),
        overlay::inline_highlight::definition(),
        overlay::ink_underline::definition(),
        overlay::scribble_circle::definition(),
        overlay::ink_arrow::definition(),
        overlay::keystroke::definition(),
        overlay::confetti::definition(),
        overlay::radial_burst::definition(),
        transition::whip_pan::definition(),
        transition::push_through::definition(),
        transition::zoom_blur::definition(),
        transition::focus_pull::definition(),
        transition::lens_zoom::definition(),
        transition::ripple_zoom::definition(),
        transition::wave_wipe::definition(),
        transition::smoke_dissolve::definition(),
        transition::glitch_cut::definition(),
        transition::page_turn::definition(),
        transition::grain_dissolve::definition(),
        transition::dither_dissolve::definition(),
        transition::swirl_dissolve::definition(),
        transition::particle_dissolve::definition(),
        background::shader_mesh_gradient::definition(),
        background::shader_grain_gradient::definition(),
        background::shader_warp::definition(),
        background::shader_swirl::definition(),
        background::shader_god_rays::definition(),
        background::shader_perlin_noise::definition(),
        background::shader_dithering::definition(),
        effect::crt_screen::definition(),
        effect::vhs_filter::definition(),
        effect::halftone_print::definition(),
        effect::camera_lens::definition(),
        effect::polaroid::definition(),
        effect::tv_power_off::definition(),
        effect::speed_lines::definition(),
        ui::terminal_simulator::definition(),
        ui::glass_code_block::definition(),
        ui::code_morph::definition(),
        ui::logo_enter::definition(),
        ui::check_list::definition(),
        ui::search_reveal::definition(),
        ui::agent_run::definition(),
        data::animated_bar_chart::definition(),
        data::animated_line_chart::definition(),
        data::rolling_number::definition(),
        scene::spring_settle::definition(),
        scene::signup_flow::definition(),
        scene::ai_prompt_flow::definition(),
        scene::command_menu::definition(),
        scene::release_teaser::definition(),
        scene::launch_anything::definition(),
        device::laptop_frame::definition(),
        device::phone_frame::definition(),
        device::captured_video::definition(),
        text::announce_title::definition(),
        text::punch_lines::definition(),
        text::text_highlight::definition(),
        text::status_cycle::definition(),
        scene::logo_assemble::definition(),
        scene::logo_drift::definition(),
        scene::hero_launch::definition(),
        transition::grid_pixelate_wipe::definition(),
        transition::frosted_glass_wipe::definition(),
        transition::chromatic_aberration_wipe::definition(),
        transition::transition_blinds::definition(),
        transition::transition_morph_shape::definition(),
        data::stat_card::definition(),
        data::metric_ticker::definition(),
        data::funnel_chart::definition(),
        data::donut_chart::definition(),
        data::comparison_bars::definition(),
        data::bar_chart_race::definition(),
        data::gauge_dial::definition(),
        scene::data_story::definition(),
        scene::logo_reveal::definition(),
        scene::pricing_card::definition(),
        scene::comparison_table::definition(),
        scene::changelog_entry::definition(),
        scene::logo_wall::definition(),
        scene::end_card::definition(),
        ui::code_reveal::definition(),
        ui::code_diff_wipe::definition(),
        ui::kanban_move::definition(),
        ui::file_tree_reveal::definition(),
        ui::drag_drop_flow::definition(),
        ui::commit_graph::definition(),
        overlay::notification_stack::definition(),
        device::device_mockup_zoom::definition(),
        device::multi_device_lineup::definition(),
        text::handwriting_text::definition(),
        text::neon_flicker_text::definition(),
        text::text_mask_video::definition(),
        background::aurora_bg::definition(),
        background::particle_field::definition(),
    ];
    let by_key = list
        .iter()
        .enumerate()
        .map(|(i, e)| (e.key.clone(), i))
        .collect();
    Elements { list, by_key }
});

pub fn elements() -> &'static [ElementMeta] {
    &ELEMENTS.list
}

pub fn element_meta(key: &str) -> Option<&'static ElementMeta> {
    ELEMENTS.by_key.get(key).map(|i| &ELEMENTS.list[*i])
}

pub fn element_node_type(key: &str) -> String {
    format!("fx.{key}")
}
