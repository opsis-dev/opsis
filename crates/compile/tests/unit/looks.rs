use std::collections::{HashMap, HashSet};

use opsis_compile::registry::registry;
use opsis_types::looks::{GLYPH_SETS, NodeLook, NodeMotion};

fn looks() -> Vec<(String, NodeLook)> {
    registry()
        .all()
        .iter()
        .map(|def| {
            let look = def
                .look
                .clone()
                .unwrap_or_else(|| panic!("{} has no look", def.kind));
            (def.kind.to_string(), look)
        })
        .collect()
}

fn appearance(look: &NodeLook) -> String {
    match (&look.morph, look.motion) {
        (Some(morph), _) => format!("{} morph {morph}", look.glyph),
        (None, Some(motion)) => format!("{} {motion:?}", look.glyph),
        (None, None) => format!("{} still", look.glyph),
    }
}

#[test]
fn every_node_has_a_stroke_glyph_from_a_known_set() {
    let unknown: Vec<String> = looks()
        .into_iter()
        .flat_map(|(kind, look)| {
            [Some(look.glyph), look.morph]
                .into_iter()
                .flatten()
                .filter(|glyph| {
                    !glyph
                        .split_once(':')
                        .is_some_and(|(set, name)| GLYPH_SETS.contains(&set) && !name.is_empty())
                })
                .map(move |glyph| format!("{kind}: {glyph}"))
        })
        .collect();
    assert_eq!(unknown, Vec::<String>::new());
}

#[test]
fn tells_nodes_that_share_a_glyph_apart_by_how_it_moves() {
    let mut first: HashMap<String, String> = HashMap::new();
    let mut clashes = Vec::new();
    for (kind, look) in looks() {
        let key = appearance(&look);
        match first.get(&key) {
            Some(other) => clashes.push(format!("{other} and {kind}")),
            None => {
                first.insert(key, kind);
            }
        }
    }
    assert_eq!(clashes, Vec::<String>::new());
}

#[test]
fn uses_every_motion_and_never_a_motion_and_a_morph_together() {
    let looks = looks();
    let used: HashSet<NodeMotion> = looks.iter().filter_map(|(_, l)| l.motion).collect();
    let unused: Vec<&NodeMotion> = NodeMotion::ALL
        .iter()
        .filter(|m| !used.contains(m))
        .collect();
    assert!(unused.is_empty(), "{unused:?}");
    let both: Vec<&String> = looks
        .iter()
        .filter(|(_, l)| l.motion.is_some() && l.morph.is_some())
        .map(|(kind, _)| kind)
        .collect();
    assert!(both.is_empty(), "{both:?}");
}
