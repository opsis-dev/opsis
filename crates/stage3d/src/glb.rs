use serde_json::Value;

const MAGIC: &[u8; 4] = b"glTF";
const HEADER: usize = 12;
const CHUNK_HEADER: usize = 8;
const JSON_CHUNK: u32 = 0x4E4F_534A;
const BYTE: u64 = 5120;
const UNSIGNED_BYTE: u64 = 5121;
const SHORT: u64 = 5122;
const UNSIGNED_SHORT: u64 = 5123;

fn word(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn unit(component: u64, value: f64) -> Option<f64> {
    match component {
        BYTE => Some((value / 127.0).max(-1.0)),
        UNSIGNED_BYTE => Some(value / 255.0),
        SHORT => Some((value / 32767.0).max(-1.0)),
        UNSIGNED_SHORT => Some(value / 65535.0),
        _ => None,
    }
}

fn normalized_bounds(json: &mut Value) -> bool {
    let Some(accessors) = json.get_mut("accessors").and_then(Value::as_array_mut) else {
        return false;
    };
    let mut changed = false;
    for accessor in accessors {
        if accessor.get("normalized").and_then(Value::as_bool) != Some(true) {
            continue;
        }
        let Some(component) = accessor.get("componentType").and_then(Value::as_u64) else {
            continue;
        };
        for key in ["min", "max"] {
            let Some(values) = accessor.get_mut(key).and_then(Value::as_array_mut) else {
                continue;
            };
            for value in values.iter_mut() {
                if let Some(scaled) = value.as_f64().and_then(|v| unit(component, v)) {
                    *value = Value::from(scaled);
                    changed = true;
                }
            }
        }
    }
    changed
}

pub fn with_unit_bounds(glb: &[u8]) -> Option<Vec<u8>> {
    if glb.get(..4) != Some(MAGIC) {
        return None;
    }
    let length = usize::try_from(word(glb, HEADER)?).ok()?;
    if word(glb, HEADER + 4)? != JSON_CHUNK {
        return None;
    }
    let start = HEADER + CHUNK_HEADER;
    let mut json: Value = serde_json::from_slice(glb.get(start..start + length)?).ok()?;
    if !normalized_bounds(&mut json) {
        return None;
    }
    let mut text = serde_json::to_vec(&json).ok()?;
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    let rest = glb.get(start + length..)?;
    let total = HEADER + CHUNK_HEADER + text.len() + rest.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&glb[..8]);
    out.extend_from_slice(&u32::try_from(total).ok()?.to_le_bytes());
    out.extend_from_slice(&u32::try_from(text.len()).ok()?.to_le_bytes());
    out.extend_from_slice(&JSON_CHUNK.to_le_bytes());
    out.extend_from_slice(&text);
    out.extend_from_slice(rest);
    Some(out)
}
