use std::sync::Arc;

use parley::FontContext;
use parley::fontique::{Blob, Collection, CollectionOptions, FontInfoOverride, SourceCache};

const WOFF2_MAGIC: &[u8; 4] = b"wOF2";
const WOFF_MAGIC: &[u8; 4] = b"wOFF";
const FALLBACK_PREFIX: &str = "opsis fallback ";

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("{name}: the font file can't be read ({message})")]
    Unreadable { name: String, message: String },
    #[error("{0}: the file holds no font")]
    Empty(String),
}

pub struct Fonts {
    pub(crate) context: FontContext,
    registered: Vec<String>,
    faces: Vec<(String, Arc<Vec<u8>>)>,
    fallbacks: Vec<String>,
}

impl std::fmt::Debug for Fonts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fonts")
            .field("registered", &self.registered)
            .finish_non_exhaustive()
    }
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

fn sfnt(name: &str, bytes: &[u8]) -> Result<Vec<u8>, FontError> {
    let unreadable = |message: String| FontError::Unreadable {
        name: name.to_owned(),
        message,
    };
    if bytes.starts_with(WOFF2_MAGIC) {
        return wuff::decompress_woff2(bytes).map_err(|e| unreadable(format!("{e:?}")));
    }
    if bytes.starts_with(WOFF_MAGIC) {
        return wuff::decompress_woff1(bytes).map_err(|e| unreadable(format!("{e:?}")));
    }
    Ok(bytes.to_vec())
}

impl Fonts {
    pub fn new() -> Self {
        Self {
            context: FontContext {
                collection: Collection::new(CollectionOptions {
                    shared: false,
                    system_fonts: false,
                }),
                source_cache: SourceCache::default(),
            },
            registered: Vec::new(),
            faces: Vec::new(),
            fallbacks: Vec::new(),
        }
    }

    pub fn register(&mut self, css_name: &str, bytes: &[u8]) -> Result<(), FontError> {
        let data = Arc::new(sfnt(css_name, bytes)?);
        let blob = Blob::new(data.clone());
        let added = self.context.collection.register_fonts(
            blob,
            Some(FontInfoOverride {
                family_name: Some(css_name),
                ..FontInfoOverride::default()
            }),
        );
        if added.is_empty() {
            return Err(FontError::Empty(css_name.to_owned()));
        }
        let key = css_name.to_lowercase();
        self.faces.push((key.clone(), data));
        if !self.registered.contains(&key) {
            self.registered.push(key);
        }
        Ok(())
    }

    pub fn register_fallback(&mut self, subset: &str, bytes: &[u8]) -> Result<(), FontError> {
        let name = format!("{FALLBACK_PREFIX}{subset}");
        self.register(&name, bytes)?;
        if !self.fallbacks.contains(&name) {
            self.fallbacks.push(name);
        }
        Ok(())
    }

    pub fn fallbacks(&self) -> &[String] {
        &self.fallbacks
    }

    pub fn faces(&self) -> &[(String, Arc<Vec<u8>>)] {
        &self.faces
    }

    pub fn has(&self, css_name: &str) -> bool {
        self.registered.contains(&css_name.to_lowercase())
    }
}
