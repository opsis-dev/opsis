use std::time::Duration;

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

#[allow(clippy::cast_precision_loss)]
pub const fn float(value: usize) -> f64 {
    value as f64
}

#[allow(clippy::cast_precision_loss)]
pub const fn float_wide(value: u64) -> f64 {
    value as f64
}

pub fn millis(duration: Duration) -> f64 {
    float_wide(u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
}

#[allow(clippy::cast_possible_truncation)]
pub const fn narrow(value: f64) -> f32 {
    value as f32
}

pub trait Truncated {
    fn truncated(value: f64) -> Self;
}

macro_rules! truncated {
    ($($kind:ty),*) => {$(
        impl Truncated for $kind {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_possible_wrap
            )]
            fn truncated(value: f64) -> Self {
                value as $kind
            }
        }
    )*};
}

truncated!(u8, u16, u32, u64, usize, i8, i16, i32, i64);

pub fn truncate<T: Truncated>(value: f64) -> T {
    T::truncated(value)
}

#[cfg(test)]
mod tests {
    use super::{count, float, narrow, signed, truncate, whole, whole_signed};

    #[test]
    fn rounds_to_the_nearest_whole_number_within_range() {
        assert_eq!(whole(2.5), 3);
        assert_eq!(whole(-4.0), 0);
        assert_eq!(whole(f64::NAN), 0);
        assert_eq!(whole(1e12), u32::MAX);
        assert_eq!(whole_signed(-6.4), -6);
        assert_eq!(whole_signed(f64::NEG_INFINITY), i32::MIN);
        assert_eq!(signed(u32::MAX), i32::MAX);
    }

    #[test]
    fn converts_like_a_cast_and_saturates_at_the_ends() {
        assert_eq!(truncate::<usize>(3.9), 3);
        assert_eq!(truncate::<i16>(-2.7), -2);
        assert_eq!(truncate::<u8>(300.0), u8::MAX);
        assert_eq!(truncate::<u32>(-1.0), 0);
        assert_eq!(truncate::<i64>(f64::NAN), 0);
        assert_eq!(count(usize::MAX), u32::MAX);
        assert_eq!(float(7), 7.0);
        assert_eq!(narrow(0.1), 0.1_f32);
    }
}
