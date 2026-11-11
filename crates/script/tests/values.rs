use indexmap::IndexMap;
use mlua::Lua;
use opsis_script::values::{from_lua, to_lua};
use opsis_types::edits::{EdgeEnds, MarkerEdit, MixEdit, NodePatch, WorkflowEdit};
use opsis_types::workflow::Position;
use proptest::prelude::*;
use serde_json::{Map, Value};

fn text() -> impl Strategy<Value = String> {
    "\\PC{0,8}"
}

fn position() -> impl Strategy<Value = Position> {
    (-1.0e6f64..1.0e6, -1.0e6f64..1.0e6).prop_map(|(x, y)| Position { x, y })
}

fn params() -> impl Strategy<Value = Map<String, Value>> {
    proptest::collection::btree_map(
        "[a-z]{1,6}",
        prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            any::<i64>().prop_map(Value::from),
            text().prop_map(Value::String),
            proptest::collection::vec(any::<i32>().prop_map(Value::from), 0..3)
                .prop_map(Value::Array),
        ],
        0..4,
    )
    .prop_map(|m| m.into_iter().collect())
}

fn edit() -> impl Strategy<Value = WorkflowEdit> {
    prop_oneof![
        (
            text(),
            proptest::option::of(text()),
            proptest::option::of(position()),
            proptest::option::of(params())
        )
            .prop_map(|(kind, label, position, params)| WorkflowEdit::AddNode {
                kind,
                id: None,
                label,
                position,
                params,
            }),
        (
            text(),
            proptest::option::of(text()),
            proptest::option::of(any::<bool>()),
            proptest::option::of(params())
        )
            .prop_map(|(id, label, disabled, params)| WorkflowEdit::UpdateNode {
                id,
                patch: NodePatch {
                    label,
                    disabled,
                    params,
                    ..NodePatch::default()
                },
            }),
        proptest::collection::vec(text(), 0..3).prop_map(|ids| WorkflowEdit::RemoveNodes { ids }),
        (text(), text(), text(), text()).prop_map(
            |(source, source_handle, target, target_handle)| {
                WorkflowEdit::AddEdge {
                    ends: EdgeEnds {
                        source,
                        source_handle,
                        target,
                        target_handle,
                    },
                }
            }
        ),
        proptest::collection::btree_map("[a-z]{1,4}", position(), 0..3).prop_map(|m| {
            WorkflowEdit::MoveNodes {
                positions: m.into_iter().collect::<IndexMap<_, _>>(),
            }
        }),
        proptest::collection::btree_map("[a-z]{1,4}", text(), 0..3).prop_map(|m| {
            WorkflowEdit::SetVariables {
                variables: m.into_iter().collect::<IndexMap<_, _>>(),
            }
        }),
        text().prop_map(|name| WorkflowEdit::Rename { name }),
        text().prop_map(|id| WorkflowEdit::Mix {
            edit: MixEdit::RemoveBus { id }
        }),
        (0.0f64..1.0e6).prop_map(|at_ms| WorkflowEdit::Markers {
            edit: MarkerEdit::Add { at_ms }
        }),
    ]
}

proptest! {
    #[test]
    fn an_edit_goes_into_lua_and_comes_back_equal(edit in edit()) {
        let lua = Lua::new();
        let back: WorkflowEdit = from_lua(&lua, to_lua(&lua, &edit).unwrap()).unwrap();
        prop_assert_eq!(back, edit);
    }
}
