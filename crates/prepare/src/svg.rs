use crate::PrepareError;

pub fn svg(bytes: &[u8]) -> Result<Vec<u8>, PrepareError> {
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default())
        .map_err(|e| PrepareError::Detail(e.to_string()))?;
    Ok(tree.to_string(&usvg::WriteOptions::default()).into_bytes())
}
