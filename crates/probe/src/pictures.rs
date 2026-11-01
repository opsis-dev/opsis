use crate::read::{Bits, Bytes, Short, unescape};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chroma {
    Gray,
    C420,
    C422,
    C444,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    pub chroma: Chroma,
    pub depth: u32,
    pub full_range: bool,
    pub alpha: bool,
}

impl Layout {
    pub const PLAIN: Layout = Layout {
        chroma: Chroma::C420,
        depth: 8,
        full_range: false,
        alpha: false,
    };

    pub fn pix_fmt(self, full_range_names: FullRange) -> String {
        let depth = if self.depth > 8 {
            format!("{}le", self.depth)
        } else {
            String::new()
        };
        if self.chroma == Chroma::Gray {
            return format!("gray{depth}");
        }
        let chroma = match self.chroma {
            Chroma::C422 => "422",
            Chroma::C444 => "444",
            Chroma::Gray | Chroma::C420 => "420",
        };
        let full = self.full_range
            && self.depth == 8
            && !self.alpha
            && match full_range_names {
                FullRange::Any => true,
                FullRange::Only420 => self.chroma == Chroma::C420,
                FullRange::Never => false,
            };
        let family = match (full, self.alpha) {
            (true, _) => "yuvj",
            (false, true) => "yuva",
            (false, false) => "yuv",
        };
        format!("{family}{chroma}p{depth}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullRange {
    Any,
    Only420,
    Never,
}

fn chroma_of(idc: u32) -> Chroma {
    match idc {
        0 => Chroma::Gray,
        2 => Chroma::C422,
        3 => Chroma::C444,
        _ => Chroma::C420,
    }
}

const HIGH_PROFILES: [u32; 13] = [100, 110, 122, 244, 44, 83, 86, 118, 128, 138, 139, 134, 135];

pub fn avc_config(record: &[u8]) -> Option<Layout> {
    let mut r = Bytes::new(record);
    r.skip(5).ok()?;
    let count = r.u8().ok()? & 0x1F;
    if count == 0 {
        return None;
    }
    let size = usize::from(r.be16().ok()?);
    h264_sps(r.take(size).ok()?).ok()
}

fn scaling_list(bits: &mut Bits, size: usize) -> Result<(), Short> {
    let mut last = 8i64;
    let mut next = 8i64;
    for _ in 0..size {
        if next != 0 {
            next = (last + bits.se()? + 256) % 256;
        }
        last = if next == 0 { last } else { next };
    }
    Ok(())
}

pub fn h264_sps(nal: &[u8]) -> Result<Layout, Short> {
    let data = unescape(nal);
    let mut bits = Bits::new(data.get(1..).ok_or(Short)?);
    let profile = bits.bits(8)?;
    bits.skip(16)?;
    bits.ue()?;
    let mut layout = Layout::PLAIN;
    if HIGH_PROFILES.contains(&profile) {
        let idc = bits.ue()?;
        layout.chroma = chroma_of(idc);
        if idc == 3 {
            bits.skip(1)?;
        }
        layout.depth = bits.ue()? + 8;
        bits.ue()?;
        bits.skip(1)?;
        if bits.flag()? {
            for i in 0..(if idc == 3 { 12 } else { 8 }) {
                if bits.flag()? {
                    scaling_list(&mut bits, if i < 6 { 16 } else { 64 })?;
                }
            }
        }
    }
    bits.ue()?;
    match bits.ue()? {
        0 => {
            bits.ue()?;
        }
        1 => {
            bits.skip(1)?;
            bits.se()?;
            bits.se()?;
            for _ in 0..bits.ue()? {
                bits.se()?;
            }
        }
        _ => {}
    }
    bits.ue()?;
    bits.skip(1)?;
    bits.ue()?;
    bits.ue()?;
    if !bits.flag()? {
        bits.skip(1)?;
    }
    bits.skip(1)?;
    if bits.flag()? {
        for _ in 0..4 {
            bits.ue()?;
        }
    }
    layout.full_range = vui_full_range(&mut bits)?;
    Ok(layout)
}

fn vui_full_range(bits: &mut Bits) -> Result<bool, Short> {
    if !bits.flag()? {
        return Ok(false);
    }
    if bits.flag()? && bits.bits(8)? == 255 {
        bits.skip(32)?;
    }
    if bits.flag()? {
        bits.skip(1)?;
    }
    if !bits.flag()? {
        return Ok(false);
    }
    bits.skip(3)?;
    bits.flag()
}

pub fn hevc_config(record: &[u8]) -> Option<Layout> {
    let mut r = Bytes::new(record);
    r.skip(16).ok()?;
    let chroma = chroma_of(u32::from(r.u8().ok()? & 3));
    let depth = u32::from(r.u8().ok()? & 7) + 8;
    r.skip(4).ok()?;
    let arrays = r.u8().ok()?;
    let mut layout = Layout {
        chroma,
        depth,
        full_range: false,
        alpha: false,
    };
    for _ in 0..arrays {
        let kind = r.u8().ok()? & 0x3F;
        let count = r.be16().ok()?;
        for _ in 0..count {
            let size = usize::from(r.be16().ok()?);
            let nal = r.take(size).ok()?;
            if kind == 33
                && let Ok(full) = hevc_full_range(nal)
            {
                layout.full_range = full;
            }
        }
    }
    Some(layout)
}

fn profile_tier_level(bits: &mut Bits, sub_layers: u32) -> Result<(), Short> {
    bits.skip(88)?;
    bits.skip(8)?;
    let mut present = Vec::new();
    for _ in 0..sub_layers {
        present.push((bits.flag()?, bits.flag()?));
    }
    if sub_layers > 0 {
        bits.skip(2 * (8 - sub_layers))?;
    }
    for (profile, level) in present {
        if profile {
            bits.skip(88)?;
        }
        if level {
            bits.skip(8)?;
        }
    }
    Ok(())
}

fn hevc_scaling_lists(bits: &mut Bits) -> Result<(), Short> {
    for size in 0..4u32 {
        let step = if size == 3 { 3 } else { 1 };
        let mut matrix = 0;
        while matrix < 6 {
            if bits.flag()? {
                let coefficients = 64.min(1u32 << (4 + (size << 1)));
                if size > 1 {
                    bits.se()?;
                }
                for _ in 0..coefficients {
                    bits.se()?;
                }
            } else {
                bits.ue()?;
            }
            matrix += step;
        }
    }
    Ok(())
}

fn short_term_sets(bits: &mut Bits, count: u32) -> Result<(), Short> {
    let mut deltas: Vec<u32> = Vec::new();
    for i in 0..count {
        let predicted = i != 0 && bits.flag()?;
        if predicted {
            bits.skip(1)?;
            bits.ue()?;
            let previous = deltas.last().copied().unwrap_or(0);
            let mut kept = 0;
            for _ in 0..=previous {
                let used = bits.flag()?;
                let delta = used || bits.flag()?;
                if delta {
                    kept += 1;
                }
            }
            deltas.push(kept);
        } else {
            let negative = bits.ue()?;
            let positive = bits.ue()?;
            for _ in 0..negative + positive {
                bits.ue()?;
                bits.skip(1)?;
            }
            deltas.push(negative + positive);
        }
    }
    Ok(())
}

pub fn hevc_full_range(nal: &[u8]) -> Result<bool, Short> {
    let data = unescape(nal);
    let mut bits = Bits::new(data.get(2..).ok_or(Short)?);
    bits.skip(4)?;
    let sub_layers = bits.bits(3)?;
    bits.skip(1)?;
    profile_tier_level(&mut bits, sub_layers)?;
    bits.ue()?;
    if bits.ue()? == 3 {
        bits.skip(1)?;
    }
    bits.ue()?;
    bits.ue()?;
    if bits.flag()? {
        for _ in 0..4 {
            bits.ue()?;
        }
    }
    bits.ue()?;
    bits.ue()?;
    let poc_bits = bits.ue()? + 4;
    let all_layers = bits.flag()?;
    for _ in (if all_layers { 0 } else { sub_layers })..=sub_layers {
        bits.ue()?;
        bits.ue()?;
        bits.ue()?;
    }
    for _ in 0..6 {
        bits.ue()?;
    }
    if bits.flag()? && bits.flag()? {
        hevc_scaling_lists(&mut bits)?;
    }
    bits.skip(2)?;
    if bits.flag()? {
        bits.skip(8)?;
        bits.ue()?;
        bits.ue()?;
        bits.skip(1)?;
    }
    let sets = bits.ue()?;
    short_term_sets(&mut bits, sets)?;
    if bits.flag()? {
        for _ in 0..bits.ue()? {
            bits.skip(poc_bits + 1)?;
        }
    }
    bits.skip(2)?;
    vui_full_range(&mut bits)
}

pub fn vp9_config(record: &[u8]) -> Option<Layout> {
    let mut r = Bytes::new(record);
    r.skip(6).ok()?;
    let packed = r.u8().ok()?;
    Some(Layout {
        chroma: match packed >> 1 & 7 {
            2 => Chroma::C422,
            3 => Chroma::C444,
            _ => Chroma::C420,
        },
        depth: u32::from(packed >> 4),
        full_range: packed & 1 == 1,
        alpha: false,
    })
}

pub fn vp9_frame(frame: &[u8]) -> Option<Layout> {
    let mut bits = Bits::new(frame);
    if bits.bits(2).ok()? != 2 {
        return None;
    }
    let low = bits.bits(1).ok()?;
    let profile = bits.bits(1).ok()? << 1 | low;
    if profile == 3 {
        bits.skip(1).ok()?;
    }
    if bits.flag().ok()? || bits.flag().ok()? {
        return None;
    }
    bits.skip(2).ok()?;
    if bits.bits(24).ok()? != 0x0049_8342 {
        return None;
    }
    let depth = if profile >= 2 && bits.flag().ok()? {
        12
    } else if profile >= 2 {
        10
    } else {
        8
    };
    let space = bits.bits(3).ok()?;
    let chroma = if space == 7 {
        Chroma::C444
    } else {
        bits.skip(1).ok()?;
        if profile == 1 || profile == 3 {
            match (bits.flag().ok()?, bits.flag().ok()?) {
                (true, true) => Chroma::C420,
                (true, false) => Chroma::C422,
                _ => Chroma::C444,
            }
        } else {
            Chroma::C420
        }
    };
    Some(Layout {
        chroma,
        depth,
        full_range: false,
        alpha: false,
    })
}

pub fn av1_config(record: &[u8]) -> Option<Layout> {
    let packed = *record.get(2)?;
    let high = packed & 0x40 != 0;
    let twelve = packed & 0x20 != 0;
    let mono = packed & 0x10 != 0;
    let (x, y) = (packed & 0x08 != 0, packed & 0x04 != 0);
    Some(Layout {
        chroma: match (mono, x, y) {
            (true, _, _) => Chroma::Gray,
            (false, true, true) => Chroma::C420,
            (false, true, false) => Chroma::C422,
            (false, false, _) => Chroma::C444,
        },
        depth: match (high, twelve) {
            (true, true) => 12,
            (true, false) => 10,
            _ => 8,
        },
        full_range: false,
        alpha: false,
    })
}

const AAC_RATES: [u32; 13] = [
    96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350,
];

pub struct AacConfig {
    pub rate: Option<u32>,
    pub channels: u32,
}

fn aac_rate(bits: &mut Bits) -> Result<Option<u32>, Short> {
    let index = bits.bits(4)?;
    if index == 15 {
        return Ok(Some(bits.bits(24)?));
    }
    Ok(AAC_RATES
        .get(usize::try_from(index).map_err(|_| Short)?)
        .copied())
}

pub fn aac_config(asc: &[u8]) -> Option<AacConfig> {
    let mut bits = Bits::new(asc);
    let mut object = bits.bits(5).ok()?;
    if object == 31 {
        object = 32 + bits.bits(6).ok()?;
    }
    let mut rate = aac_rate(&mut bits).ok()?;
    let channels = bits.bits(4).ok()?;
    if object == 5 || object == 29 {
        rate = aac_rate(&mut bits).ok()?.or(rate);
    }
    Some(AacConfig {
        rate,
        channels: if channels == 7 { 8 } else { channels },
    })
}

pub fn aac_rate_of(index: u32) -> Option<u32> {
    AAC_RATES.get(usize::try_from(index).ok()?).copied()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{Chroma, FullRange, Layout};

    proptest! {
        #[test]
        fn names_a_layout_the_way_ffmpeg_does(depth in prop::sample::select(vec![8u32, 10, 12]), chroma in prop::sample::select(vec![Chroma::C420, Chroma::C422, Chroma::C444]), full in any::<bool>(), alpha in any::<bool>()) {
            let name = Layout { chroma, depth, full_range: full, alpha }.pix_fmt(FullRange::Any);
            prop_assert_eq!(name.ends_with("le"), depth > 8);
            prop_assert_eq!(name.starts_with("yuvj"), full && depth == 8 && !alpha);
            prop_assert_eq!(name.starts_with("yuva"), alpha);
        }
    }

    #[test]
    fn spells_the_common_formats() {
        assert_eq!(Layout::PLAIN.pix_fmt(FullRange::Any), "yuv420p");
        let ten = Layout {
            depth: 10,
            ..Layout::PLAIN
        };
        assert_eq!(ten.pix_fmt(FullRange::Any), "yuv420p10le");
        let full = Layout {
            full_range: true,
            chroma: Chroma::C444,
            ..Layout::PLAIN
        };
        assert_eq!(full.pix_fmt(FullRange::Any), "yuvj444p");
        assert_eq!(full.pix_fmt(FullRange::Only420), "yuv444p");
    }
}
