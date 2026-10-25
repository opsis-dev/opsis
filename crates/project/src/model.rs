use std::path::Component;

use gltf::Gltf;
use percent_encoding::percent_decode_str;

pub const MODEL_EXTENSIONS: [&str; 2] = ["glb", "gltf"];

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ModelFiles {
    pub files: Vec<String>,
    pub refused: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("This isn't a 3D model file.")]
    NotAModel,
    #[error("This 3D model file is damaged.")]
    Damaged,
}

fn local_path(uri: &str) -> Option<String> {
    if url::Url::parse(uri).is_ok() || uri.starts_with(['/', '\\']) {
        return None;
    }
    let decoded = percent_decode_str(uri)
        .decode_utf8()
        .ok()?
        .replace('\\', "/");
    let clean = path_clean::clean(&decoded);
    let mut parts = Vec::new();
    for component in clean.components() {
        let Component::Normal(part) = component else {
            return None;
        };
        parts.push(part.to_str()?.to_owned());
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn uris(model: &Gltf) -> Vec<&str> {
    let buffers = model.buffers().filter_map(|b| match b.source() {
        gltf::buffer::Source::Uri(uri) => Some(uri),
        gltf::buffer::Source::Bin => None,
    });
    let images = model.images().filter_map(|i| match i.source() {
        gltf::image::Source::Uri { uri, .. } => Some(uri),
        gltf::image::Source::View { .. } => None,
    });
    buffers
        .chain(images)
        .filter(|uri| !uri.starts_with("data:"))
        .collect()
}

pub fn model_files(bytes: &[u8]) -> Result<ModelFiles, ModelError> {
    let model = Gltf::from_slice(bytes).map_err(|error| match error {
        gltf::Error::Deserialize(_) if !bytes.starts_with(b"glTF") => ModelError::NotAModel,
        _ => ModelError::Damaged,
    })?;
    let mut found = ModelFiles::default();
    for uri in uris(&model) {
        match local_path(uri) {
            Some(file) if !found.files.contains(&file) => found.files.push(file),
            Some(_) => {}
            None => found.refused.push(uri.to_owned()),
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn gltf(mut parts: Value) -> Vec<u8> {
        parts["asset"] = json!({ "version": "2.0" });
        serde_json::to_vec(&parts).unwrap()
    }

    fn glb(json: &[u8]) -> Vec<u8> {
        let mut text = json.to_vec();
        text.resize(json.len().next_multiple_of(4), b' ');
        let length = u32::try_from(text.len()).unwrap();
        let mut bytes = Vec::new();
        for word in [0x4654_6c67, 2, 20 + length, length, 0x4e4f_534a] {
            bytes.extend_from_slice(&u32::to_le_bytes(word));
        }
        bytes.extend(text);
        bytes
    }

    #[test]
    fn lists_its_buffers_and_images_but_not_the_ones_inside_the_file() {
        let model = gltf(json!({
            "buffers": [
                { "uri": "scene.bin", "byteLength": 4 },
                { "uri": "data:application/octet-stream;base64,AAAA", "byteLength": 3 },
            ],
            "bufferViews": [{ "buffer": 1, "byteLength": 3 }],
            "images": [{ "uri": "textures/wood%20grain.png" }, { "bufferView": 0, "mimeType": "image/png" }, { "uri": "scene.bin" }],
        }));
        assert_eq!(
            model_files(&model).unwrap(),
            ModelFiles {
                files: vec!["scene.bin".into(), "textures/wood grain.png".into()],
                refused: vec![]
            }
        );
    }

    #[test]
    fn refuses_files_outside_the_models_folder_and_anything_on_the_internet() {
        let refused = [
            "../secret.png",
            "maps/../../up.png",
            "/etc/passwd",
            "https://example.com/a.png",
            "file:///tmp/a.png",
            "C:\\Users\\a.png",
        ];
        let images: Vec<Value> = refused
            .iter()
            .chain(&["maps/./ok.png"])
            .map(|uri| json!({ "uri": uri }))
            .collect();
        assert_eq!(
            model_files(&gltf(json!({ "images": images }))).unwrap(),
            ModelFiles {
                files: vec!["maps/ok.png".into()],
                refused: refused.map(str::to_owned).to_vec()
            }
        );
    }

    #[test]
    fn reads_the_description_inside_a_binary_model() {
        let model = glb(&gltf(json!({ "images": [{ "uri": "skin.jpg" }] })));
        assert_eq!(
            model_files(&model).unwrap(),
            ModelFiles {
                files: vec!["skin.jpg".into()],
                refused: vec![]
            }
        );
        assert!(matches!(
            model_files(b"not a model at all"),
            Err(ModelError::NotAModel)
        ));
        assert!(matches!(
            model_files(&model[..30]),
            Err(ModelError::Damaged)
        ));
        let dangling = gltf(json!({ "images": [{ "bufferView": 3, "mimeType": "image/png" }] }));
        assert!(matches!(model_files(&dangling), Err(ModelError::Damaged)));
    }
}
