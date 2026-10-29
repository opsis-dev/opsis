use opsis_compile::nodes::image::PlaceAlign;
use opsis_compile::nodes::media::{
    ConvertCodec, Dither, PaletteFrom, ProresProfile, ScaleMode, SoundQuality,
};
use opsis_media::codecs::{prores_args, sound_args};
use opsis_media::filters::{GifLook, fit_filter, gif_filter, retime_sound};
use opsis_types::numbers;
use proptest::prelude::*;

fn bit_rate(args: &[String]) -> u32 {
    args.iter()
        .skip_while(|a| *a != "-b:a")
        .nth(1)
        .and_then(|rate| rate.trim_end_matches('k').parse().ok())
        .unwrap_or(0)
}

#[test]
fn spends_more_bits_on_sound_as_the_quality_rises() {
    for codec in [ConvertCodec::H264, ConvertCodec::H265, ConvertCodec::Vp9] {
        let rates = [SoundQuality::Low, SoundQuality::Medium, SoundQuality::High]
            .map(|q| bit_rate(&sound_args(codec, q)));
        assert!(
            rates[0] < rates[1] && rates[1] < rates[2],
            "{codec:?}: {rates:?}"
        );
    }
    assert!(!sound_args(ConvertCodec::Prores, SoundQuality::Low).contains(&"-b:a".to_owned()));
}

#[test]
fn asks_for_a_lighter_prores_kind_with_a_lower_profile() {
    let profile = |p| {
        let args = prores_args(p);
        let at = args.iter().position(|a| a == "-profile:v").unwrap();
        args[at + 1].parse::<u32>().unwrap()
    };
    let kinds = [
        ProresProfile::Proxy,
        ProresProfile::Lt,
        ProresProfile::Standard,
        ProresProfile::Hq,
    ]
    .map(profile);
    assert!(kinds.windows(2).all(|w| w[0] < w[1]), "{kinds:?}");
}

#[test]
fn places_the_picture_at_either_edge_or_the_centre() {
    for mode in [ScaleMode::Pad, ScaleMode::Crop] {
        let [start, centre, end] = [PlaceAlign::Start, PlaceAlign::Center, PlaceAlign::End]
            .map(|a| fit_filter(mode, 1280.0, 720.0, "", a));
        assert!(start != centre && centre != end && start != end, "{mode:?}");
    }
    let stretched = [PlaceAlign::Start, PlaceAlign::End]
        .map(|a| fit_filter(ScaleMode::Stretch, 1280.0, 720.0, "", a));
    assert_eq!(stretched[0], stretched[1], "a stretch has nothing to place");
}

proptest! {
    #[test]
    fn changes_the_pitch_with_the_speed_only_when_asked(rate in 0.25..64.0f64) {
        let kept = retime_sound(rate, true);
        let shifted = retime_sound(rate, false);
        prop_assert!(kept.starts_with("atempo="));
        let pitch = format!("pitch={rate:.4}");
        prop_assert!(shifted.contains("rubberband") && shifted.contains(&pitch));
    }

    #[test]
    fn limits_the_gif_colours_to_the_number_asked(colours in 2u32..256) {
        let look = |colours, palette_from| GifLook { dither: Dither::Bayer, colours, palette_from };
        let limited = gif_filter(15.0, 640.0, &look(colours, PaletteFrom::Whole));
        let asked = format!("max_colors={colours}");
        prop_assert!(limited.contains(&asked));
        prop_assert!(limited.contains("stats_mode=full"));
        prop_assert!(!gif_filter(15.0, 640.0, &look(256, PaletteFrom::Moving)).contains("max_colors"));
    }
}

fn settings<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).expect("settings")
}

proptest! {
    #[test]
    fn repeats_the_echo_a_delay_apart_each_quieter(repeats in 1u32..8, delay in 50u32..2000) {
        let p: opsis_compile::nodes::media::VoiceEffects = settings(serde_json::json!({
            "effect": "echo", "mix": 1.0, "delayMs": delay, "decay": 0.5, "repeats": repeats
        }));
        let filter = opsis_media::filters::voice_filter(&p, None);
        let delays: Vec<u32> = filter
            .split("delays=").nth(1).unwrap()
            .split(':').next().unwrap()
            .split('|').map(|d| d.parse().unwrap()).collect();
        prop_assert_eq!(numbers::count(delays.len()), repeats);
        prop_assert!(delays.windows(2).all(|w| w[1] == w[0] + delay));
    }

    #[test]
    fn cuts_rumble_only_when_asked_and_lifts_by_the_most_asked(rumble in 0u32..300, lift in 0.0..30.0f64) {
        let p: opsis_compile::nodes::media::CleanSpeech = settings(serde_json::json!({
            "rumbleHz": rumble, "evenLift": lift, "evenLevel": true
        }));
        let filter = opsis_media::filters::speech_filter(&p, None);
        prop_assert_eq!(filter.contains("highpass"), rumble > 0);
        let expansion: f64 = filter.split("speechnorm=e=").nth(1).unwrap().split(':').next().unwrap().parse().unwrap();
        prop_assert!((20.0 * expansion.log10() - lift.max(0.0)).abs() < 0.2);
    }
}

#[test]
fn draws_channels_apart_and_on_the_scale_asked() {
    let picture = |extra: serde_json::Value| {
        let mut p = serde_json::json!({ "kind": "waveform" });
        p.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        opsis_media::filters::sound_picture_graph(&settings(p))
    };
    assert_ne!(
        picture(serde_json::json!({})),
        picture(serde_json::json!({ "splitChannels": true }))
    );
    assert_ne!(
        picture(serde_json::json!({})),
        picture(serde_json::json!({ "waveScale": "log" }))
    );
}

#[test]
fn makes_the_last_countdown_beep_as_long_and_high_as_asked() {
    use opsis_compile::nodes::audio::{ToneKind, ToneWave};
    use opsis_media::generate::{Beeps, beeps};
    let countdown = |final_length: f64, final_pitch: f64, wave: ToneWave| {
        beeps(&Beeps {
            kind: ToneKind::Countdown,
            pitch_hz: 440.0,
            length_ms: 100.0,
            count: 2,
            spacing_ms: 500.0,
            wave,
            final_length,
            final_pitch,
        })
    };
    let crossings = |samples: &[f32]| {
        samples
            .windows(2)
            .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
            .count()
    };
    let start = 2 * 24_000;
    let plain = countdown(3.0, 0.0, ToneWave::Sine);
    let longer = countdown(5.0, 0.0, ToneWave::Sine);
    assert!(longer.len() > plain.len());
    let higher = countdown(3.0, 12.0, ToneWave::Sine);
    let ratio = f64::from(numbers::count(crossings(&higher[start..])))
        / f64::from(numbers::count(crossings(&plain[start..])));
    assert!((ratio - 2.0).abs() < 0.05, "an octave up: {ratio}");
    assert_ne!(countdown(3.0, 0.0, ToneWave::Square), plain);
}

#[test]
fn moves_a_whoosh_the_way_asked() {
    use opsis_compile::nodes::audio::{GenerateSound, SoundPan, sound_shape};
    let shape = |pan| {
        sound_shape(&GenerateSound {
            pan,
            ..GenerateSound::default()
        })
    };
    let lean = |pan| {
        let [left, right] = opsis_media::sound_maker::make(&shape(pan));
        let tail = left.len() * 3 / 4;
        let power = |c: &[f32]| c[tail..].iter().map(|v| f64::from(*v).powi(2)).sum::<f64>();
        power(&right) - power(&left)
    };
    assert!(lean(SoundPan::LeftToRight) > 0.0);
    assert!(lean(SoundPan::RightToLeft) < 0.0);
    assert!(lean(SoundPan::Centre).abs() < 1e-6);
}
