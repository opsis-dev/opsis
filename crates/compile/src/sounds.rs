use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinSound {
    pub id: String,
    pub label: String,
    pub duration_ms: f64,
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
