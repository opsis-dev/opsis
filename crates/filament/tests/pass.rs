use opsis_filament::{Backend, Filament, TINT_MATERIAL, Uniform};

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
            &[&picture()],
            (WIDTH, HEIGHT),
            &[Uniform {
                name: "amount",
                values: &[0.5],
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
                &[&[0; 12]],
                (WIDTH, HEIGHT),
                &[Uniform {
                    name: "amount",
                    values: &[1.0],
                }],
            )
            .is_err()
    );
}
