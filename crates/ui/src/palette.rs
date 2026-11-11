use indexmap::IndexMap;
use opsis_compile::catalog::{CatalogFamily, CatalogPort};
use opsis_compile::node::numbered_index;
use opsis_qt::{PalettePort, PortShape};

use crate::look;
use crate::ui::plain;

const ANY_NUMBER: &str = "any number";

pub fn palette_ports(
    ports: &IndexMap<String, CatalogPort>,
    family: Option<&CatalogFamily>,
) -> Vec<PalettePort> {
    let mut shown = Vec::with_capacity(ports.len());
    let mut numbered_shown = false;
    for (id, port) in ports {
        if let Some(family) = family.filter(|f| numbered_index(&f.prefix, id).is_some()) {
            if !numbered_shown {
                numbered_shown = true;
                shown.push(numbered(family));
            }
            continue;
        }
        shown.push(single(port));
    }
    shown
}

fn single(port: &CatalogPort) -> PalettePort {
    PalettePort {
        label: port.label.clone(),
        note: if port.many {
            ANY_NUMBER.to_owned()
        } else {
            String::new()
        },
        color: look::port_color(&plain(&port.kind)).to_owned(),
        optional: port.optional,
        shape: if port.many {
            PortShape::Many
        } else {
            PortShape::Single
        },
    }
}

fn numbered(family: &CatalogFamily) -> PalettePort {
    PalettePort {
        label: family.label.clone(),
        note: match family.required {
            0 => ANY_NUMBER.to_owned(),
            needed => format!("{ANY_NUMBER} · {needed} needed"),
        },
        color: look::port_color(&plain(&family.kind)).to_owned(),
        optional: false,
        shape: PortShape::Numbered,
    }
}

#[cfg(test)]
mod tests {
    use opsis_types::workflow::PortType;
    use proptest::prelude::*;

    use super::*;

    fn port(many: bool) -> CatalogPort {
        CatalogPort {
            kind: PortType::Clip,
            label: "Port".to_owned(),
            optional: false,
            many,
        }
    }

    proptest! {
        #[test]
        fn numbered_inputs_fold_into_one_row_and_the_rest_keep_their_order(
            plain_before in 0usize..4,
            numbered_count in 0usize..6,
            plain_after in 0usize..4,
            required in 0usize..3,
        ) {
            let mut ports = IndexMap::new();
            for i in 0..plain_before {
                ports.insert(format!("before{i}"), port(i % 2 == 0));
            }
            for i in 0..numbered_count {
                ports.insert(format!("in{}", i + 1), port(false));
            }
            for i in 0..plain_after {
                ports.insert(format!("after{i}"), port(false));
            }
            let family = CatalogFamily {
                prefix: "in".to_owned(),
                label: "Clips".to_owned(),
                kind: PortType::Clip,
                required,
            };
            let shown = palette_ports(&ports, Some(&family));
            let folded = usize::from(numbered_count > 0);
            prop_assert_eq!(shown.len(), plain_before + folded + plain_after);
            let numbered = shown.iter().filter(|p| p.shape == PortShape::Numbered).count();
            prop_assert_eq!(numbered, folded);
            if folded == 1 {
                prop_assert_eq!(shown[plain_before].shape, PortShape::Numbered);
                prop_assert_eq!(shown[plain_before].note.contains("needed"), required > 0);
            }
            for (index, shown_port) in shown.iter().enumerate().take(plain_before) {
                let many = index % 2 == 0;
                prop_assert_eq!(shown_port.shape == PortShape::Many, many);
                prop_assert_eq!(shown_port.note.is_empty(), !many);
            }
        }
    }
}
