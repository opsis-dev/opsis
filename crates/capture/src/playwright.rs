use std::sync::LazyLock;

use serde_json::Value;

pub const INJECTED_SOURCE: &str = include_str!("playwright/injected.generated.js");
pub const CLOCK_SOURCE: &str = include_str!("playwright/clock.generated.js");

static TABLES: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("playwright/tables.generated.json"))
        .expect("playwright tables parse")
});

pub fn tables() -> &'static Value {
    &TABLES
}
