#![expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pose has to match it"
)]

use std::f64::consts::TAU;

use opsis_scene::random::random;
use opsis_scene::spring::{DEFAULT_SPRING, Spring, SpringConfig, spring};
use opsis_types::timeline::CameraPath;

use crate::elements::shared::Motion;
use crate::elements::three::{Pose, view_pose};
use crate::timeline::camera_path::camera_at;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionTime {
    pub frame: f64,
    pub fps: f64,
    pub speed: f64,
    pub seed: f64,
    pub tilt_deg: f64,
    pub end_facing: bool,
    pub amount: f64,
    pub reverse: bool,
    pub grow_in: bool,
}

const SPIN_PERIOD_S: f64 = 6.0;
const ORBIT_PERIOD_S: f64 = 8.0;
const SWING_PERIOD_S: f64 = 3.2;
const ENTER_FROM: f64 = 0.9;
const ENTER_SPRING: SpringConfig = SpringConfig {
    damping: 200.0,
    ..DEFAULT_SPRING
};
const TILT_SPRING: SpringConfig = SpringConfig {
    damping: 14.0,
    mass: 0.9,
    ..DEFAULT_SPRING
};
const TILT_REST: [f64; 3] = [0.08, -0.18, 0.0];
const FACING: [f64; 3] = [0.0, 0.0, 0.0];
const TILT_DEPTH: f64 = -2.0;
const FLOAT_TILT_X: f64 = 0.08;
const FLOAT_TURN_Y: f64 = 0.25;
const FLOAT_ROLL_Z: f64 = 0.04;
const FLOAT_LIFT: f64 = 0.08;
const ORBIT_PITCH: f64 = 0.15;
const ORBIT_TURN: f64 = 0.7;
const SWING_TURN: f64 = 0.45;
const SPIN_PITCH: f64 = 0.12;

const fn still(scale: f64) -> Pose {
    Pose {
        rotation: [0.0, 0.0, 0.0],
        position: [0.0, 0.0, 0.0],
        scale,
    }
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn tilt_from(deg: f64) -> [f64; 3] {
    let rad = (deg * std::f64::consts::PI) / 180.0;
    [-rad, rad, 0.0]
}

fn sprung(frame: f64, fps: f64, config: SpringConfig) -> f64 {
    spring(&Spring {
        config,
        ..Spring::at(frame, fps)
    })
    .unwrap_or(1.0)
}

pub fn pose_at(motion: Motion, time: &MotionTime) -> Pose {
    let t = (time.frame / time.fps) * time.speed;
    let enter = sprung(time.frame, time.fps, ENTER_SPRING);
    let scale = if time.grow_in {
        ENTER_FROM + (1.0 - ENTER_FROM) * enter
    } else {
        1.0
    };
    let turn = if time.reverse { -1.0 } else { 1.0 };
    let amount = time.amount;
    match motion {
        Motion::Spin => Pose {
            rotation: [SPIN_PITCH, (turn * TAU * t) / SPIN_PERIOD_S, 0.0],
            ..still(scale)
        },
        Motion::TiltIn => {
            let s = sprung(time.frame * time.speed, time.fps, TILT_SPRING);
            Pose {
                rotation: lerp3(
                    tilt_from(time.tilt_deg),
                    if time.end_facing { FACING } else { TILT_REST },
                    s,
                ),
                position: [0.0, 0.0, (1.0 - s) * TILT_DEPTH],
                scale,
            }
        }
        Motion::Float => {
            let phase = random(time.seed) * TAU;
            Pose {
                rotation: [
                    (t * 0.9 + phase).sin() * FLOAT_TILT_X * amount,
                    (t * 0.6 + phase * 2.0).sin() * FLOAT_TURN_Y * amount * turn,
                    (t * 0.7 + phase * 3.0).sin() * FLOAT_ROLL_Z * amount,
                ],
                position: [0.0, (t * 1.1 + phase).sin() * FLOAT_LIFT * amount, 0.0],
                scale,
            }
        }
        Motion::Orbit => Pose {
            rotation: [
                ORBIT_PITCH,
                ((TAU * t) / ORBIT_PERIOD_S).sin() * ORBIT_TURN * amount * turn,
                0.0,
            ],
            ..still(scale)
        },
        Motion::Swing => Pose {
            rotation: [
                0.0,
                ((TAU * t) / SWING_PERIOD_S).sin() * SWING_TURN * amount * turn,
                0.0,
            ],
            ..still(scale)
        },
        Motion::Still | Motion::Keyframes => still(scale),
    }
}

pub fn element_pose(motion: Motion, camera: Option<&CameraPath>, time: &MotionTime) -> Pose {
    if motion == Motion::Keyframes {
        view_pose(&camera_at(camera, time.frame, false))
    } else {
        pose_at(motion, time)
    }
}
