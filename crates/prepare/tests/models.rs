use std::path::{Path, PathBuf};

use draco_gltf::{ComponentType, MeshIndex, PackedGeometry, PrimitiveIndex};
use opsis_prepare::model::model;
use serde_json::Value;

const QUANTIZED_SLACK: f64 = 1e-3;

fn models() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/prepare/models")
}

fn glb_json(glb: &[u8]) -> Value {
    let length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    serde_json::from_slice(&glb[20..20 + length]).unwrap()
}

fn scalar(bytes: &[u8], kind: ComponentType, normalized: bool) -> f64 {
    let raw = match kind {
        ComponentType::F32 => return f64::from(f32::from_le_bytes(bytes.try_into().unwrap())),
        ComponentType::I16 => (
            f64::from(i16::from_le_bytes(bytes.try_into().unwrap())),
            32767.0,
        ),
        ComponentType::U16 => (
            f64::from(u16::from_le_bytes(bytes.try_into().unwrap())),
            65535.0,
        ),
        ComponentType::I8 => (f64::from(i8::from_ne_bytes([bytes[0]])), 127.0),
        ComponentType::U8 => (f64::from(bytes[0]), 255.0),
        other => panic!("positions as {other:?}"),
    };
    if normalized {
        (raw.0 / raw.1).max(-1.0)
    } else {
        raw.0
    }
}

fn bounds(geometry: &PackedGeometry) -> ([f64; 3], [f64; 3]) {
    let position = geometry
        .attributes()
        .iter()
        .find(|a| a.semantic() == "POSITION")
        .unwrap();
    let size = position.bytes().len() / position.count() / 3;
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for (i, component) in position.bytes().chunks(size).enumerate() {
        let value = scalar(component, position.component_type(), position.normalized());
        min[i % 3] = min[i % 3].min(value);
        max[i % 3] = max[i % 3].max(value);
    }
    (min, max)
}

#[test]
fn decodes_every_model_to_the_same_geometry_without_extensions_the_loader_needs() {
    let expected: Value =
        serde_json::from_str(&std::fs::read_to_string(models().join("expected.json")).unwrap())
            .unwrap();
    for (name, want) in expected.as_object().unwrap() {
        let source = models().join(name);
        let made = model(&std::fs::read(&source).unwrap(), source.parent().unwrap()).unwrap();
        assert_eq!(&made[..4], b"glTF", "{name}");
        let root = glb_json(&made);
        let used: Vec<String> = root["extensionsUsed"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|e| e.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            used.iter().all(|e| e == "KHR_mesh_quantization"),
            "{name}: {used:?}"
        );
        assert!(
            root["images"]
                .as_array()
                .unwrap()
                .iter()
                .all(|i| i.get("uri").is_none()),
            "{name}: an image still points outside"
        );
        assert_eq!(
            root["animations"].as_array().map(Vec::len),
            Some(usize::try_from(want["animations"].as_u64().unwrap()).unwrap()),
            "{name}"
        );
        let scene = draco_gltf::import_slice(&made, None).unwrap();
        for (i, prim) in want["primitives"].as_array().unwrap().iter().enumerate() {
            let geometry = scene
                .read_primitive(PrimitiveIndex::new(MeshIndex(i), 0))
                .unwrap();
            assert_eq!(
                geometry.vertex_count() as u64,
                prim["vertices"].as_u64().unwrap(),
                "{name} mesh {i}"
            );
            assert_eq!(
                geometry.indices().map_or(0, |ix| ix.count()) as u64,
                prim["indices"].as_u64().unwrap(),
                "{name} mesh {i}"
            );
            let (min, max) = bounds(&geometry);
            for k in 0..3 {
                for (got, want) in [(min[k], &prim["min"][k]), (max[k], &prim["max"][k])] {
                    assert!(
                        (got - want.as_f64().unwrap()).abs() < QUANTIZED_SLACK,
                        "{name} mesh {i}: {got} against {want}"
                    );
                }
            }
        }
    }
}
