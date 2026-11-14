use std::collections::HashMap;
use std::io::{BufReader, BufWriter};

use opsis_filament::Backend;
use opsis_renderer::{ProtocolError, Reply, Request, receive, send};
use opsis_stage3d::{Environment, Materials, Picture};

use crate::{Effects, Setting as EffectSetting};

type Received = Option<(Vec<u8>, (u32, u32))>;

#[derive(Default)]
struct Kept {
    materials: Materials,
    models: HashMap<String, Vec<u8>>,
    environment: Option<(Vec<u8>, Vec<f32>)>,
}

fn floats(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect()
}

fn run(
    effects: &mut Effects,
    program: &str,
    size: (u32, u32),
    settings: &[opsis_renderer::Setting],
    inputs: &[Vec<u8>],
) -> Result<Vec<u8>, String> {
    let inputs: Vec<&[u8]> = inputs.iter().map(Vec::as_slice).collect();
    let settings: Vec<EffectSetting> = settings
        .iter()
        .map(|s| EffectSetting {
            name: s.name.clone(),
            values: s.values.clone(),
            columns: s.columns,
        })
        .collect();
    effects
        .run(program, &inputs, size, &settings)
        .map_err(|e| e.to_string())
}

struct StageJob<'r> {
    stage: &'r opsis_scene::node::Stage3d,
    size: (u32, u32),
    pictures: &'r [Option<(u32, u32)>],
    models: &'r [String],
    environment: bool,
}

fn stage(
    effects: &mut Effects,
    kept: &mut Kept,
    job: &StageJob<'_>,
    blobs: Vec<Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let mut blobs = blobs.into_iter();
    let picture_blobs: Vec<Received> = job
        .pictures
        .iter()
        .map(|p| p.and_then(|size| blobs.next().map(|b| (b, size))))
        .collect();
    for source in job.models {
        if let Some(bytes) = blobs.next() {
            kept.models.insert(source.clone(), bytes);
        }
    }
    if job.environment
        && let (Some(reflections), Some(irradiance)) = (blobs.next(), blobs.next())
    {
        kept.environment = Some((reflections, floats(&irradiance)));
    }
    let pictures: Vec<Option<Picture<'_>>> = picture_blobs
        .iter()
        .map(|p| {
            p.as_ref().map(|(rgba, (width, height))| Picture {
                rgba,
                width: *width,
                height: *height,
            })
        })
        .collect();
    let models = &kept.models;
    let files = |source: &str| models.get(source).cloned();
    let environment = kept
        .environment
        .as_ref()
        .map(|(reflections, irradiance)| Environment {
            reflections,
            irradiance,
        });
    opsis_stage3d::render(
        effects.filament(),
        &mut kept.materials,
        job.stage,
        job.size,
        &pictures,
        &files,
        environment,
    )
    .map_err(|e| e.to_string())
}

pub fn serve(backend: Backend) -> Result<(), ProtocolError> {
    let mut input = BufReader::new(std::io::stdin().lock());
    let mut output = BufWriter::new(std::io::stdout().lock());
    let mut effects = Effects::new(backend).map_err(|e| e.to_string());
    let mut kept = Kept::default();
    loop {
        let (request, blobs) = match receive::<Request>(&mut input) {
            Ok(received) => received,
            Err(ProtocolError::Closed) => return Ok(()),
            Err(error) => return Err(error),
        };
        let drawn = match (&mut effects, &request) {
            (Err(error), _) => Err(error.clone()),
            (
                Ok(effects),
                Request::Run {
                    program,
                    size,
                    settings,
                },
            ) => run(effects, program, *size, settings, &blobs),
            (
                Ok(effects),
                Request::Stage {
                    stage: staged,
                    size,
                    pictures,
                    models,
                    environment,
                },
            ) => {
                let job = StageJob {
                    stage: staged,
                    size: *size,
                    pictures,
                    models,
                    environment: *environment,
                };
                stage(effects, &mut kept, &job, blobs)
            }
        };
        match drawn {
            Ok(pixels) => send(&mut output, &Reply::Drawn, &[&pixels])?,
            Err(error) => send(&mut output, &Reply::Failed(error), &[])?,
        }
    }
}
