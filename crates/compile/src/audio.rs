use opsis_types::mix::ChannelKind;
use opsis_types::timeline::{AudioItem, AudioRole};

pub fn channel_of(a: &AudioItem) -> String {
    a.channel
        .clone()
        .or_else(|| a.node_id.clone())
        .unwrap_or_else(|| a.id.clone())
}

pub fn sound_length(a: &AudioItem, total: u32) -> f64 {
    let rest = (f64::from(total) - f64::from(a.from)).max(0.0);
    if a.role == Some(AudioRole::Effect) {
        let natural = a
            .length_frames
            .map_or(rest, |l| f64::from(l) / a.rate.unwrap_or(1.0));
        return natural.min(a.duration_in_frames.map_or(natural, f64::from));
    }
    a.duration_in_frames.map_or(rest, f64::from)
}

pub fn audio_kind(a: &AudioItem) -> ChannelKind {
    match a.role {
        Some(AudioRole::Music) => ChannelKind::Music,
        _ => ChannelKind::Effect,
    }
}
