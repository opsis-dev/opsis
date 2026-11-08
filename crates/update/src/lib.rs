pub mod error;
pub mod feed;
pub mod fetch;
pub mod install;
pub mod installer;

pub const TARGET: &str = env!("OPSIS_TARGET");
pub const COMMIT: &str = env!("OPSIS_COMMIT");
