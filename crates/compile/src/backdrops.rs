use opsis_macros::choice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Light,
    Dark,
}

#[choice(export)]
pub enum Backdrop {
    Glaze,
    Ink,
    Cobalt,
    Graphite,
    Ember,
    Flare,
    Prism,
    Bloom,
    Halo,
    Dusk,
    Studio,
    Moonrise,
    Canyon,
    Mist,
    Chalk,
    Aurora,
}

impl Backdrop {
    pub const ALL: [Backdrop; 16] = [
        Self::Glaze,
        Self::Ink,
        Self::Cobalt,
        Self::Graphite,
        Self::Ember,
        Self::Flare,
        Self::Prism,
        Self::Bloom,
        Self::Halo,
        Self::Dusk,
        Self::Studio,
        Self::Moonrise,
        Self::Canyon,
        Self::Mist,
        Self::Chalk,
        Self::Aurora,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Glaze => "glaze",
            Self::Ink => "ink",
            Self::Cobalt => "cobalt",
            Self::Graphite => "graphite",
            Self::Ember => "ember",
            Self::Flare => "flare",
            Self::Prism => "prism",
            Self::Bloom => "bloom",
            Self::Halo => "halo",
            Self::Dusk => "dusk",
            Self::Studio => "studio",
            Self::Moonrise => "moonrise",
            Self::Canyon => "canyon",
            Self::Mist => "mist",
            Self::Chalk => "chalk",
            Self::Aurora => "aurora",
        }
    }

    pub fn tone(self) -> Tone {
        match self {
            Self::Canyon | Self::Mist | Self::Chalk | Self::Aurora => Tone::Light,
            _ => Tone::Dark,
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.id() == name)
    }
}

pub const DEFAULT_BACKDROP: Backdrop = Backdrop::Glaze;
