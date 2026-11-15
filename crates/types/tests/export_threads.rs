use garde::Validate;
use opsis_types::settings::{AppSettings, default_export_threads, export_threads};
use proptest::prelude::*;

proptest! {
    #[test]
    fn exports_use_between_one_thread_and_every_thread(chosen in proptest::option::of(0u32..4096), threads in 0u32..1024) {
        let used = export_threads(chosen, threads);
        prop_assert!(used >= 1);
        prop_assert!(used <= threads.max(1));
    }

    #[test]
    fn no_choice_means_half_the_threads_rounded_up(threads in 1u32..1024) {
        prop_assert_eq!(export_threads(None, threads), threads.div_ceil(2));
        prop_assert_eq!(default_export_threads(threads), threads.div_ceil(2));
    }

    #[test]
    fn a_choice_above_the_computer_is_held_to_its_threads(threads in 1u32..1024, over in 1u32..4096) {
        prop_assert_eq!(export_threads(Some(threads + over), threads), threads);
        prop_assert_eq!(export_threads(Some(threads), threads), threads);
    }
}

#[test]
fn a_stored_thread_count_must_be_at_least_one() {
    let mut settings = AppSettings::default();
    assert_eq!(settings.media.export_threads, None);
    assert!(settings.validate().is_ok());
    settings.media.export_threads = Some(0);
    assert!(settings.validate().is_err());
    settings.media.export_threads = Some(8);
    assert!(settings.validate().is_ok());
}
