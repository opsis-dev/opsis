const CONTEXT_SIZE: usize = 32;
const STATE_FACTOR: i64 = 214_748_364;
const MAX_STATE: usize = 248;
const START_STATE: u8 = 128;
const START_RANGE: u32 = 0xFF00;
const ONE: i64 = 1 << 32;
const LONGEST_SYMBOL: usize = 31;
const CUSTOM_STATE_TABLE: u64 = 2;
const RGB: u64 = 1;

struct States {
    one: [u8; 256],
    zero: [u8; 256],
}

fn state_byte(value: i64) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn states() -> States {
    let mut one = [0u8; 256];
    let mut last: i64 = 0;
    let mut p: i64 = ONE / 2;
    for _ in 0..128 {
        let mut p8 = (256 * p + ONE / 2) >> 32;
        if p8 <= last {
            p8 = last + 1;
        }
        if let Ok(at) = usize::try_from(last)
            && at != 0
            && at < 256
            && usize::try_from(p8).is_ok_and(|p8| p8 <= MAX_STATE)
        {
            one[at] = state_byte(p8);
        }
        p += ((ONE - p) * STATE_FACTOR + ONE / 2) >> 32;
        last = p8;
    }
    for (at, slot) in one
        .iter_mut()
        .enumerate()
        .take(MAX_STATE + 1)
        .skip(256 - MAX_STATE)
    {
        if *slot != 0 {
            continue;
        }
        let index = i64::try_from(at).unwrap_or(0);
        let mut p = (index * ONE + 128) >> 8;
        p += ((ONE - p) * STATE_FACTOR + ONE / 2) >> 32;
        let mut p8 = (256 * p + ONE / 2) >> 32;
        if p8 <= index {
            p8 = index + 1;
        }
        *slot = state_byte(p8.min(i64::try_from(MAX_STATE).unwrap_or(i64::MAX)));
    }
    let mut zero = [0u8; 256];
    for (at, slot) in zero.iter_mut().enumerate().take(255).skip(1) {
        *slot = 0u8.wrapping_sub(one[256 - at]);
    }
    States { one, zero }
}

struct Decoder<'a> {
    data: &'a [u8],
    at: usize,
    low: u32,
    range: u32,
    states: States,
}

impl<'a> Decoder<'a> {
    fn new(data: &'a [u8]) -> Option<Self> {
        let low = u32::from(u16::from_be_bytes([*data.first()?, *data.get(1)?]));
        let (low, at) = if low >= START_RANGE {
            (START_RANGE, data.len())
        } else {
            (low, 2)
        };
        Some(Self {
            data,
            at,
            low,
            range: START_RANGE,
            states: states(),
        })
    }

    fn refill(&mut self) {
        if self.range < 0x100 {
            self.range <<= 8;
            self.low <<= 8;
            if let Some(byte) = self.data.get(self.at) {
                self.low += u32::from(*byte);
                self.at += 1;
            }
        }
    }

    fn bit(&mut self, state: &mut u8) -> bool {
        let part = (self.range * u32::from(*state)) >> 8;
        self.range -= part;
        let set = self.low >= self.range;
        if set {
            self.low -= self.range;
            *state = self.states.one[usize::from(*state)];
            self.range = part;
        } else {
            *state = self.states.zero[usize::from(*state)];
        }
        self.refill();
        set
    }

    fn symbol(&mut self, state: &mut [u8; CONTEXT_SIZE]) -> Option<u64> {
        if self.bit(&mut state[0]) {
            return Some(0);
        }
        let mut exponent = 0;
        while self.bit(&mut state[1 + exponent.min(9)]) {
            exponent += 1;
            if exponent > LONGEST_SYMBOL {
                return None;
            }
        }
        let mut value: u64 = 1;
        for at in (0..exponent).rev() {
            value = value * 2 + u64::from(self.bit(&mut state[22 + at.min(9)]));
        }
        Some(value)
    }

    fn signed_symbol(&mut self, state: &mut [u8; CONTEXT_SIZE]) -> Option<()> {
        if self.bit(&mut state[0]) {
            return Some(());
        }
        let mut exponent = 0;
        while self.bit(&mut state[1 + exponent.min(9)]) {
            exponent += 1;
            if exponent > LONGEST_SYMBOL {
                return None;
            }
        }
        for at in (0..exponent).rev() {
            self.bit(&mut state[22 + at.min(9)]);
        }
        self.bit(&mut state[11 + exponent.min(10)]);
        Some(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parameters {
    pub colorspace: u64,
    pub bits: u64,
    pub chroma_planes: bool,
    pub h_shift: u64,
    pub v_shift: u64,
    pub alpha: bool,
}

fn parameters(decoder: &mut Decoder<'_>, in_record: bool) -> Option<Parameters> {
    let mut state = [START_STATE; CONTEXT_SIZE];
    let version = decoder.symbol(&mut state)?;
    if (version >= 3) != in_record {
        return None;
    }
    if version >= 3 {
        decoder.symbol(&mut state)?;
    }
    if decoder.symbol(&mut state)? >= CUSTOM_STATE_TABLE {
        for _ in 1..256 {
            decoder.signed_symbol(&mut state)?;
        }
    }
    let colorspace = decoder.symbol(&mut state)?;
    let bits = if version >= 1 {
        decoder.symbol(&mut state)?
    } else {
        8
    };
    let chroma_planes = decoder.bit(&mut state[0]);
    let h_shift = decoder.symbol(&mut state)?;
    let v_shift = decoder.symbol(&mut state)?;
    let alpha = decoder.bit(&mut state[0]);
    Some(Parameters {
        colorspace,
        bits: if bits == 0 { 8 } else { bits },
        chroma_planes,
        h_shift,
        v_shift,
        alpha,
    })
}

pub fn ffv1_parameters(record: &[u8], first_frame: Option<&[u8]>) -> Option<Parameters> {
    if !record.is_empty() {
        return parameters(&mut Decoder::new(record)?, true);
    }
    let mut decoder = Decoder::new(first_frame?)?;
    let mut key = START_STATE;
    if !decoder.bit(&mut key) {
        return None;
    }
    parameters(&mut decoder, false)
}

impl Parameters {
    pub fn pix_fmt(self) -> Option<String> {
        let depth = match self.bits {
            8 => String::new(),
            9..=16 => format!("{}le", self.bits),
            _ => return None,
        };
        if self.colorspace == RGB {
            return Some(match (self.bits, self.alpha) {
                (8, false) => "bgr0".to_owned(),
                (8, true) => "bgra".to_owned(),
                (_, false) => format!("gbrp{depth}"),
                (_, true) => format!("gbrap{depth}"),
            });
        }
        if self.colorspace != 0 {
            return None;
        }
        if !self.chroma_planes {
            return Some(if self.alpha {
                format!(
                    "ya{}",
                    if self.bits == 8 {
                        "8".to_owned()
                    } else {
                        depth
                    }
                )
            } else {
                format!("gray{depth}")
            });
        }
        let chroma = match (self.h_shift, self.v_shift) {
            (0, 0) => "444",
            (0, 1) => "440",
            (1, 0) => "422",
            (1, 1) => "420",
            (2, 0) => "411",
            (2, 2) => "410",
            _ => return None,
        };
        let family = if self.alpha { "yuva" } else { "yuv" };
        Some(format!("{family}{chroma}p{depth}"))
    }
}
