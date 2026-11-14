use std::io::{BufRead, Write};

use opsis_scene::node::Stage3d;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setting {
    pub name: String,
    pub values: Vec<f32>,
    pub columns: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Request {
    Run {
        program: String,
        size: (u32, u32),
        settings: Vec<Setting>,
    },
    Stage {
        stage: Box<Stage3d>,
        size: (u32, u32),
        pictures: Vec<Option<(u32, u32)>>,
        models: Vec<String>,
        environment: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reply {
    Drawn,
    Failed(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct Header<T> {
    message: T,
    blobs: Vec<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("the drawing process stopped")]
    Closed,
}

pub fn send<T: Serialize>(
    out: &mut impl Write,
    message: &T,
    blobs: &[&[u8]],
) -> Result<(), ProtocolError> {
    let header = Header {
        message,
        blobs: blobs.iter().map(|b| b.len()).collect(),
    };
    serde_json::to_writer(&mut *out, &header)?;
    out.write_all(b"\n")?;
    for blob in blobs {
        out.write_all(blob)?;
    }
    out.flush()?;
    Ok(())
}

pub fn receive<T: for<'de> Deserialize<'de>>(
    input: &mut impl BufRead,
) -> Result<(T, Vec<Vec<u8>>), ProtocolError> {
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Err(ProtocolError::Closed);
    }
    let header: Header<T> = serde_json::from_str(&line)?;
    let mut blobs = Vec::with_capacity(header.blobs.len());
    for length in header.blobs {
        let mut blob = vec![0u8; length];
        input.read_exact(&mut blob)?;
        blobs.push(blob);
    }
    Ok((header.message, blobs))
}
