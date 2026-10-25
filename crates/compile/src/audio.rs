use opsis_types::mix::ChannelKind;
use opsis_types::timeline::{AudioItem, AudioRole};

pub fn channel_of(a: &AudioItem) -> String {
    a.channel
        .clone()
        .or_else(|| a.node_id.clone())
        .unwrap_or_else(|| a.id.clone())
}

pub fn sound_length(a: &AudioItem, total: f64) -> f64 {
    let rest = (total - a.from).max(0.0);
    if a.role == Some(AudioRole::Effect) {
        let natural = a.length_frames.map_or(rest, |l| l / a.rate.unwrap_or(1.0));
        return natural.min(a.duration_in_frames.unwrap_or(natural));
    }
    a.duration_in_frames.unwrap_or(rest)
}

pub fn audio_kind(a: &AudioItem) -> ChannelKind {
    match a.role {
        Some(AudioRole::Music) => ChannelKind::Music,
        _ => ChannelKind::Effect,
    }
}
