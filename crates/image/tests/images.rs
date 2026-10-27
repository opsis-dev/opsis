use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{
    DynamicImage, GenericImageView, ImageEncoder, ImageFormat as FileFormat, Rgb, RgbImage, Rgba,
    RgbaImage,
};
use opsis_compile::nodes::effects::Area;
use opsis_compile::nodes::image::{
    AdjustImage, AutoLevels, BlurImage, ClearColour, CropImage, CropMode, ImageFormat,
    LevelsMethod, OutlineImage, PlaceImage, RedactImage, RedactStyle, ResizeFit, ResizeImage,
    RotateImage, SpotlightImage, StillFileFormat, TiltImage, TrimEdges, Turn, tilt_corners,
    tilted_size,
};
use opsis_image::load::load_image;
use opsis_image::ops;

const CELL: u32 = 16;
const COLOURS: [[u8; 3]; 6] = [
    [230, 30, 30],
    [30, 200, 30],
    [30, 30, 230],
    [230, 230, 30],
    [30, 230, 230],
    [230, 30, 230],
];
const JPEG_SLACK: u8 = 12;

fn upright() -> RgbImage {
    RgbImage::from_fn(3 * CELL, 2 * CELL, |x, y| {
        Rgb(COLOURS[((y / CELL) * 3 + x / CELL) as usize])
    })
}

fn jpeg(image: &DynamicImage) -> Vec<u8> {
    let mut out = Vec::new();
    image
        .write_with_encoder(JpegEncoder::new_with_quality(&mut out, 100))
        .unwrap();
    out
}

fn with_orientation(jpeg: &[u8], orientation: u16) -> Vec<u8> {
    let mut tiff = b"MM\0*\0\0\0\x08".to_vec();
    tiff.extend_from_slice(&1u16.to_be_bytes());
    tiff.extend_from_slice(&0x0112u16.to_be_bytes());
    tiff.extend_from_slice(&3u16.to_be_bytes());
    tiff.extend_from_slice(&1u32.to_be_bytes());
    tiff.extend_from_slice(&orientation.to_be_bytes());
    tiff.extend_from_slice(&[0, 0]);
    tiff.extend_from_slice(&0u32.to_be_bytes());
    let mut segment = b"Exif\0\0".to_vec();
    segment.extend_from_slice(&tiff);
    let mut out = jpeg[..2].to_vec();
    out.extend_from_slice(&[0xFF, 0xE1]);
    out.extend_from_slice(&u16::try_from(segment.len() + 2).unwrap().to_be_bytes());
    out.extend_from_slice(&segment);
    out.extend_from_slice(&jpeg[2..]);
    out
}

fn stored_for(orientation: u16, upright: DynamicImage) -> DynamicImage {
    match orientation {
        1 => upright,
        2 => upright.fliph(),
        3 => upright.rotate180(),
        4 => upright.flipv(),
        5 => upright.rotate90().fliph(),
        6 => upright.rotate270(),
        7 => upright.rotate90().flipv(),
        8 => upright.rotate90(),
        _ => unreachable!(),
    }
}

fn near(a: [u8; 3], b: [u8; 3], slack: u8) -> bool {
    a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= slack)
}

#[test]
fn turns_every_exif_orientation_upright() {
    for orientation in 1..=8 {
        let stored = stored_for(orientation, DynamicImage::ImageRgb8(upright()));
        let file = with_orientation(&jpeg(&stored), orientation);
        let loaded = load_image(&file).unwrap().to_rgb8();
        assert_eq!(
            loaded.dimensions(),
            (3 * CELL, 2 * CELL),
            "orientation {orientation}"
        );
        for (i, colour) in COLOURS.iter().enumerate() {
            let (x, y) = (
                (i as u32 % 3) * CELL + CELL / 2,
                (i as u32 / 3) * CELL + CELL / 2,
            );
            let got = loaded.get_pixel(x, y).0;
            assert!(
                near(got, *colour, JPEG_SLACK),
                "orientation {orientation} cell {i}: {got:?} against {colour:?}"
            );
        }
    }
}

fn decode_srgb(v: f64) -> f64 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn encode_srgb(v: f64) -> f64 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

fn p3_to_srgb(p3: [u8; 3]) -> [u8; 3] {
    let m = [
        [1.224_940_1, -0.224_940_4, 0.0],
        [-0.042_056_9, 1.042_057_1, 0.0],
        [-0.019_637_6, -0.078_636_1, 1.098_273_5],
    ];
    let linear = p3.map(|v| decode_srgb(f64::from(v) / 255.0));
    [0, 1, 2].map(|i| {
        let mixed = m[i][0] * linear[0] + m[i][1] * linear[1] + m[i][2] * linear[2];
        (encode_srgb(mixed) * 255.0).round() as u8
    })
}

#[test]
fn turns_wide_colour_into_srgb_like_a_reference_conversion() {
    let p3 = [204, 128, 77];
    let block = RgbImage::from_pixel(16, 16, Rgb(p3));
    let profile = moxcms::ColorProfile::new_display_p3().encode().unwrap();
    let mut file = Vec::new();
    let mut encoder = PngEncoder::new(&mut file);
    encoder.set_icc_profile(profile).unwrap();
    encoder
        .write_image(block.as_raw(), 16, 16, image::ExtendedColorType::Rgb8)
        .unwrap();
    let loaded = load_image(&file).unwrap().to_rgb8();
    let want = p3_to_srgb(p3);
    let got = loaded.get_pixel(8, 8).0;
    assert!(near(got, want, 2), "{got:?} against {want:?}");
    assert_ne!(got, p3);
}

fn sample() -> DynamicImage {
    DynamicImage::ImageRgb8(upright())
}

#[test]
fn resizes_to_the_planned_size() {
    let image = sample();
    let resize = |width, height, fit| ResizeImage {
        width,
        height,
        fit,
        ..ResizeImage::default()
    };
    let sized = |p: &ResizeImage| {
        let out = ops::resize(&image, p).unwrap();
        (out.width(), out.height())
    };
    assert_eq!(sized(&resize(24, 0, ResizeFit::Inside)), (24, 16));
    assert_eq!(sized(&resize(0, 64, ResizeFit::Inside)), (96, 64));
    assert_eq!(sized(&resize(40, 40, ResizeFit::Inside)), (40, 27));
    assert_eq!(sized(&resize(40, 40, ResizeFit::Cover)), (40, 40));
    assert_eq!(sized(&resize(10, 50, ResizeFit::Exact)), (10, 50));
    let cover = ops::resize(&image, &resize(32, 32, ResizeFit::Cover))
        .unwrap()
        .to_rgb8();
    assert!(
        near(cover.get_pixel(16, 8).0, COLOURS[1], 40),
        "cover keeps the middle column"
    );
}

#[test]
fn crops_an_area_or_margins_and_keeps_inside_the_image() {
    let image = sample();
    let area = CropImage {
        area: vec![Area {
            x: 1.0 / 3.0,
            y: 0.0,
            width: 1.0 / 3.0,
            height: 0.5,
        }],
        ..CropImage::default()
    };
    let out = ops::crop(&image, &area).to_rgb8();
    assert_eq!(out.dimensions(), (CELL, CELL));
    assert_eq!(out.get_pixel(8, 8).0, COLOURS[1]);
    let margins = CropImage {
        mode: CropMode::Margins,
        top: 16,
        left: 32,
        right: 0,
        bottom: 0,
        ..CropImage::default()
    };
    let out = ops::crop(&image, &margins).to_rgb8();
    assert_eq!(
        (out.dimensions(), out.get_pixel(0, 0).0),
        ((CELL, CELL), COLOURS[5])
    );
    let beyond = CropImage {
        mode: CropMode::Margins,
        top: 900,
        left: 900,
        right: 900,
        bottom: 900,
        ..CropImage::default()
    };
    assert_eq!(ops::crop(&image, &beyond).dimensions(), (1, 1));
}

#[test]
fn turns_flips_and_straightens_onto_a_larger_canvas() {
    let image = sample();
    let rotate = |turn, h, v, angle| RotateImage {
        turn,
        flip_horizontal: h,
        flip_vertical: v,
        angle,
        background: "#ffffff".into(),
    };
    let right = ops::rotate(&image, &rotate(Turn::Right, false, false, 0.0)).to_rgb8();
    assert_eq!(right.dimensions(), (2 * CELL, 3 * CELL));
    assert_eq!(right.get_pixel(2 * CELL - 1, 0).0, COLOURS[0]);
    let flipped = ops::rotate(&image, &rotate(Turn::None, true, false, 0.0)).to_rgb8();
    assert_eq!(flipped.get_pixel(0, 0).0, COLOURS[2]);
    let tilted = ops::rotate(&image, &rotate(Turn::None, false, false, 30.0)).to_rgba8();
    assert!(tilted.width() > 3 * CELL && tilted.height() > 2 * CELL);
    assert_eq!(tilted.get_pixel(0, 0).0, [255, 255, 255, 255]);
}

#[test]
fn adjusts_colour_and_leaves_it_alone_at_zero() {
    let image = sample();
    assert_eq!(
        ops::adjust(&image, &AdjustImage::default()).to_rgb8(),
        image.to_rgb8()
    );
    let grey = ops::adjust(
        &image,
        &AdjustImage {
            grayscale: true,
            ..AdjustImage::default()
        },
    )
    .to_rgb8();
    let [r, g, b] = grey.get_pixel(0, 0).0;
    assert!(r == g && g == b);
    let inverted = ops::adjust(
        &image,
        &AdjustImage {
            invert: true,
            ..AdjustImage::default()
        },
    )
    .to_rgb8();
    assert_eq!(inverted.get_pixel(0, 0).0, COLOURS[0].map(|v| 255 - v));
    let brighter = ops::adjust(
        &image,
        &AdjustImage {
            brightness: 0.5,
            ..AdjustImage::default()
        },
    )
    .to_rgb8();
    assert!(brighter.get_pixel(0, 0).0[1] > COLOURS[0][1]);
}

#[test]
fn blurs_and_sharpens_without_changing_size() {
    let image = sample();
    let none = BlurImage {
        blur: 0.0,
        sharpen: 0.0,
        threshold: 0,
    };
    assert_eq!(ops::blur(&image, &none).to_rgb8(), image.to_rgb8());
    let soft = ops::blur(
        &image,
        &BlurImage {
            blur: 6.0,
            sharpen: 0.0,
            threshold: 0,
        },
    )
    .to_rgb8();
    assert_eq!(soft.dimensions(), image.to_rgb8().dimensions());
    assert_ne!(
        soft.get_pixel(CELL, CELL / 2).0,
        image.to_rgb8().get_pixel(CELL, CELL / 2).0
    );
}

#[test]
fn places_an_image_centred_on_the_spot_at_its_size_and_opacity() {
    let base = DynamicImage::ImageRgba8(RgbaImage::from_pixel(100, 50, Rgba([0, 0, 0, 255])));
    let mark = DynamicImage::ImageRgba8(RgbaImage::from_pixel(10, 10, Rgba([255, 255, 255, 255])));
    let p = PlaceImage {
        x: 0.5,
        y: 0.5,
        size: 20.0,
        opacity: 0.5,
    };
    let out = ops::place(&base, &mark, &p).unwrap().to_rgba8();
    assert_eq!(out.dimensions(), (100, 50));
    let middle = out.get_pixel(50, 25).0;
    assert!((120..=135).contains(&middle[0]), "{middle:?}");
    assert_eq!(out.get_pixel(5, 5).0, [0, 0, 0, 255]);
    assert_eq!(out.get_pixel(39, 25).0, [0, 0, 0, 255]);
    assert_ne!(out.get_pixel(41, 25).0, [0, 0, 0, 255]);
}

#[test]
fn saves_every_format_so_it_opens_again() {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(8, 8, Rgba([200, 40, 40, 0])));
    for (format, file_format) in [
        (StillFileFormat::Png, FileFormat::Png),
        (StillFileFormat::Jpeg, FileFormat::Jpeg),
        (StillFileFormat::Webp, FileFormat::WebP),
        (StillFileFormat::Avif, FileFormat::Avif),
    ] {
        let p = ImageFormat {
            format,
            ..ImageFormat::default()
        };
        let bytes = ops::encode(&image, Some(&p)).unwrap();
        assert_eq!(
            image::guess_format(&bytes).unwrap(),
            file_format,
            "{format:?}"
        );
        if format != StillFileFormat::Avif {
            let back = image::load(Cursor::new(&bytes), file_format).unwrap();
            assert_eq!(back.dimensions(), (8, 8));
            if format == StillFileFormat::Jpeg {
                assert!(
                    near(back.to_rgb8().get_pixel(4, 4).0, [255, 255, 255], 3),
                    "see-through turns to the background"
                );
            }
        }
    }
}

fn noise(width: u32, height: u32) -> RgbaImage {
    RgbaImage::from_fn(width, height, |x, y| {
        let v = ((x * 37 + y * 91) % 256) as u8;
        Rgba([v, v.wrapping_mul(3), 255 - v, 255])
    })
}

#[test]
fn hides_only_inside_its_areas_in_every_style_and_depth() {
    let area = Area {
        x: 0.25,
        y: 0.25,
        width: 0.5,
        height: 0.5,
    };
    let inside = |x: u32, y: u32| (16..48).contains(&x) && (16..48).contains(&y);
    let shallow = DynamicImage::ImageRgba8(noise(64, 64));
    let deep = DynamicImage::ImageRgba16(shallow.to_rgba16());
    for style in [RedactStyle::Pixelate, RedactStyle::Blur, RedactStyle::Solid] {
        let p = RedactImage {
            areas: vec![area.clone()],
            style,
            block_size: 8,
            blur: 6.0,
            color: "#ff00ff".into(),
        };
        for image in [&shallow, &deep] {
            let out = ops::redact(image, &p);
            assert_eq!(out.color(), image.color(), "{style:?} keeps the depth");
            let (before, after) = (image.to_rgba8(), out.to_rgba8());
            let mut changed = 0;
            for (x, y, pixel) in after.enumerate_pixels() {
                if inside(x, y) {
                    changed += usize::from(pixel != before.get_pixel(x, y));
                } else {
                    assert_eq!(pixel, before.get_pixel(x, y), "{style:?} at {x}, {y}");
                }
            }
            assert!(changed > 32 * 32 * 9 / 10, "{style:?} changed {changed}");
        }
    }
    let pixelated = ops::redact(
        &shallow,
        &RedactImage {
            areas: vec![area.clone()],
            block_size: 8,
            ..RedactImage::default()
        },
    )
    .to_rgba8();
    assert_eq!(pixelated.get_pixel(16, 16), pixelated.get_pixel(23, 23));
    let covered = ops::redact(
        &shallow,
        &RedactImage {
            areas: vec![area],
            style: RedactStyle::Solid,
            color: "#ff00ff".into(),
            ..RedactImage::default()
        },
    )
    .to_rgba8();
    assert_eq!(covered.get_pixel(30, 30).0, [255, 0, 255, 255]);
}

#[test]
fn keeps_lit_areas_and_darkens_the_rest_in_every_depth() {
    let grey = RgbaImage::from_pixel(64, 64, Rgba([200, 200, 200, 255]));
    let p = SpotlightImage {
        areas: vec![Area {
            x: 0.25,
            y: 0.25,
            width: 0.5,
            height: 0.5,
        }],
        dim: 0.5,
        blur: 0.0,
        radius: 8.0,
        softness: 4.0,
    };
    for image in [
        DynamicImage::ImageRgba8(grey.clone()),
        DynamicImage::ImageRgba16(DynamicImage::ImageRgba8(grey.clone()).to_rgba16()),
        DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(grey.clone()).to_rgb8()),
    ] {
        let out = ops::spotlight(&image, &p);
        assert_eq!(out.color(), image.color());
        let out = out.to_rgba8();
        assert_eq!(
            out.get_pixel(32, 32).0,
            [200, 200, 200, 255],
            "the middle stays"
        );
        assert_eq!(
            out.get_pixel(2, 2).0,
            [100, 100, 100, 255],
            "far outside is half as bright"
        );
        assert!(
            out.get_pixel(17, 17).0[0] < 200,
            "a rounded corner is darkened"
        );
        let edge = out.get_pixel(32, 15).0[0];
        assert!(edge > 100 && edge < 200, "the edge is soft: {edge}");
    }
}

#[test]
fn trims_even_margins_within_the_tolerance_and_keeps_padding() {
    let framed = RgbaImage::from_fn(64, 48, |x, y| {
        if (10..54).contains(&x) && (12..40).contains(&y) {
            Rgba([20, 60, 200, 255])
        } else {
            let wobble = ((x + y) % 3) as u8;
            Rgba([250 - wobble, 250, 250 + wobble / 2, 255])
        }
    });
    let image = DynamicImage::ImageRgba8(framed);
    let trim = |tolerance, padding| ops::trim(&image, &TrimEdges { tolerance, padding });
    assert_eq!(trim(8, 0).dimensions(), (44, 28));
    assert_eq!(trim(8, 0).to_rgba8().get_pixel(0, 0).0, [20, 60, 200, 255]);
    assert_eq!(trim(8, 4).dimensions(), (52, 36));
    assert_eq!(trim(8, 100).dimensions(), (64, 48));
    assert_eq!(
        trim(0, 0).dimensions(),
        (64, 48),
        "the wobble counts as content"
    );
    let blank = DynamicImage::ImageRgba8(RgbaImage::from_pixel(10, 10, Rgba([255, 255, 255, 255])));
    assert_eq!(
        ops::trim(&blank, &TrimEdges::default()).dimensions(),
        (10, 10)
    );
}

#[test]
fn clears_the_corner_colour_from_the_edges_and_keeps_enclosed_areas() {
    let logo = RgbaImage::from_fn(40, 40, |x, y| {
        let ring = (10..30).contains(&x) && (10..30).contains(&y);
        let hole = (16..24).contains(&x) && (16..24).contains(&y);
        if ring && !hole {
            Rgba([200, 30, 30, 255])
        } else {
            Rgba([255, 255, 255, 255])
        }
    });
    let image = DynamicImage::ImageRgba8(logo);
    let clear = |edges_only| {
        ops::clear_colour(
            &image,
            &ClearColour {
                edges_only,
                ..ClearColour::default()
            },
        )
        .to_rgba8()
    };
    let edges = clear(true);
    assert_eq!(edges.get_pixel(0, 0).0[3], 0, "the corner is see-through");
    assert_eq!(
        edges.get_pixel(20, 20).0[3],
        255,
        "the white inside the ring stays"
    );
    assert_eq!(
        edges.get_pixel(12, 12).0,
        [200, 30, 30, 255],
        "the ring stays"
    );
    let everywhere = clear(false);
    assert_eq!(
        everywhere.get_pixel(20, 20).0[3],
        0,
        "the white inside goes too"
    );
    let deep = DynamicImage::ImageRgb16(image.to_rgb16());
    assert_eq!(
        ops::clear_colour(&deep, &ClearColour::default()).color(),
        image::ColorType::Rgba16
    );
    let picked = ops::clear_colour(
        &image,
        &ClearColour {
            color: "#c81e1e".into(),
            edges_only: false,
            ..ClearColour::default()
        },
    )
    .to_rgba8();
    assert_eq!(picked.get_pixel(12, 12).0[3], 0, "the picked colour goes");
    assert_eq!(picked.get_pixel(0, 0).0[3], 255, "white stays");
}

#[test]
fn draws_an_outline_only_where_the_picture_was_see_through() {
    let cutout = RgbaImage::from_fn(20, 20, |x, y| {
        if (5..15).contains(&x) && (5..15).contains(&y) {
            Rgba([10, 120, 30, 255])
        } else {
            Rgba([0, 0, 0, 0])
        }
    });
    let image = DynamicImage::ImageRgba8(cutout);
    assert!(ops::has_see_through(&image));
    let p = OutlineImage {
        width: 3.0,
        color: "#ffffff".into(),
        softness: 0.0,
        shadow: false,
    };
    let out = ops::outline(&image, &p).to_rgba8();
    assert_eq!(
        out.dimensions(),
        (26, 26),
        "the canvas grows by the outline"
    );
    assert_eq!(
        out.get_pixel(13, 13).0,
        [10, 120, 30, 255],
        "the picture stays"
    );
    assert_eq!(
        out.get_pixel(6, 13).0,
        [255, 255, 255, 255],
        "the outline is beside it"
    );
    assert_eq!(out.get_pixel(1, 1).0[3], 0, "far away stays see-through");
    let shadowed = ops::outline(
        &image,
        &OutlineImage {
            shadow: true,
            ..p.clone()
        },
    )
    .to_rgba8();
    assert_eq!(shadowed.dimensions(), (38, 38));
    let below = shadowed.get_pixel(19, 19 + 10).0;
    assert!(
        below[3] > 0 && below[0] < 50,
        "a dark shadow falls below: {below:?}"
    );
    let opaque = DynamicImage::ImageRgb8(RgbImage::new(4, 4));
    assert!(!ops::has_see_through(&opaque));
}

#[test]
fn tilts_the_corners_to_where_the_plan_puts_them() {
    let red = DynamicImage::ImageRgba8(RgbaImage::from_pixel(100, 60, Rgba([220, 20, 20, 255])));
    let p = TiltImage {
        turn: 25.0,
        tilt: 10.0,
        lean: 0.0,
        background: "#00000000".into(),
        shadow: false,
    };
    let out = ops::tilt(&red, &p).unwrap().to_rgba8();
    let (w, h) = tilted_size((100.0, 60.0), &p);
    assert_eq!(out.dimensions(), (w as u32, h as u32));
    let corners = tilt_corners((100.0, 60.0), &p);
    let centre = corners
        .iter()
        .fold((0.0, 0.0), |a, c| (a.0 + c.0 / 4.0, a.1 + c.1 / 4.0));
    let at = |x: f64, y: f64| out.get_pixel(x as u32, y as u32).0;
    assert_eq!(at(centre.0, centre.1), [220, 20, 20, 255]);
    for (x, y) in corners {
        let inside = (x + (centre.0 - x) * 0.08, y + (centre.1 - y) * 0.08);
        assert_eq!(
            at(inside.0, inside.1)[3],
            255,
            "inside the corner at {x}, {y}"
        );
    }
    let right_height = corners[2].1 - corners[1].1;
    let left_height = corners[3].1 - corners[0].1;
    assert!(right_height < left_height, "the right side turns away");
    let top_width = corners[1].0 - corners[0].0;
    let bottom_width = corners[2].0 - corners[3].0;
    assert!(top_width < bottom_width, "the top leans back");
    let shadowed = ops::tilt(&red, &TiltImage { shadow: true, ..p }).unwrap();
    assert!(shadowed.width() > out.width());
}

#[test]
fn stretches_dull_levels_to_black_and_white() {
    let dull = RgbImage::from_fn(64, 16, |x, _| {
        let v = 80 + (x * 2) as u8;
        Rgb([v, v, v + 10])
    });
    let image = DynamicImage::ImageRgb8(dull);
    let range = |out: &DynamicImage| {
        let values: Vec<u8> = out.to_rgb8().pixels().flat_map(|p| p.0).collect();
        (*values.iter().min().unwrap(), *values.iter().max().unwrap())
    };
    for method in [LevelsMethod::Stretch, LevelsMethod::Equalize] {
        let out = ops::auto_levels(
            &image,
            &AutoLevels {
                method,
                clip: 0.0,
                ..AutoLevels::default()
            },
        );
        assert_eq!(out.color(), image.color());
        let (low, high) = range(&out);
        assert!(low < 8 && high > 247, "{method:?} spans {low}..{high}");
    }
    let untouched = ops::auto_levels(
        &image,
        &AutoLevels {
            strength: 0.0,
            ..AutoLevels::default()
        },
    );
    assert_eq!(range(&untouched), (80, 216));
    let cast = ops::auto_levels(
        &image,
        &AutoLevels {
            clip: 0.0,
            per_channel: true,
            ..AutoLevels::default()
        },
    )
    .to_rgb8();
    let last = cast.get_pixel(63, 0).0;
    assert!(
        last[0] > 250 && last[2] > 250,
        "each colour reaches white: {last:?}"
    );
}
