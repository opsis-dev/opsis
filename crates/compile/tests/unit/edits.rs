use indexmap::IndexMap;
use opsis_compile::edits::workflow::{
    GROUP_PAD, GROUP_TITLE_HEIGHT, add_project_font, connect, group_nodes, paste_nodes,
    remove_nodes, rename_workflow_variable, set_font_role, ungroup,
};
use opsis_compile::edits::{EditContext, EditError, apply_edit, edited_nodes};
use opsis_compile::graph::GROUP_TYPE;
use opsis_compile::ids::SaltedIds;
use opsis_compile::node::Platform;
use opsis_types::actions::Viewport;
use opsis_types::edits::{EdgeEnds, NodePatch, WorkflowEdit};
use opsis_types::fonts::{FontCategory, FontFamilyDef, FontRole, FontSource};
use opsis_types::mix::{Mix, Strip};
use opsis_types::workflow::{Position, Workflow};
use serde_json::json;

use crate::helpers::defaults_with;

const COLUMN: f64 = 300.0;
const ROW: f64 = 170.0;
const SIZE: Viewport = Viewport {
    width: 200,
    height: 100,
};

fn sounds() -> Workflow {
    let node = |id: &str, kind: &str, x: f64, y: f64| json!({ "id": id, "type": kind, "position": { "x": x * COLUMN, "y": y * ROW }, "params": defaults_with(kind, json!({})) });
    serde_json::from_value(json!({
        "version": 1,
        "id": "wf",
        "name": "Sounds",
        "nodes": [
            node("a", "audio.sfx", 0.0, 0.0),
            node("b", "audio.sfx", 0.0, 1.0),
            node("mix", "audio.mix", 1.0, 0.0),
            node("m", "audio.music", 0.0, 2.0),
        ],
        "edges": [],
    }))
    .unwrap()
}

fn ends(source: &str, source_handle: &str, target: &str, target_handle: &str) -> EdgeEnds {
    EdgeEnds {
        source: source.into(),
        source_handle: source_handle.into(),
        target: target.into(),
        target_handle: target_handle.into(),
    }
}

fn sizes(ids: &[&str]) -> IndexMap<String, Viewport> {
    ids.iter().map(|id| ((*id).to_owned(), SIZE)).collect()
}

fn grouped() -> Workflow {
    let mut wf = sounds();
    group_nodes(
        &mut wf,
        &["a".into(), "b".into()],
        "Effects",
        &sizes(&["a", "b"]),
        "g".into(),
    );
    wf
}

fn ids(wf: &Workflow) -> Vec<&str> {
    wf.nodes.iter().map(|n| n.id.as_str()).collect()
}

#[test]
fn lets_a_many_port_take_several_sources_but_replaces_a_single_ports_source() {
    let mut wf = sounds();
    connect(&mut wf, ends("a", "audio", "mix", "sounds"));
    connect(&mut wf, ends("b", "audio", "mix", "sounds"));
    let edge_ids: Vec<&str> = wf.edges.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(edge_ids, ["a.audio->mix.sounds", "b.audio->mix.sounds"]);
    connect(&mut wf, ends("a", "x", "mix", "clip"));
    connect(&mut wf, ends("b", "x", "mix", "clip"));
    let clips: Vec<&str> = wf
        .edges
        .iter()
        .filter(|e| e.target_handle == "clip")
        .map(|e| e.source.as_str())
        .collect();
    assert_eq!(clips, ["b"]);
}

#[test]
fn groups_around_the_members_and_ungroups_back_to_the_same_places() {
    let mut wf = grouped();
    let group = wf.node("g").unwrap();
    assert_eq!(group.kind, GROUP_TYPE);
    assert_eq!(group.params["title"], json!("Effects"));
    let plain = sounds();
    let (a, b) = (&plain.nodes[0], &plain.nodes[1]);
    let top = GROUP_PAD + GROUP_TITLE_HEIGHT;
    assert_eq!(
        group.position,
        Position {
            x: a.position.x - GROUP_PAD,
            y: a.position.y - top
        }
    );
    assert_eq!(
        group.size,
        Some(Viewport {
            width: 280,
            height: 374,
        })
    );
    assert_eq!(
        f64::from(group.size.unwrap().height),
        b.position.y - a.position.y + 100.0 + top + GROUP_PAD
    );
    let members: Vec<&str> = wf
        .nodes
        .iter()
        .filter(|n| n.parent_id.as_deref() == Some("g"))
        .map(|n| n.id.as_str())
        .collect();
    assert_eq!(members, ["a", "b"]);
    ungroup(&mut wf, "g").unwrap();
    assert_eq!(wf.nodes, sounds().nodes);
}

#[test]
fn keeps_a_group_of_fewer_than_two_free_nodes_from_forming() {
    let mut wf = sounds();
    assert_eq!(
        group_nodes(&mut wf, &["a".into()], "x", &sizes(&["a"]), "g".into()),
        None
    );
    assert_eq!(wf, sounds());
}

#[test]
fn removes_nodes_with_their_edges_and_mixer_strips_and_frees_the_children_of_a_removed_group() {
    let mut wf = grouped();
    connect(&mut wf, ends("m", "audio", "mix", "sounds"));
    let mut mix = Mix::default();
    let strip = |fader: f64| Strip {
        fader,
        ..Strip::default()
    };
    mix.channels.insert("m".into(), strip(-6.0));
    mix.channels.insert("a".into(), strip(-3.0));
    wf.mix = Some(mix);
    remove_nodes(&mut wf, &["g".into(), "m".into()]);
    assert_eq!(ids(&wf), ["a", "b", "mix"]);
    assert_eq!(wf.node("a"), sounds().node("a"));
    assert_eq!(wf.edges, []);
    let channels: Vec<&str> = wf
        .mix
        .as_ref()
        .unwrap()
        .channels
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(channels, ["a"]);
}

#[test]
fn pastes_copies_with_new_ids_offset_outside_groups_and_keeps_their_edges() {
    let mut source = grouped();
    connect(&mut source, ends("a", "audio", "mix", "sounds"));
    let copied: Vec<_> = source
        .nodes
        .iter()
        .filter(|n| ["g", "a", "mix"].contains(&n.id.as_str()))
        .cloned()
        .collect();
    let mut wf = source.clone();
    let pasted = paste_nodes(
        &mut wf,
        &copied,
        source.edges.clone(),
        Position { x: 10.0, y: 20.0 },
        &mut SaltedIds::new("p"),
    );
    assert_eq!(pasted.len(), 3);
    let fresh = &wf.nodes[source.nodes.len()..];
    let (group, a, mixer) = (&fresh[0], &fresh[1], &fresh[2]);
    assert_eq!(
        group.position,
        Position {
            x: copied[0].position.x + 10.0,
            y: copied[0].position.y + 20.0
        }
    );
    assert_eq!(a.parent_id.as_ref(), Some(&group.id));
    assert_eq!(a.position, copied[1].position);
    let last = wf.edges.last().unwrap();
    assert_eq!(last.id, format!("{}.audio->{}.sounds", a.id, mixer.id));
}

#[test]
fn pastes_a_grouped_node_without_its_group_at_its_place_on_the_canvas() {
    let source = grouped();
    let group = source.node("g").unwrap().clone();
    let in_group = source.node("a").unwrap().clone();
    let mut wf = source;
    paste_nodes(
        &mut wf,
        std::slice::from_ref(&in_group),
        Vec::new(),
        Position { x: 10.0, y: 20.0 },
        &mut SaltedIds::new("p"),
    );
    let pasted = wf.nodes.last().unwrap();
    assert_eq!(pasted.parent_id, None);
    assert_eq!(
        pasted.position,
        Position {
            x: group.position.x + in_group.position.x + 10.0,
            y: group.position.y + in_group.position.y + 20.0,
        }
    );
}

#[test]
fn renames_a_variable_everywhere_unless_the_new_name_is_blank() {
    let mut base = sounds();
    base.variables
        .insert("site".into(), "https://example.com".into());
    base.nodes = vec![serde_json::from_value(json!({ "id": "nav", "type": "browser.navigate", "position": { "x": 0, "y": 0 }, "params": defaults_with("browser.navigate", json!({ "url": "{{site}}" })) })).unwrap()];
    let renamed = |from: &str, to: &str, wf: &Workflow| {
        let mut out = wf.clone();
        rename_workflow_variable(&mut out, from, to);
        out
    };
    let home = renamed("site", "home", &base);
    assert_eq!(home.variables.keys().collect::<Vec<_>>(), ["home"]);
    assert_eq!(home.nodes[0].params["url"], json!("{{home}}"));
    assert_eq!(renamed("site", " ", &base), base);
    assert_eq!(renamed("site", "", &base), base);
    let mut both = base.clone();
    both.variables.insert("home".into(), "b".into());
    assert_eq!(renamed("site", "home", &both), both);
    assert_eq!(renamed("missing", "other", &base), base);
}

fn font(id: &str, family: &str) -> FontFamilyDef {
    FontFamilyDef {
        id: id.into(),
        family: family.into(),
        category: FontCategory::SansSerif,
        source: FontSource::File,
        faces: Vec::new(),
        license: None,
        license_file: None,
    }
}

#[test]
fn replaces_a_project_font_of_the_same_family() {
    let mut wf = sounds();
    add_project_font(&mut wf, font("one", "Inter"));
    add_project_font(&mut wf, font("two", "Inter"));
    let inter: Vec<&FontFamilyDef> = wf
        .settings
        .fonts
        .families
        .iter()
        .filter(|f| f.family == "Inter")
        .collect();
    assert_eq!(inter, [&font("two", "Inter")]);
}

#[test]
fn keeps_a_font_copied_into_the_project_when_a_role_then_picks_it() {
    let mut wf = sounds();
    add_project_font(&mut wf, font("lora", "Lora"));
    set_font_role(&mut wf, FontRole::Heading, "Lora".into());
    assert_eq!(wf.settings.fonts.heading, "Lora");
    assert!(wf.settings.fonts.families.contains(&font("lora", "Lora")));
}

#[test]
fn refuses_edits_to_nodes_and_types_that_do_not_exist() {
    let mut wf = sounds();
    let mut ids = SaltedIds::new("t");
    let mut ctx = EditContext {
        platform: Platform::Linux,
        ids: &mut ids,
    };
    let update = WorkflowEdit::UpdateNode {
        id: "gone".into(),
        patch: NodePatch::default(),
    };
    assert_eq!(
        apply_edit(&mut wf, update, &mut ctx),
        Err(EditError::NoNode("gone".into()))
    );
    let add = WorkflowEdit::AddNode {
        kind: "no.such".into(),
        id: None,
        label: None,
        position: None,
        params: None,
    };
    assert_eq!(
        apply_edit(&mut wf, add, &mut ctx),
        Err(EditError::NoNodeType("no.such".into()))
    );
    let added = apply_edit(
        &mut wf,
        WorkflowEdit::AddNode {
            kind: "audio.sfx".into(),
            id: Some("a".into()),
            label: None,
            position: None,
            params: None,
        },
        &mut ctx,
    );
    assert_eq!(added, Err(EditError::NodeExists("a".into())));
}

#[test]
fn adds_a_node_with_its_defaults_right_of_the_others() {
    let mut wf = sounds();
    let mut ids = SaltedIds::new("t");
    let mut ctx = EditContext {
        platform: Platform::Linux,
        ids: &mut ids,
    };
    let added = apply_edit(
        &mut wf,
        WorkflowEdit::AddNode {
            kind: "audio.sfx".into(),
            id: None,
            label: Some("Pop".into()),
            position: None,
            params: Some(serde_json::from_value(json!({ "volume": 0.5 })).unwrap()),
        },
        &mut ctx,
    )
    .unwrap();
    assert_eq!(added, ["audio_t0"]);
    let node = wf.node("audio_t0").unwrap();
    assert_eq!(
        node.position,
        Position {
            x: 2.0 * COLUMN,
            y: 0.0
        }
    );
    assert_eq!(node.label.as_deref(), Some("Pop"));
    assert_eq!(
        node.params,
        defaults_with("audio.sfx", json!({ "volume": 0.5 }))
            .as_object()
            .cloned()
            .unwrap()
    );
}

fn edited_film() -> Workflow {
    let mut g = crate::helpers::Graph::default();
    g.node("a", "fx.solid", json!({}));
    g.node("b", "fx.solid", json!({}));
    g.node("trim", "timeline.trim", json!({}));
    g.node("out", "output.render", json!({}));
    g.link("a", "trim").link("trim", "out");
    g.build()
}

fn sorted(mut ids: Vec<String>) -> Vec<String> {
    ids.sort();
    ids
}

#[test]
fn names_a_node_whose_settings_changed_and_everything_after_it() {
    let before = edited_film();
    let mut after = before.clone();
    after.nodes[2].params = serde_json::from_value(json!({ "startMs": 500 })).unwrap();
    assert_eq!(sorted(edited_nodes(&before, &after)), ["out", "trim"]);
}

#[test]
fn names_a_node_whose_connections_changed() {
    let before = edited_film();
    let mut after = before.clone();
    for edge in &mut after.edges {
        if edge.source == "a" {
            edge.source = "b".into();
        }
    }
    assert_eq!(sorted(edited_nodes(&before, &after)), ["out", "trim"]);
}

#[test]
fn ignores_a_node_that_only_moved() {
    let before = edited_film();
    let mut after = before.clone();
    after.nodes[0].position = Position { x: 500.0, y: 500.0 };
    assert_eq!(edited_nodes(&before, &after), [] as [String; 0]);
}
