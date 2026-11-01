use std::path::Path;

use draco_gltf::{GeometryWriteOptions, MeshIndex, PrimitiveIndex};
use serde_json::{Value, json};

use crate::PrepareError;

const GLB_MAGIC: &[u8; 4] = b"glTF";
const GLB_VERSION: u32 = 2;
const JSON_CHUNK: u32 = 0x4E4F_534A;
const BIN_CHUNK: u32 = 0x004E_4942;
const HEADER: usize = 12;
const CHUNK_HEADER: usize = 8;
const DRACO: &str = "KHR_draco_mesh_compression";
const MESHOPT: [&str; 2] = ["EXT_meshopt_compression", "KHR_meshopt_compression"];

fn unreadable<E: std::fmt::Display>(error: E) -> PrepareError {
    PrepareError::Detail(error.to_string())
}

fn draco_primitives(root: &Value) -> Vec<PrimitiveIndex> {
    let meshes = root.get("meshes").and_then(Value::as_array);
    meshes
        .into_iter()
        .flatten()
        .enumerate()
        .flat_map(|(m, mesh)| {
            let primitives = mesh.get("primitives").and_then(Value::as_array);
            primitives
                .into_iter()
                .flatten()
                .enumerate()
                .filter(|(_, p)| p.pointer(&format!("/extensions/{DRACO}")).is_some())
                .map(move |(p, _)| PrimitiveIndex::new(MeshIndex(m), p))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn drop_extension(object: &mut Value, names: &[&str]) {
    let Some(extensions) = object.get_mut("extensions").and_then(Value::as_object_mut) else {
        return;
    };
    for name in names {
        extensions.remove(*name);
    }
    if extensions.is_empty()
        && let Some(map) = object.as_object_mut()
    {
        map.remove("extensions");
    }
}

fn drop_meshopt(root: &mut Value) {
    for list in ["bufferViews", "buffers"] {
        if let Some(items) = root.get_mut(list).and_then(Value::as_array_mut) {
            for item in items {
                drop_extension(item, &MESHOPT);
            }
        }
    }
    for list in ["extensionsUsed", "extensionsRequired"] {
        if let Some(names) = root.get_mut(list).and_then(Value::as_array_mut) {
            names.retain(|n| !n.as_str().is_some_and(|n| MESHOPT.contains(&n)));
        }
    }
}

struct Glb {
    json: Value,
    bin: Vec<u8>,
}

fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn read_glb(bytes: &[u8]) -> Result<Glb, PrepareError> {
    if bytes.get(..4) != Some(GLB_MAGIC) {
        return Err(PrepareError::Unreadable);
    }
    let mut at = HEADER;
    let (mut json, mut bin) = (None, Vec::new());
    while at + CHUNK_HEADER <= bytes.len() {
        let length = read_u32(bytes, at).ok_or(PrepareError::Unreadable)? as usize;
        let kind = read_u32(bytes, at + 4).ok_or(PrepareError::Unreadable)?;
        let body = bytes
            .get(at + CHUNK_HEADER..at + CHUNK_HEADER + length)
            .ok_or(PrepareError::Unreadable)?;
        match kind {
            JSON_CHUNK => json = Some(serde_json::from_slice(body).map_err(unreadable)?),
            BIN_CHUNK => bin = body.to_vec(),
            _ => {}
        }
        at += CHUNK_HEADER + length;
    }
    Ok(Glb {
        json: json.ok_or(PrepareError::Unreadable)?,
        bin,
    })
}

fn padded(mut bytes: Vec<u8>, fill: u8) -> Vec<u8> {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(fill);
    }
    bytes
}

fn write_glb(glb: Glb) -> Result<Vec<u8>, PrepareError> {
    let json = padded(serde_json::to_vec(&glb.json).map_err(unreadable)?, b' ');
    let bin = padded(glb.bin, 0);
    let total = HEADER
        + CHUNK_HEADER
        + json.len()
        + if bin.is_empty() {
            0
        } else {
            CHUNK_HEADER + bin.len()
        };
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(GLB_MAGIC);
    out.extend_from_slice(&GLB_VERSION.to_le_bytes());
    out.extend_from_slice(&u32::try_from(total).map_err(unreadable)?.to_le_bytes());
    for (kind, body) in [(JSON_CHUNK, &json), (BIN_CHUNK, &bin)] {
        if body.is_empty() && kind == BIN_CHUNK {
            continue;
        }
        out.extend_from_slice(&u32::try_from(body.len()).map_err(unreadable)?.to_le_bytes());
        out.extend_from_slice(&kind.to_le_bytes());
        out.extend_from_slice(body);
    }
    Ok(out)
}

fn image_mime(uri: &str) -> &'static str {
    let extension = Path::new(uri)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ktx2") => "image/ktx2",
        _ => "image/png",
    }
}

fn inside(folder: &Path, uri: &str) -> Option<std::path::PathBuf> {
    let decoded = percent_encoding::percent_decode_str(uri)
        .decode_utf8()
        .ok()?;
    let relative = Path::new(decoded.as_ref());
    let safe = relative.components().all(|c| {
        matches!(
            c,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    });
    safe.then(|| folder.join(relative))
}

fn embed_images(glb: &mut Glb, folder: &Path) -> Result<(), PrepareError> {
    let Some(images) = glb.json.get("images").and_then(Value::as_array).cloned() else {
        return Ok(());
    };
    let mut views = glb
        .json
        .get("bufferViews")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut placed = Vec::with_capacity(images.len());
    for mut image in images {
        let uri = image.get("uri").and_then(Value::as_str).map(str::to_owned);
        if let Some(uri) = uri.filter(|u| !u.starts_with("data:")) {
            let file = inside(folder, &uri).ok_or(PrepareError::Unreadable)?;
            let bytes = std::fs::read(file).map_err(unreadable)?;
            glb.bin = padded(std::mem::take(&mut glb.bin), 0);
            let offset = glb.bin.len();
            glb.bin.extend_from_slice(&bytes);
            views.push(json!({ "buffer": 0, "byteOffset": offset, "byteLength": bytes.len() }));
            if let Some(map) = image.as_object_mut() {
                map.remove("uri");
                map.insert("bufferView".into(), json!(views.len() - 1));
                map.entry("mimeType")
                    .or_insert_with(|| json!(image_mime(&uri)));
            }
        }
        placed.push(image);
    }
    let length = glb.bin.len();
    if let Some(root) = glb.json.as_object_mut() {
        root.insert("images".into(), Value::Array(placed));
        root.insert("bufferViews".into(), Value::Array(views));
        match root.get_mut("buffers").and_then(Value::as_array_mut) {
            Some(buffers) if !buffers.is_empty() => buffers[0]["byteLength"] = json!(length),
            _ => {
                root.insert("buffers".into(), json!([{ "byteLength": length }]));
            }
        }
    }
    Ok(())
}

pub fn model(bytes: &[u8], folder: &Path) -> Result<Vec<u8>, PrepareError> {
    let mut scene = draco_gltf::import_slice(bytes, Some(folder)).map_err(unreadable)?;
    let document: Value =
        serde_json::from_slice(&scene.document().to_json_bytes().map_err(unreadable)?)
            .map_err(unreadable)?;
    for primitive in draco_primitives(&document) {
        let geometry = scene.read_primitive(primitive).map_err(unreadable)?;
        scene
            .write_primitive(primitive, &geometry, GeometryWriteOptions::default())
            .map_err(unreadable)?;
    }
    let glb = scene
        .to_bytes(draco_gltf::OutputFormat::GlbV2)
        .map_err(unreadable)?;
    let mut glb = read_glb(&glb)?;
    drop_meshopt(&mut glb.json);
    embed_images(&mut glb, folder)?;
    write_glb(glb)
}
