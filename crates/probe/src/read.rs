use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use crate::ProbeError;

const CHUNK: usize = 64 * 1024;

#[derive(Debug)]
pub struct Short;

impl From<Short> for ProbeError {
    fn from(_: Short) -> Self {
        Self::Damaged
    }
}

pub struct Source {
    file: File,
    len: u64,
    buf: Vec<u8>,
    buf_at: u64,
}

impl Source {
    pub fn open(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        Ok(Self {
            file,
            len,
            buf: Vec::new(),
            buf_at: 0,
        })
    }

    pub const fn len(&self) -> u64 {
        self.len
    }

    pub fn bytes(&mut self, at: u64, n: usize) -> io::Result<Vec<u8>> {
        if at >= self.len {
            return Ok(Vec::new());
        }
        let wanted =
            usize::try_from((self.len - at).min(u64::try_from(n).unwrap_or(u64::MAX))).unwrap_or(n);
        let end = at + u64::try_from(wanted).unwrap_or(u64::MAX);
        let buf_end = self.buf_at + u64::try_from(self.buf.len()).unwrap_or(0);
        if at >= self.buf_at && end <= buf_end {
            let from = usize::try_from(at - self.buf_at).unwrap_or(0);
            return Ok(self.buf[from..from + wanted].to_vec());
        }
        if wanted <= CHUNK {
            self.fill(at, CHUNK)?;
            let take = wanted.min(self.buf.len());
            return Ok(self.buf[..take].to_vec());
        }
        let mut out = vec![0; wanted];
        self.file.seek(SeekFrom::Start(at))?;
        self.file.read_exact(&mut out)?;
        Ok(out)
    }

    fn fill(&mut self, at: u64, n: usize) -> io::Result<()> {
        let room = usize::try_from(self.len - at).unwrap_or(n).min(n);
        self.buf.resize(room, 0);
        self.file.seek(SeekFrom::Start(at))?;
        self.file.read_exact(&mut self.buf)?;
        self.buf_at = at;
        Ok(())
    }
}

pub struct Bytes<'a> {
    data: &'a [u8],
    at: usize,
}

macro_rules! numbers {
    ($($name:ident: $kind:ty = $size:literal, $from:ident;)*) => {$(
        pub fn $name(&mut self) -> Result<$kind, Short> {
            let bytes: [u8; $size] = self.take($size)?.try_into().map_err(|_| Short)?;
            Ok(<$kind>::$from(bytes))
        }
    )*};
}

impl<'a> Bytes<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, at: 0 }
    }

    pub const fn left(&self) -> usize {
        self.data.len() - self.at
    }

    pub fn rest(&self) -> &'a [u8] {
        &self.data[self.at..]
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8], Short> {
        let end = self.at.checked_add(n).ok_or(Short)?;
        let out = self.data.get(self.at..end).ok_or(Short)?;
        self.at = end;
        Ok(out)
    }

    pub fn skip(&mut self, n: usize) -> Result<(), Short> {
        self.take(n).map(|_| ())
    }

    pub fn u8(&mut self) -> Result<u8, Short> {
        Ok(self.take(1)?[0])
    }

    pub fn be24(&mut self) -> Result<u32, Short> {
        let b = self.take(3)?;
        Ok(u32::from(b[0]) << 16 | u32::from(b[1]) << 8 | u32::from(b[2]))
    }

    numbers! {
        be16: u16 = 2, from_be_bytes;
        be32: u32 = 4, from_be_bytes;
        be64: u64 = 8, from_be_bytes;
        le16: u16 = 2, from_le_bytes;
        le32: u32 = 4, from_le_bytes;
        le64: u64 = 8, from_le_bytes;
        bef32: f32 = 4, from_be_bytes;
        bef64: f64 = 8, from_be_bytes;
        bei16: i16 = 2, from_be_bytes;
        bei64: i64 = 8, from_be_bytes;
        lei64: i64 = 8, from_le_bytes;
    }
}

pub struct Bits<'a> {
    data: &'a [u8],
    bit: usize,
}

impl<'a> Bits<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, bit: 0 }
    }

    pub fn bits(&mut self, n: u32) -> Result<u32, Short> {
        let mut out = 0u32;
        for _ in 0..n {
            let byte = *self.data.get(self.bit / 8).ok_or(Short)?;
            let shift = 7 - u32::try_from(self.bit % 8).map_err(|_| Short)?;
            out = out << 1 | u32::from(byte >> shift & 1);
            self.bit += 1;
        }
        Ok(out)
    }

    pub fn flag(&mut self) -> Result<bool, Short> {
        Ok(self.bits(1)? == 1)
    }

    pub fn skip(&mut self, n: u32) -> Result<(), Short> {
        let mut left = n;
        while left > 0 {
            let step = left.min(32);
            self.bits(step)?;
            left -= step;
        }
        Ok(())
    }

    pub fn ue(&mut self) -> Result<u32, Short> {
        let mut zeros = 0u32;
        while !self.flag()? {
            zeros += 1;
            if zeros > 31 {
                return Err(Short);
            }
        }
        if zeros == 0 {
            return Ok(0);
        }
        Ok((1u32 << zeros) - 1 + self.bits(zeros)?)
    }

    pub fn se(&mut self) -> Result<i64, Short> {
        let code = i64::from(self.ue()?);
        Ok(if code % 2 == 1 {
            (code + 1) / 2
        } else {
            -(code / 2)
        })
    }
}

pub fn unescape(nal: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(nal.len());
    let mut zeros = 0;
    for &b in nal {
        if zeros >= 2 && b == 3 {
            zeros = 0;
            continue;
        }
        zeros = if b == 0 { zeros + 1 } else { 0 };
        out.push(b);
    }
    out
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{Bits, unescape};

    fn golomb(values: &[u32]) -> Vec<u8> {
        let mut bits: Vec<bool> = Vec::new();
        for &v in values {
            let code = u64::from(v) + 1;
            let width = 64 - code.leading_zeros();
            bits.extend(std::iter::repeat_n(
                false,
                usize::try_from(width - 1).unwrap(),
            ));
            bits.extend((0..width).rev().map(|i| code >> i & 1 == 1));
        }
        bits.chunks(8)
            .map(|c| {
                c.iter()
                    .enumerate()
                    .fold(0u8, |byte, (i, &b)| byte | u8::from(b) << (7 - i))
            })
            .collect()
    }

    proptest! {
        #[test]
        fn reads_back_every_exp_golomb_number(values in prop::collection::vec(0u32..1_000_000, 1..40)) {
            let data = golomb(&values);
            let mut bits = Bits::new(&data);
            for &v in &values {
                prop_assert_eq!(bits.ue().unwrap(), v);
            }
        }

        #[test]
        fn removes_only_the_bytes_that_guard_start_codes(payload in prop::collection::vec(any::<u8>(), 0..200)) {
            let mut escaped = Vec::new();
            let mut zeros = 0;
            for &b in &payload {
                if zeros >= 2 && b <= 3 {
                    escaped.push(3);
                    zeros = 0;
                }
                zeros = if b == 0 { zeros + 1 } else { 0 };
                escaped.push(b);
            }
            prop_assert_eq!(unescape(&escaped), payload);
        }
    }
}
