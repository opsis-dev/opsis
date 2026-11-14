use opsis_types::fonts::FontFamilyDef;
use opsis_types::ts_export::render_plain;
use specta::Types;

pub const SPEC_TYPES_FILE: &str = "scripts/lib/generated/spec.ts";

pub fn spec_types() -> String {
    render_plain(&Types::default().register::<FontFamilyDef>())
}
