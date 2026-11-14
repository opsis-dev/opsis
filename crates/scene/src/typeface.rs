use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlyphCommand {
    Move([f64; 2]),
    Line([f64; 2]),
    Quad {
        to: [f64; 2],
        control: [f64; 2],
    },
    Cubic {
        to: [f64; 2],
        first: [f64; 2],
        second: [f64; 2],
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub advance: f64,
    pub commands: Vec<GlyphCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Typeface {
    pub resolution: f64,
    pub glyphs: HashMap<char, Glyph>,
}

#[derive(Deserialize)]
struct RawGlyph {
    ha: f64,
    #[serde(default)]
    o: String,
}

#[derive(Deserialize)]
struct RawTypeface {
    glyphs: HashMap<String, RawGlyph>,
    resolution: f64,
}

fn commands(outline: &str) -> Vec<GlyphCommand> {
    let tokens: Vec<&str> = outline.split(' ').collect();
    let mut out = Vec::new();
    let mut i = 0;
    let pair = |i: &mut usize| {
        let x = tokens
            .get(*i)
            .and_then(|t| t.parse().ok())
            .unwrap_or(f64::NAN);
        let y = tokens
            .get(*i + 1)
            .and_then(|t| t.parse().ok())
            .unwrap_or(f64::NAN);
        *i += 2;
        [x, y]
    };
    while i < tokens.len() {
        let action = tokens[i];
        i += 1;
        match action {
            "m" => out.push(GlyphCommand::Move(pair(&mut i))),
            "l" => out.push(GlyphCommand::Line(pair(&mut i))),
            "q" => {
                let to = pair(&mut i);
                let control = pair(&mut i);
                out.push(GlyphCommand::Quad { to, control });
            }
            "b" => {
                let to = pair(&mut i);
                let first = pair(&mut i);
                let second = pair(&mut i);
                out.push(GlyphCommand::Cubic { to, first, second });
            }
            _ => {}
        }
    }
    out
}

impl Typeface {
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let raw: RawTypeface = serde_json::from_slice(bytes).ok()?;
        let glyphs = raw
            .glyphs
            .into_iter()
            .filter_map(|(key, glyph)| {
                let mut chars = key.chars();
                let ch = chars.next()?;
                chars.next().is_none().then(|| {
                    (
                        ch,
                        Glyph {
                            advance: glyph.ha,
                            commands: commands(&glyph.o),
                        },
                    )
                })
            })
            .collect();
        Some(Self {
            resolution: raw.resolution,
            glyphs,
        })
    }

    pub fn glyph(&self, ch: char) -> Option<&Glyph> {
        self.glyphs.get(&ch).or_else(|| self.glyphs.get(&'?'))
    }

    pub fn advance(&self, ch: char) -> f64 {
        self.glyphs
            .get(&ch)
            .or_else(|| self.glyphs.get(&'?'))
            .or_else(|| self.glyphs.get(&' '))
            .map_or(0.0, |g| g.advance)
    }
}
