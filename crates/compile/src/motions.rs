use opsis_types::looks::NodeMotion;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub tx: f64,
    pub ty: f64,
    pub sx: f64,
    pub sy: f64,
    pub rotate: f64,
    pub tilt_x: f64,
    pub tilt_y: f64,
    pub opacity: f64,
    pub blur: f64,
    pub clip_top: f64,
    pub iris: f64,
    pub sweep: f64,
    pub mask: f64,
    pub dash: f64,
    pub split_x: f64,
    pub split_y: f64,
    pub rgb: f64,
    pub smear: f64,
}

pub const REST: Pose = Pose {
    tx: 0.0,
    ty: 0.0,
    sx: 1.0,
    sy: 1.0,
    rotate: 0.0,
    tilt_x: 0.0,
    tilt_y: 0.0,
    opacity: 1.0,
    blur: 0.0,
    clip_top: 0.0,
    iris: 0.75,
    sweep: 360.0,
    mask: 0.0,
    dash: 0.0,
    split_x: 0.0,
    split_y: 0.0,
    rgb: 0.0,
    smear: 0.0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ease {
    InOut,
    Out,
    Back,
    Linear,
    Step,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Glyph,
    Shapes,
    March,
    Ring,
    Flare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mask {
    None,
    Wipe,
    Clock,
    Shimmer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub pose: Pose,
    pub offset: Option<f64>,
    pub ease: Ease,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionSpec {
    pub frames: Vec<Frame>,
    pub ms: u32,
    pub origin: (f64, f64),
    pub part: Part,
    pub clip: bool,
    pub mask: Mask,
}

const fn at(pose: Pose, ease: Ease) -> Frame {
    Frame {
        pose,
        offset: None,
        ease,
    }
}

const fn at_offset(pose: Pose, offset: f64, ease: Ease) -> Frame {
    Frame {
        pose,
        offset: Some(offset),
        ease,
    }
}

const fn spec(frames: Vec<Frame>, ms: u32) -> MotionSpec {
    MotionSpec {
        frames,
        ms,
        origin: (0.5, 0.5),
        part: Part::Glyph,
        clip: false,
        mask: Mask::None,
    }
}

fn out_and_back(mid: Pose, ms: u32, ease: Ease) -> MotionSpec {
    spec(
        vec![at(REST, ease), at(mid, ease), at(REST, Ease::Linear)],
        ms,
    )
}

fn away(gone: Pose, back: Pose, ms: u32, ease: Ease) -> MotionSpec {
    let mut motion = spec(
        vec![
            at(REST, ease),
            at_offset(
                Pose {
                    opacity: 0.0,
                    ..gone
                },
                0.45,
                Ease::Step,
            ),
            at_offset(
                Pose {
                    opacity: 0.0,
                    ..back
                },
                0.46,
                ease,
            ),
            at(REST, Ease::Linear),
        ],
        ms,
    );
    motion.clip = true;
    motion
}

const fn scaled(s: f64) -> Pose {
    Pose {
        sx: s,
        sy: s,
        ..REST
    }
}

const fn blurred(px: f64) -> Pose {
    Pose { blur: px, ..REST }
}

fn orbit(step: u32) -> Pose {
    let angle = f64::from(step) * std::f64::consts::FRAC_PI_4;
    Pose {
        tx: angle.sin() * 0.12,
        ty: angle.cos().mul_add(-0.12, 0.12),
        ..REST
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one table of every motion, each entry as short as its CSS original"
)]
pub fn motion_spec(motion: NodeMotion) -> MotionSpec {
    use Ease::{Back, InOut, Linear, Out, Step};
    match motion {
        NodeMotion::Press => out_and_back(scaled(0.8), 320, Out),
        NodeMotion::Bob => out_and_back(Pose { ty: -0.12, ..REST }, 700, InOut),
        NodeMotion::NudgeUp => out_and_back(Pose { ty: -0.16, ..REST }, 520, InOut),
        NodeMotion::NudgeDown => out_and_back(Pose { ty: 0.16, ..REST }, 520, InOut),
        NodeMotion::NudgeRight => out_and_back(Pose { tx: 0.16, ..REST }, 520, InOut),
        NodeMotion::Pan => spec(
            vec![
                at(REST, InOut),
                at(Pose { tx: -0.16, ..REST }, InOut),
                at(Pose { tx: 0.16, ..REST }, InOut),
                at(REST, Linear),
            ],
            1300,
        ),
        NodeMotion::Track => spec((0..=8).map(|i| at(orbit(i % 8), Linear)).collect(), 1100),
        NodeMotion::Turn => spec(
            vec![
                at(REST, InOut),
                at(
                    Pose {
                        rotate: 360.0,
                        ..REST
                    },
                    Linear,
                ),
            ],
            800,
        ),
        NodeMotion::TurnOver => spec(
            vec![
                at(REST, InOut),
                at(
                    Pose {
                        rotate: 180.0,
                        ..REST
                    },
                    Linear,
                ),
            ],
            700,
        ),
        NodeMotion::Twist => out_and_back(
            Pose {
                rotate: 35.0,
                ..REST
            },
            600,
            InOut,
        ),
        NodeMotion::Stretch => out_and_back(Pose { sx: 1.3, ..REST }, 600, InOut),
        NodeMotion::Grow => MotionSpec {
            origin: (0.5, 1.0),
            ..spec(
                vec![
                    at(REST, InOut),
                    at(Pose { sy: 0.15, ..REST }, Back),
                    at(REST, Linear),
                ],
                700,
            )
        },
        NodeMotion::Open => MotionSpec {
            origin: (0.5, 1.0),
            ..out_and_back(
                Pose {
                    tilt_x: 55.0,
                    ..REST
                },
                800,
                InOut,
            )
        },
        NodeMotion::Flip => spec(
            vec![
                at(REST, InOut),
                at(
                    Pose {
                        tilt_y: 180.0,
                        ..REST
                    },
                    InOut,
                ),
                at(
                    Pose {
                        tilt_y: 360.0,
                        ..REST
                    },
                    Linear,
                ),
            ],
            800,
        ),
        NodeMotion::Shutter => MotionSpec {
            origin: (0.5, 0.0),
            ..out_and_back(Pose { sy: 0.0, ..REST }, 600, InOut)
        },
        NodeMotion::Collapse => spec(
            vec![
                at(REST, InOut),
                at_offset(Pose { sy: 0.06, ..REST }, 0.3, InOut),
                at_offset(scaled(0.06), 0.55, InOut),
                at_offset(
                    Pose {
                        opacity: 0.0,
                        ..scaled(0.06)
                    },
                    0.7,
                    InOut,
                ),
                at(REST, Linear),
            ],
            1100,
        ),
        NodeMotion::SlideThrough => away(
            Pose { tx: 0.7, ..REST },
            Pose { tx: -0.7, ..REST },
            800,
            InOut,
        ),
        NodeMotion::Push => away(
            Pose { tx: -0.6, ..REST },
            Pose { tx: 0.6, ..REST },
            800,
            Out,
        ),
        NodeMotion::Roll => away(
            Pose { ty: -0.8, ..REST },
            Pose { ty: 0.8, ..REST },
            700,
            InOut,
        ),
        NodeMotion::ZoomThrough => away(scaled(1.6), scaled(0.6), 800, InOut),
        NodeMotion::Whip => MotionSpec {
            clip: true,
            ..spec(
                vec![
                    at(REST, InOut),
                    at_offset(
                        Pose {
                            tx: 0.6,
                            blur: 3.0,
                            opacity: 0.0,
                            ..REST
                        },
                        0.4,
                        Step,
                    ),
                    at_offset(
                        Pose {
                            tx: -0.6,
                            blur: 3.0,
                            opacity: 0.0,
                            ..REST
                        },
                        0.41,
                        InOut,
                    ),
                    at(REST, Linear),
                ],
                520,
            )
        },
        NodeMotion::ZoomBlur => out_and_back(
            Pose {
                blur: 3.0,
                ..scaled(1.45)
            },
            700,
            InOut,
        ),
        NodeMotion::Smear => out_and_back(
            Pose {
                tx: 0.18,
                smear: 1.0,
                ..REST
            },
            650,
            InOut,
        ),
        NodeMotion::Draw => MotionSpec {
            part: Part::Shapes,
            ..spec(
                vec![
                    at(REST, InOut),
                    at_offset(Pose { dash: 1.0, ..REST }, 0.3, InOut),
                    at(REST, Linear),
                ],
                1100,
            )
        },
        NodeMotion::March => MotionSpec {
            part: Part::March,
            ..spec(
                vec![
                    at(REST, Linear),
                    at(
                        Pose {
                            dash: -15.0,
                            ..REST
                        },
                        Linear,
                    ),
                ],
                1500,
            )
        },
        NodeMotion::Caret => spec(
            [1.0, 0.0, 1.0, 0.0, 1.0, 1.0]
                .iter()
                .zip(0u32..)
                .map(|(opacity, i)| {
                    at_offset(
                        Pose {
                            opacity: *opacity,
                            ..REST
                        },
                        f64::from(i) / 5.0,
                        Step,
                    )
                })
                .collect(),
            1100,
        ),
        NodeMotion::WipeReveal => MotionSpec {
            mask: Mask::Wipe,
            ..spec(
                vec![
                    at(REST, InOut),
                    at_offset(Pose { mask: 1.0, ..REST }, 0.35, InOut),
                    at(REST, Linear),
                ],
                900,
            )
        },
        NodeMotion::MaskUp => spec(
            vec![
                at(REST, Out),
                at_offset(
                    Pose {
                        opacity: 0.0,
                        ..REST
                    },
                    0.2,
                    Step,
                ),
                at_offset(
                    Pose {
                        clip_top: 1.0,
                        ..REST
                    },
                    0.21,
                    Out,
                ),
                at(REST, Linear),
            ],
            800,
        ),
        NodeMotion::ClockWipe => MotionSpec {
            mask: Mask::Clock,
            ..out_and_back(Pose { sweep: 0.0, ..REST }, 1000, InOut)
        },
        NodeMotion::Iris => out_and_back(Pose { iris: 0.0, ..REST }, 800, InOut),
        NodeMotion::Shimmer => MotionSpec {
            mask: Mask::Shimmer,
            ..spec(
                vec![at(Pose { mask: 1.0, ..REST }, Linear), at(REST, Linear)],
                1000,
            )
        },
        NodeMotion::BlurSwap => {
            let hidden = Pose {
                blur: 5.0,
                opacity: 0.0,
                ..scaled(0.85)
            };
            spec(
                vec![
                    at(REST, InOut),
                    at_offset(hidden, 0.45, InOut),
                    at_offset(hidden, 0.55, InOut),
                    at(REST, Linear),
                ],
                750,
            )
        }
        NodeMotion::RackFocus => out_and_back(blurred(3.0), 800, InOut),
        NodeMotion::Rise => spec(
            vec![
                at(REST, Out),
                at_offset(
                    Pose {
                        opacity: 0.0,
                        ..REST
                    },
                    0.2,
                    Step,
                ),
                at_offset(
                    Pose {
                        ty: 0.35,
                        opacity: 0.0,
                        ..REST
                    },
                    0.21,
                    Out,
                ),
                at(REST, Linear),
            ],
            900,
        ),
        NodeMotion::Tracking => spec(
            vec![
                at(REST, InOut),
                at_offset(
                    Pose {
                        sx: 1.6,
                        blur: 3.0,
                        opacity: 0.3,
                        ..REST
                    },
                    0.35,
                    Back,
                ),
                at(REST, Linear),
            ],
            900,
        ),
        NodeMotion::Snap => spec(
            vec![
                at(REST, InOut),
                at_offset(
                    Pose {
                        blur: 3.0,
                        ..scaled(1.5)
                    },
                    0.5,
                    Step,
                ),
                at_offset(REST, 0.51, Step),
                at(REST, Linear),
            ],
            800,
        ),
        NodeMotion::FadeThrough => out_and_back(
            Pose {
                opacity: 0.1,
                ..REST
            },
            800,
            InOut,
        ),
        NodeMotion::Dissolve => out_and_back(
            Pose {
                opacity: 0.0,
                blur: 4.0,
                ..scaled(1.2)
            },
            900,
            InOut,
        ),
        NodeMotion::Glitch => {
            let split = |x: f64, y: f64, rgb: f64| Pose {
                split_x: x / 100.0,
                split_y: y / 100.0,
                rgb,
                ..REST
            };
            spec(
                vec![
                    at(split(0.0, 0.0, 0.0), Step),
                    at(split(3.0, -2.0, 2.0), Step),
                    at(split(-4.0, 1.0, -2.0), Step),
                    at(split(2.0, 3.0, 2.0), Step),
                    at(split(-3.0, -1.0, -2.0), Step),
                    at(split(4.0, 2.0, 2.0), Step),
                    at(split(0.0, 0.0, 0.0), Linear),
                ],
                520,
            )
        }
        NodeMotion::Flicker => spec(
            [
                (1.0, 0.0),
                (0.1, 0.08),
                (1.0, 0.16),
                (0.15, 0.3),
                (0.9, 0.38),
                (0.05, 0.5),
                (1.0, 0.62),
                (1.0, 1.0),
            ]
            .iter()
            .map(|(opacity, offset)| {
                at_offset(
                    Pose {
                        opacity: *opacity,
                        ..REST
                    },
                    *offset,
                    Step,
                )
            })
            .collect(),
            1000,
        ),
        NodeMotion::HardCut => spec(
            [1.0, 1.35, 0.75, 1.15, 1.0]
                .iter()
                .zip(0u32..)
                .map(|(s, i)| at_offset(scaled(*s), f64::from(i) / 4.0, Step))
                .collect(),
            800,
        ),
        NodeMotion::Ripple => MotionSpec {
            part: Part::Ring,
            ..spec(
                vec![
                    at(
                        Pose {
                            opacity: 0.9,
                            ..scaled(0.4)
                        },
                        Out,
                    ),
                    at(
                        Pose {
                            opacity: 0.0,
                            ..scaled(1.6)
                        },
                        Linear,
                    ),
                ],
                800,
            )
        },
        NodeMotion::Flare => MotionSpec {
            part: Part::Flare,
            ..spec(
                vec![
                    at(
                        Pose {
                            tx: -0.9,
                            opacity: 0.0,
                            ..REST
                        },
                        InOut,
                    ),
                    at(
                        Pose {
                            tx: 0.0,
                            opacity: 1.0,
                            ..REST
                        },
                        InOut,
                    ),
                    at(
                        Pose {
                            tx: 0.9,
                            opacity: 0.0,
                            ..REST
                        },
                        Linear,
                    ),
                ],
                1000,
            )
        },
    }
}

pub fn offsets(frames: &[Frame]) -> Vec<f64> {
    let last = frames.len().saturating_sub(1);
    let mut known: Vec<Option<f64>> = frames
        .iter()
        .enumerate()
        .map(|(i, f)| match (i, f.offset) {
            (_, Some(o)) => Some(o),
            (0, None) => Some(0.0),
            (i, None) if i == last => Some(1.0),
            _ => None,
        })
        .collect();
    let mut start = 0;
    while start < last {
        let end = (start + 1..=last)
            .find(|i| known[*i].is_some())
            .unwrap_or(last);
        let from = known[start].unwrap_or(0.0);
        let to = known[end].unwrap_or(1.0);
        let gaps = f64::from(u32::try_from(end - start).unwrap_or(1));
        for (step, slot) in known.iter_mut().enumerate().take(end).skip(start + 1) {
            let n = f64::from(u32::try_from(step - start).unwrap_or(0));
            *slot = Some((to - from).mul_add(n / gaps, from));
        }
        start = end;
    }
    known.into_iter().map(|o| o.unwrap_or(0.0)).collect()
}

fn bezier(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    let curve = |a: f64, b: f64, t: f64| {
        let u = 1.0 - t;
        (3.0 * u * t).mul_add(u.mul_add(a, t * b), t * t * t)
    };
    let (mut low, mut high) = (0.0f64, 1.0f64);
    let mut t = x;
    for _ in 0..40 {
        let guess = curve(x1, x2, t);
        if (guess - x).abs() < 1e-7 {
            break;
        }
        if guess < x {
            low = t;
        } else {
            high = t;
        }
        t = f64::midpoint(low, high);
    }
    curve(y1, y2, t)
}

pub fn eased(ease: Ease, progress: f64) -> f64 {
    let p = progress.clamp(0.0, 1.0);
    match ease {
        Ease::InOut => bezier(0.45, 0.0, 0.25, 1.0, p),
        Ease::Out => bezier(0.2, 0.8, 0.2, 1.0, p),
        Ease::Back => bezier(0.34, 1.56, 0.64, 1.0, p),
        Ease::Linear => p,
        Ease::Step => {
            if p >= 1.0 {
                1.0
            } else {
                0.0
            }
        }
    }
}

fn mix(a: f64, b: f64, t: f64) -> f64 {
    (b - a).mul_add(t, a)
}

fn blend(a: &Pose, b: &Pose, t: f64) -> Pose {
    Pose {
        tx: mix(a.tx, b.tx, t),
        ty: mix(a.ty, b.ty, t),
        sx: mix(a.sx, b.sx, t),
        sy: mix(a.sy, b.sy, t),
        rotate: mix(a.rotate, b.rotate, t),
        tilt_x: mix(a.tilt_x, b.tilt_x, t),
        tilt_y: mix(a.tilt_y, b.tilt_y, t),
        opacity: mix(a.opacity, b.opacity, t),
        blur: mix(a.blur, b.blur, t),
        clip_top: mix(a.clip_top, b.clip_top, t),
        iris: mix(a.iris, b.iris, t),
        sweep: mix(a.sweep, b.sweep, t),
        mask: mix(a.mask, b.mask, t),
        dash: mix(a.dash, b.dash, t),
        split_x: mix(a.split_x, b.split_x, t),
        split_y: mix(a.split_y, b.split_y, t),
        rgb: mix(a.rgb, b.rgb, t),
        smear: mix(a.smear, b.smear, t),
    }
}

pub fn sample(motion: &MotionSpec, elapsed_ms: f64) -> Pose {
    let Some(first) = motion.frames.first() else {
        return REST;
    };
    let progress = (elapsed_ms / f64::from(motion.ms.max(1))).clamp(0.0, 1.0);
    let stops = offsets(&motion.frames);
    for (i, pair) in motion.frames.windows(2).enumerate() {
        let (from, to) = (stops[i], stops[i + 1]);
        if progress <= to || i + 2 == motion.frames.len() {
            let span = to - from;
            let local = if span <= 0.0 {
                1.0
            } else {
                (progress - from) / span
            };
            return blend(&pair[0].pose, &pair[1].pose, eased(pair[0].ease, local));
        }
    }
    first.pose
}
