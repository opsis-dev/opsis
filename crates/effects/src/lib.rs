use std::collections::HashMap;

use opsis_filament::{Backend, Filament, FilamentError, Input, MaterialId, Uniform};

include!(concat!(env!("OUT_DIR"), "/materials.rs"));

const PAPER_NOISE: &[u8] = include_bytes!("../textures/paper-noise.png");
const NOISE_PROGRAMS: [&str; 4] = [
    "shader-warp",
    "shader-god-rays",
    "shader-grain-gradient",
    "shader-smoke-ring",
];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EffectsError {
    #[error("there's no effect program named {0}")]
    Unknown(String),
    #[error("the noise texture couldn't be read")]
    Noise,
    #[error(transparent)]
    Filament(#[from] FilamentError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Texture {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Debug)]
pub struct Effects {
    filament: Filament,
    loaded: HashMap<String, MaterialId>,
    noise: Option<Texture>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub name: String,
    pub values: Vec<f32>,
    pub columns: usize,
}

pub fn has_program(program: &str) -> bool {
    MATERIALS.iter().any(|(name, _)| *name == program)
}

fn read_noise() -> Result<Texture, EffectsError> {
    let image = image::load_from_memory_with_format(PAPER_NOISE, image::ImageFormat::Png)
        .map_err(|_| EffectsError::Noise)?
        .to_rgba8();
    Ok(Texture {
        width: image.width(),
        height: image.height(),
        pixels: image.into_raw(),
    })
}

impl Effects {
    pub fn new(backend: Backend) -> Result<Self, EffectsError> {
        Ok(Self {
            filament: Filament::new(backend)?,
            loaded: HashMap::new(),
            noise: None,
        })
    }

    fn material(&mut self, program: &str) -> Result<MaterialId, EffectsError> {
        if let Some(id) = self.loaded.get(program) {
            return Ok(*id);
        }
        let (_, package) = MATERIALS
            .iter()
            .find(|(name, _)| *name == program)
            .ok_or_else(|| EffectsError::Unknown(program.to_owned()))?;
        let id = self.filament.material(package)?;
        self.loaded.insert(program.to_owned(), id);
        Ok(id)
    }

    pub fn run(
        &mut self,
        program: &str,
        inputs: &[&[u8]],
        (width, height): (u32, u32),
        settings: &[Setting],
    ) -> Result<Vec<u8>, EffectsError> {
        let material = self.material(program)?;
        if NOISE_PROGRAMS.contains(&program) && self.noise.is_none() {
            self.noise = Some(read_noise()?);
        }
        let mut pictures: Vec<Input<'_>> = inputs
            .iter()
            .map(|pixels| Input {
                pixels,
                width,
                height,
            })
            .collect();
        if NOISE_PROGRAMS.contains(&program)
            && let Some(noise) = &self.noise
        {
            pictures.push(Input {
                pixels: &noise.pixels,
                width: noise.width,
                height: noise.height,
            });
        }
        let uniforms: Vec<Uniform<'_>> = settings
            .iter()
            .map(|s| Uniform {
                name: &s.name,
                values: &s.values,
                columns: s.columns,
            })
            .collect();
        Ok(self
            .filament
            .pass(material, &pictures, (width, height), &uniforms)?)
    }
}
