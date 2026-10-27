use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinSound {
    pub id: String,
    pub label: String,
    pub duration_ms: u32,
}

impl BuiltinSound {
    pub fn reference(&self) -> String {
        format!("app:sfx/{}.wav", self.id)
    }
}

static SOUNDS: LazyLock<Vec<BuiltinSound>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("data/sounds.json")).expect("built-in sound list parses")
});

pub fn builtin_sounds() -> &'static [BuiltinSound] {
    &SOUNDS
}

pub fn builtin_sound(reference: &str) -> Option<&'static BuiltinSound> {
    SOUNDS.iter().find(|s| s.reference() == reference)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    Calm,
    Upbeat,
    Ambient,
}

impl Mood {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Calm => "Calm",
            Self::Upbeat => "Upbeat",
            Self::Ambient => "Ambient",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinSong {
    pub id: String,
    pub label: String,
    pub mood: Mood,
    pub duration_ms: u32,
}

impl BuiltinSong {
    pub fn reference(&self) -> String {
        format!("app:music/{}.m4a", self.id)
    }
}

static SONGS: LazyLock<Vec<BuiltinSong>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("data/music.json")).expect("built-in music list parses")
});

pub fn builtin_songs() -> &'static [BuiltinSong] {
    &SONGS
}

pub fn builtin_song(reference: &str) -> Option<&'static BuiltinSong> {
    SONGS.iter().find(|s| s.reference() == reference)
}

pub fn builtin_label(reference: &str) -> Option<&'static str> {
    builtin_sound(reference)
        .map(|s| s.label.as_str())
        .or_else(|| builtin_song(reference).map(|s| s.label.as_str()))
}
