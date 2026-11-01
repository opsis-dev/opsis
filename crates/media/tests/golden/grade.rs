use opsis_media::grade::{GradeError, GradeSize, filter_path, grade_filters, look_kind_of};
use opsis_types::values::StyleEntry;

use crate::common::{cases, json, text, typed, with};

const GRADE: &str = "media/grade.json";

#[test]
fn grade_chains_are_as_recorded() {
    let f = opsis_golden::read(GRADE);
    let chains = cases(&f["chains"], |case| {
        let entries: Vec<StyleEntry> = typed(&case["entries"]);
        let size: GradeSize = typed(&case["size"]);
        let out =
            grade_filters::<GradeError>(&entries, size, &|file| Ok(format!("/project/{file}")))
                .expect("a grade chain");
        with(case, &[("out", json(out))])
    });
    let look_kinds = cases(&f["lookKinds"], |case| {
        with(case, &[("kind", json(look_kind_of(text(&case["file"]))))])
    });
    let paths = cases(&f["paths"], |case| {
        with(case, &[("out", json(filter_path(text(&case["file"]))))])
    });
    opsis_golden::check(
        GRADE,
        &serde_json::json!({ "chains": chains, "lookKinds": look_kinds, "paths": paths }),
    );
}
