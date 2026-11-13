use opsis_scene::color::Color;
use opsis_scene::measure::Measure;
use opsis_scene::node::Node;
use opsis_scene::timeline::{At, DrawError, Drawn, Library, Presented, SceneTracks, Side};
use opsis_types::recording::RecordingProps;
use opsis_types::timeline::SequencerProps;
use serde_json::{Map, Value};

use crate::elements::draw::{self, Cx, css_color};

#[derive(Clone, Copy)]
pub struct FilmLibrary<'m> {
    pub measure: &'m dyn Measure,
}

impl std::fmt::Debug for FilmLibrary<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilmLibrary").finish_non_exhaustive()
    }
}

impl Library for FilmLibrary<'_> {
    fn element(&self, component: &str, props: &Map<String, Value>, at: &At) -> Drawn {
        draw::element(
            component,
            props,
            &Cx {
                at,
                measure: self.measure,
            },
        )
    }

    fn wrapper(&self, component: &str, props: &Map<String, Value>, child: Node, at: &At) -> Drawn {
        draw::wrapper(
            component,
            props,
            child,
            &Cx {
                at,
                measure: self.measure,
            },
        )
    }

    fn presentation(&self, presented: &Presented<'_>, children: Vec<Node>, at: &At) -> Drawn {
        draw::presentation(
            &presented.transition.kind,
            presented.transition.params.as_ref(),
            draw::Transition {
                entering: presented.side == Side::Entering,
                progress: presented.progress,
                duration: f64::from(presented.transition.duration_in_frames),
                children,
            },
            &Cx {
                at,
                measure: self.measure,
            },
        )
    }

    fn scenes(&self, _: &SequencerProps, _: &dyn SceneTracks, _: &At) -> Drawn {
        Ok(None)
    }

    fn recording(&self, _: &RecordingProps, _: &At) -> Result<Node, DrawError> {
        Err(DrawError::new(
            "recording",
            "takes are not drawn natively yet",
        ))
    }

    fn color(&self, css: &str) -> Color {
        css_color(css)
    }
}
