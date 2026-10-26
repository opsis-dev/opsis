pub fn whole(value: f64) -> u32 {
    value.round().clamp(0.0, f64::from(u32::MAX)) as u32
}

pub fn whole_signed(value: f64) -> i32 {
    value
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
}

pub fn signed(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{signed, whole, whole_signed};

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
}
