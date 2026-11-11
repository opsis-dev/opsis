#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Units {
    starts: Vec<(usize, u32)>,
    bytes: usize,
    total: u32,
}

impl Units {
    pub fn new(text: &str) -> Self {
        let mut starts = Vec::with_capacity(text.len());
        let mut units = 0u32;
        for (at, ch) in text.char_indices() {
            starts.push((at, units));
            units += u32::try_from(ch.len_utf16()).unwrap_or(2);
        }
        Self {
            starts,
            bytes: text.len(),
            total: units,
        }
    }

    pub const fn len(&self) -> u32 {
        self.total
    }

    pub const fn is_empty(&self) -> bool {
        self.total == 0
    }

    pub fn unit(&self, byte: usize) -> u32 {
        if byte >= self.bytes {
            return self.total;
        }
        match self.starts.binary_search_by_key(&byte, |(b, _)| *b) {
            Ok(i) => self.starts[i].1,
            Err(i) => self.starts.get(i).map_or(self.total, |(_, u)| *u),
        }
    }

    pub fn byte(&self, unit: u32) -> usize {
        if unit >= self.total {
            return self.bytes;
        }
        match self.starts.binary_search_by_key(&unit, |(_, u)| *u) {
            Ok(i) => self.starts[i].0,
            Err(i) => self.starts.get(i).map_or(self.bytes, |(b, _)| *b),
        }
    }
}

pub fn units_of(text: &str) -> u32 {
    text.chars()
        .map(|c| u32::try_from(c.len_utf16()).unwrap_or(2))
        .sum()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn every_character_start_maps_both_ways(text in "\\PC{0,40}") {
            let units = Units::new(&text);
            prop_assert_eq!(units.len(), units_of(&text));
            for (at, _) in text.char_indices() {
                prop_assert_eq!(units.byte(units.unit(at)), at);
            }
            prop_assert_eq!(units.byte(units.len()), text.len());
        }
    }
}
