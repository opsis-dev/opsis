use opsis_scene::color::Color;
use opsis_scene::measure::Measure;
use opsis_scene::node::{Node, div};
use opsis_scene::style::Style;
use opsis_scene::timeline::{At, ChildAt, DrawError, Drawn, Library, Presented, SceneTracks, Side};
use opsis_types::recording::RecordingProps;
use opsis_types::timeline::{ReframeMode, SequencerProps};
use serde_json::{Map, Value};

use crate::elements::camera::recording::recording;
use crate::elements::draw::{self, Cx, css_color};

#[derive(Clone, Copy)]
pub struct FilmLibrary<'m> {
    pub measure: &'m dyn Measure,
    pub reframe: ReframeMode,
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

    fn wrapper(
        &self,
        component: &str,
        props: &Map<String, Value>,
        child: ChildAt<'_>,
        at: &At,
    ) -> Drawn {
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

    fn scenes(&self, sequencer: &SequencerProps, scenes: &dyn SceneTracks, at: &At) -> Drawn {
        draw::scenes(
            &sequencer.component,
            &sequencer.props,
            scenes,
            &Cx {
                at,
                measure: self.measure,
            },
        )
    }

    fn recording(&self, props: &RecordingProps, at: &At) -> Result<Node, DrawError> {
        recording(props, at, self.reframe)
            .map(|drawn| drawn.unwrap_or_else(|| div(Style::default()).into()))
            .map_err(|e| DrawError::new("recording", e))
    }

    fn color(&self, css: &str) -> Color {
        css_color(css)
    }
}
