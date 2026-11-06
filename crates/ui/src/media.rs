use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use opsis_types::analysis::{Filmstrip, Waveform};

#[derive(Debug, Clone)]
enum Entry {
    Loading,
    Strip(Arc<Filmstrip>),
    Wave(Arc<Waveform>),
    Failed,
}

#[derive(Debug, Default, Clone)]
pub struct MediaCache(Arc<Mutex<HashMap<String, Entry>>>);

pub fn key(dir: &str, kind: &str, src: &str) -> String {
    format!("{dir}\n{kind}\n{src}")
}

impl MediaCache {
    pub fn claim(&self, key: &str) -> bool {
        let mut entries = self.0.lock().expect("media cache");
        if entries.contains_key(key) {
            return false;
        }
        entries.insert(key.to_owned(), Entry::Loading);
        true
    }

    pub fn filmstrip(&self, key: &str) -> Option<Arc<Filmstrip>> {
        match self.0.lock().expect("media cache").get(key) {
            Some(Entry::Strip(strip)) => Some(Arc::clone(strip)),
            _ => None,
        }
    }

    pub fn waveform(&self, key: &str) -> Option<Arc<Waveform>> {
        match self.0.lock().expect("media cache").get(key) {
            Some(Entry::Wave(wave)) => Some(Arc::clone(wave)),
            _ => None,
        }
    }

    pub fn put_strip(&self, key: &str, strip: Option<Filmstrip>) {
        self.0.lock().expect("media cache").insert(
            key.to_owned(),
            strip.map_or(Entry::Failed, |s| Entry::Strip(Arc::new(s))),
        );
    }

    pub fn put_wave(&self, key: &str, wave: Option<Waveform>) {
        self.0.lock().expect("media cache").insert(
            key.to_owned(),
            wave.map_or(Entry::Failed, |w| Entry::Wave(Arc::new(w))),
        );
    }
}
