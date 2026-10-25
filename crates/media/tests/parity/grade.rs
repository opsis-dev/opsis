use crate::common::{fixture, list, text, typed};
use opsis_media::grade::{
    GradeChain, GradeError, GradeSize, LookKind, filter_path, grade_filters, look_kind_of,
};
use opsis_types::values::StyleEntry;
use pretty_assertions::assert_eq;

#[test]
fn grade_chains_match() {
    let f = fixture("media/grade.json");
    for case in list(&f["chains"]) {
        let entries: Vec<StyleEntry> = typed(&case["entries"]);
        let size: GradeSize = typed(&case["size"]);
        let out =
            grade_filters::<GradeError>(&entries, size, &|file| Ok(format!("/project/{file}")))
                .unwrap();
        assert_eq!(out, typed::<GradeChain>(&case["out"]), "{case}");
    }
    for case in list(&f["lookKinds"]) {
        assert_eq!(
            look_kind_of(text(&case["file"])),
            typed::<Option<LookKind>>(&case["kind"]),
            "{case}"
        );
    }
    for case in list(&f["paths"]) {
        assert_eq!(
            filter_path(text(&case["file"])),
            text(&case["out"]),
            "{case}"
        );
    }
}
