pub fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

pub fn utf16_slice(text: &str, start: usize, end: usize) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    let end = end.min(units.len());
    let start = start.min(end);
    String::from_utf16_lossy(&units[start..end])
}

pub fn utf16_units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

pub fn to_fixed(value: f64, digits: i32) -> f64 {
    let scale = 10_f64.powi(digits);
    (value * scale).round() / scale
}

pub fn graphemes(text: &str) -> Vec<&str> {
    let breaks: Vec<usize> = icu_segmenter::GraphemeClusterSegmenter::new()
        .segment_str(text)
        .collect();
    breaks.windows(2).map(|w| &text[w[0]..w[1]]).collect()
}
