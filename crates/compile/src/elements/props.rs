use serde::Serialize;

use crate::clip::PlaceholderProps;
use crate::nodes::capture::StillProps;

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct CoreElementProps {
    #[serde(rename = "placeholder")]
    pub placeholder: PlaceholderProps,
    #[serde(rename = "still")]
    pub still: StillProps,
    #[serde(rename = "title")]
    pub title: crate::elements::text::title::Title,
    #[serde(rename = "kinetic-headline")]
    pub kinetic_headline: crate::elements::text::kinetic_headline::KineticHeadline,
    #[serde(rename = "lower-third")]
    pub lower_third: crate::elements::overlay::lower_third::LowerThird,
    #[serde(rename = "callout")]
    pub callout: crate::elements::overlay::callout::Callout,
    #[serde(rename = "captions")]
    pub captions: crate::elements::overlay::captions::Captions,
    #[serde(rename = "image")]
    pub image: crate::elements::media::image::Image,
    #[serde(rename = "mosaic")]
    pub mosaic: crate::elements::effect::mosaic::Mosaic,
    #[serde(rename = "lottie")]
    pub lottie: crate::elements::overlay::lottie::Lottie,
    #[serde(rename = "video")]
    pub video: crate::elements::media::video::Video,
    #[serde(rename = "solid")]
    pub solid: crate::elements::background::solid::Solid,
    #[serde(rename = "gradient")]
    pub gradient: crate::elements::background::gradient::Gradient,
    #[serde(rename = "noise")]
    pub noise: crate::elements::background::noise::Noise,
    #[serde(rename = "starburst")]
    pub starburst: crate::elements::background::starburst::Starburst,
    #[serde(rename = "paper")]
    pub paper: crate::elements::background::paper::Paper,
    #[serde(rename = "backdrop")]
    pub backdrop: crate::elements::background::backdrop::StudioBackdrop,
    #[serde(rename = "video-loop")]
    pub video_loop: crate::elements::background::video_loop::VideoLoop,
    #[serde(rename = "light-leak")]
    pub light_leak: crate::elements::overlay::light_leak::LightLeak,
    #[serde(rename = "logo-3d")]
    pub logo_3d: crate::elements::scene::logo_3d::Logo3d,
    #[serde(rename = "card-3d")]
    pub card_3d: crate::elements::scene::card_3d::Card3d,
    #[serde(rename = "text-3d")]
    pub text_3d: crate::elements::text::text_3d::Text3d,
    #[serde(rename = "model-3d")]
    pub model_3d: crate::elements::scene::model_3d::Model3d,
    #[serde(rename = "screen-3d")]
    pub screen_3d: crate::elements::device::screen_3d::Screen3d,
    #[serde(rename = "shapes-3d")]
    pub shapes_3d: crate::elements::background::shapes_3d::Shapes3d,
}
