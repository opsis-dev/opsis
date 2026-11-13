use std::collections::HashMap;

use opsis_filament::{Backend, Filament, FilamentError, MaterialId, Uniform};

include!(concat!(env!("OUT_DIR"), "/materials.rs"));

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EffectsError {
    #[error("there's no effect program named {0}")]
    Unknown(String),
    #[error(transparent)]
    Filament(#[from] FilamentError),
}

#[derive(Debug)]
pub struct Effects {
    filament: Filament,
    loaded: HashMap<String, MaterialId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub name: String,
    pub values: Vec<f32>,
}

pub fn has_program(program: &str) -> bool {
    MATERIALS.iter().any(|(name, _)| *name == program)
}

impl Effects {
    pub fn new(backend: Backend) -> Result<Self, EffectsError> {
        Ok(Self {
            filament: Filament::new(backend)?,
            loaded: HashMap::new(),
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
        size: (u32, u32),
        settings: &[Setting],
    ) -> Result<Vec<u8>, EffectsError> {
        let material = self.material(program)?;
        let uniforms: Vec<Uniform<'_>> = settings
            .iter()
            .map(|s| Uniform {
                name: &s.name,
                values: &s.values,
            })
            .collect();
        Ok(self.filament.pass(material, inputs, size, &uniforms)?)
    }
}
