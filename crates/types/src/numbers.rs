use std::time::Duration;

use humansize::{BINARY, format_size};

pub fn whole(value: f64) -> u32 {
    truncate(value.round().clamp(0.0, f64::from(u32::MAX)))
}

pub fn whole_signed(value: f64) -> i32 {
    truncate(
        value
            .round()
            .clamp(f64::from(i32::MIN), f64::from(i32::MAX)),
    )
}

pub fn signed(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

pub fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "converting to f64 is what this function is for"
)]
pub const fn float(value: usize) -> f64 {
    value as f64
}

#[expect(
    clippy::cast_precision_loss,
    reason = "converting to f64 is what this function is for"
)]
pub const fn float_wide(value: u64) -> f64 {
    value as f64
}

pub fn millis(duration: Duration) -> f64 {
    float_wide(u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "narrowing to f32 is what this function is for"
)]
pub const fn narrow(value: f64) -> f32 {
    value as f32
}

pub trait Truncated {
    fn truncated(value: f64) -> Self;
}

macro_rules! truncated {
    (unsigned $($kind:ty),*) => {$(
        impl Truncated for $kind {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "truncating toward zero like a cast is what truncate is for"
            )]
            fn truncated(value: f64) -> Self {
                value as $kind
            }
        }
    )*};
    (signed $($kind:ty),*) => {$(
        impl Truncated for $kind {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "truncating toward zero like a cast is what truncate is for"
            )]
            fn truncated(value: f64) -> Self {
                value as $kind
            }
        }
    )*};
}

truncated!(unsigned u8, u16, u32, u64, usize);
truncated!(signed i8, i16, i32, i64);

pub fn truncate<T: Truncated>(value: f64) -> T {
    T::truncated(value)
}

pub fn byte_size(bytes: u64) -> String {
    format_size(bytes, BINARY.decimal_places(1))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{
        byte_size, count, float, float_wide, narrow, signed, truncate, whole, whole_signed,
    };

    const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

    #[test]
    fn saturates_at_the_ends_and_reads_nothing_as_zero() {
        assert_eq!(whole(f64::NAN), 0);
        assert_eq!(whole(1e12), u32::MAX);
        assert_eq!(whole_signed(f64::NEG_INFINITY), i32::MIN);
        assert_eq!(truncate::<u8>(300.0), u8::MAX);
        assert_eq!(truncate::<i64>(f64::NAN), 0);
        assert_eq!(count(usize::MAX), u32::MAX);
        assert_eq!(signed(u32::MAX), i32::MAX);
    }

    #[test]
    fn writes_sizes_in_binary_units() {
        assert_eq!(
            [byte_size(512), byte_size(1536), byte_size(29 * 1024 * 1024)],
            ["512 B", "1.5 KiB", "29 MiB"]
        );
    }

    proptest! {
        #[test]
        fn rounds_to_the_nearest_whole_number_inside_the_range(value in -1e10..1e10f64) {
            let expected = value.round().clamp(0.0, f64::from(u32::MAX));
            prop_assert_eq!(f64::from(whole(value)), expected);
            let signed_expected = value.round().clamp(f64::from(i32::MIN), f64::from(i32::MAX));
            prop_assert_eq!(f64::from(whole_signed(value)), signed_expected);
        }

        #[test]
        fn never_rounds_a_larger_number_to_a_smaller_one(a in -1e10..1e10f64, b in -1e10..1e10f64) {
            let (low, high) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(whole(low) <= whole(high));
            prop_assert!(whole_signed(low) <= whole_signed(high));
        }

        #[test]
        fn truncates_toward_zero_inside_the_range(value in -30_000.0..30_000.0f64) {
            prop_assert_eq!(f64::from(truncate::<i16>(value)), value.trunc());
            prop_assert_eq!(f64::from(truncate::<i32>(value)), value.trunc());
            prop_assert_eq!(f64::from(truncate::<u32>(value)), value.trunc().max(0.0));
        }

        #[test]
        fn keeps_whole_numbers_that_fit(n in 0..(1usize << 53)) {
            prop_assert_eq!(truncate::<usize>(float(n)), n);
            prop_assert_eq!(truncate::<u64>(float_wide(n as u64)), n as u64);
            prop_assert_eq!(u64::from(count(n)), (n as u64).min(u64::from(u32::MAX)));
        }

        #[test]
        fn narrows_to_the_nearest_single(value in -1e30..1e30f64) {
            let single = f64::from(narrow(value));
            prop_assert!((single - value).abs() <= value.abs() * f64::from(f32::EPSILON));
        }

        #[test]
        fn keeps_integers_that_fit_when_signing(value in 0..=u32::MAX) {
            prop_assert_eq!(i64::from(signed(value)), i64::from(value).min(i64::from(i32::MAX)));
        }

        #[test]
        fn writes_a_size_under_its_unit_and_close_to_its_bytes(bytes in (0u32..62).prop_flat_map(|bits| 0..(1u64 << bits))) {
            let text = byte_size(bytes);
            let (number, unit) = text.split_once(' ').unwrap();
            let power = UNITS.iter().position(|u| *u == unit).unwrap();
            let shown: f64 = number.parse().unwrap();
            let scale = 1024f64.powi(i32::try_from(power).unwrap());
            if power == 0 {
                prop_assert_eq!(number, bytes.to_string());
            } else {
                prop_assert!((1.0..1024.0).contains(&shown) || shown == 1024.0);
                let exact = float_wide(bytes) / scale;
                prop_assert!((shown - exact).abs() <= 0.05 + 1e-9);
            }
        }
    }
}
