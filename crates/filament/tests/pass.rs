use opsis_filament::{
    Backend, Camera, Filament, Input, Light, Mesh, STANDARD_MATERIAL, Scene, TINT_MATERIAL, Uniform,
};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

fn picture() -> Vec<u8> {
    let mut out = Vec::new();
    for y in 0..HEIGHT {
        for _ in 0..WIDTH {
            let top = y < HEIGHT / 2;
            out.extend_from_slice(&[if top { 200 } else { 40 }, 100, 0, 255]);
        }
    }
    out
}

fn pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
    let at = usize::try_from((y * WIDTH + x) * 4).expect("an index");
    [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]]
}

fn near(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= 1)
}

#[test]
#[ignore = "needs a Vulkan driver (a GPU, or lavapipe through VK_DRIVER_FILES)"]
fn a_tint_pass_halves_the_picture_and_keeps_it_upright() {
    let mut filament = Filament::new(Backend::Vulkan).expect("an engine");
    assert_eq!(filament.backend(), Backend::Vulkan);
    let tint = filament.material(TINT_MATERIAL).expect("the tint material");
    let out = filament
        .pass(
            tint,
            &[Input {
                pixels: &picture(),
                width: WIDTH,
                height: HEIGHT,
            }],
            (WIDTH, HEIGHT),
            &[Uniform {
                name: "amount",
                values: &[0.5],
                columns: 1,
            }],
        )
        .expect("a pass");
    let (top, bottom) = (pixel(&out, 10, 4), pixel(&out, 10, 28));
    assert!(near(top, [100, 50, 0, 255]), "{top:?}");
    assert!(near(bottom, [20, 50, 0, 255]), "{bottom:?}");
}

#[test]
fn a_picture_of_the_wrong_size_is_refused() {
    let Ok(mut filament) = Filament::new(Backend::Vulkan) else {
        return;
    };
    let tint = filament.material(TINT_MATERIAL).expect("the tint material");
    assert!(
        filament
            .pass(
                tint,
                &[Input {
                    pixels: &[0; 12],
                    width: WIDTH,
                    height: HEIGHT,
                }],
                (WIDTH, HEIGHT),
                &[Uniform {
                    name: "amount",
                    values: &[1.0],
                    columns: 1,
                }],
            )
            .is_err()
    );
}

fn sphere(rings: u32, segments: u32) -> (Vec<f32>, Vec<u32>) {
    let mut points = Vec::new();
    for ring in 0..=rings {
        let v = f64::from(ring) / f64::from(rings) * std::f64::consts::PI;
        for segment in 0..=segments {
            let u = f64::from(segment) / f64::from(segments) * std::f64::consts::TAU;
            for c in [v.sin() * u.cos(), v.cos(), v.sin() * u.sin()] {
                points.push(opsis_types_narrow(c));
            }
        }
    }
    let mut indices = Vec::new();
    for ring in 0..rings {
        for segment in 0..segments {
            let a = ring * (segments + 1) + segment;
            let b = a + segments + 1;
            indices.extend([a, a + 1, b, b, a + 1, b + 1]);
        }
    }
    (points, indices)
}

fn opsis_types_narrow(value: f64) -> f32 {
    format!("{value}").parse().expect("a float")
}

const IDENTITY: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

#[test]
#[ignore = "needs a Vulkan driver (a GPU, or lavapipe through VK_DRIVER_FILES)"]
fn a_lit_sphere_is_brighter_towards_its_light() {
    let mut filament = Filament::new(Backend::Vulkan).expect("an engine");
    let standard = filament
        .material(STANDARD_MATERIAL)
        .expect("the standard material");
    let (points, indices) = sphere(24, 48);
    let uniforms = [
        Uniform {
            name: "baseColor",
            values: &[0.8, 0.2, 0.2, 1.0],
            columns: 4,
        },
        Uniform {
            name: "metallic",
            values: &[0.0],
            columns: 1,
        },
        Uniform {
            name: "roughness",
            values: &[0.5],
            columns: 1,
        },
    ];
    let meshes = [Mesh {
        positions: &points,
        normals: &points,
        uvs: None,
        indices: &indices,
        transform: IDENTITY,
        material: standard,
        uniforms: &uniforms,
        textures: &[],
    }];
    let lights = [Light {
        direction: [-1.0, 0.0, -1.0],
        color: [1.0, 1.0, 1.0],
        intensity: 3.0,
    }];
    let scene = Scene {
        camera: Camera {
            fov_degrees: 30.0,
            near: 0.1,
            far: 20.0,
            eye: [0.0, 0.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
        },
        meshes: &meshes,
        lights: &lights,
        environment: None,
        samples: 4,
    };
    let out = filament.scene(&scene, (WIDTH, HEIGHT)).expect("a picture");
    let (lit, dark) = (pixel(&out, 37, 16), pixel(&out, 27, 16));
    assert!(lit[0] > dark[0], "{lit:?} {dark:?}");
    assert_eq!(pixel(&out, 1, 1)[3], 0, "the background stays clear");
}
