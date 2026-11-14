use std::path::Path;

use opsis_types::numbers::float;

use crate::{WRITE, path, writing};

const FLIP_TOLERANCE: f32 = 0.05;

#[derive(Debug, Clone, Copy)]
pub struct Picture<'a> {
    pub png: &'a [u8],
    pub rgba: &'a [u8],
    pub size: (u32, u32),
}

pub fn exact_setup() -> bool {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
    {
        false
    }
}

fn rgb(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|[r, g, b, _]| [*r, *g, *b])
        .collect()
}

fn differing_share(a: &[u8], b: &[u8]) -> f64 {
    let pixels = a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0);
    let (mut differing, mut all) = (0usize, 0usize);
    for (p, q) in pixels {
        all += 1;
        differing += usize::from(p != q);
    }
    float(differing) / float(all.max(1))
}

pub fn compared(key: &str, png: &[u8], made: Picture<'_>, allowed_share: f64) -> Option<String> {
    let recorded = image::load_from_memory(png)
        .unwrap_or_else(|e| panic!("{key}: {e}"))
        .to_rgba8();
    if recorded.dimensions() != made.size {
        return Some(format!(
            "{key}: size {:?} against {:?}",
            recorded.dimensions(),
            made.size
        ));
    }
    if exact_setup() {
        let share = differing_share(recorded.as_raw(), made.rgba);
        return (share > allowed_share)
            .then(|| format!("{key}: {share:.5} of its pixels differ from its golden"));
    }
    let (width, height) = made.size;
    let a = nv_flip::FlipImageRgb8::with_data(width, height, &rgb(recorded.as_raw()));
    let b = nv_flip::FlipImageRgb8::with_data(width, height, &rgb(made.rgba));
    let mean =
        nv_flip::FlipPool::from_image(&nv_flip::flip(a, b, nv_flip::DEFAULT_PIXELS_PER_DEGREE))
            .mean();
    (mean > FLIP_TOLERANCE).then(|| format!("{key}: mean {mean:.4} above {FLIP_TOLERANCE}"))
}

pub fn check_picture(
    name: &str,
    made: Picture<'_>,
    allowed_share: f64,
    out: &Path,
) -> Option<String> {
    let file = path(name);
    if writing() {
        assert!(
            exact_setup(),
            "golden pictures are recorded on x86 Linux with AVX2"
        );
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        }
        std::fs::write(&file, made.png).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        return None;
    }
    let mine = out.join(name);
    if let Some(dir) = mine.parent() {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    }
    std::fs::write(&mine, made.png).unwrap_or_else(|e| panic!("{}: {e}", mine.display()));
    let Ok(recorded) = std::fs::read(&file) else {
        return Some(format!("{name}: no golden; record it with {WRITE}=1"));
    };
    compared(name, &recorded, made, allowed_share)
}
