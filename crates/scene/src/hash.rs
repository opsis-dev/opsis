use std::hash::{DefaultHasher, Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::node::{Element, Node};
use crate::style::Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContentHash(pub u128);

const LOW: u8 = 0x4c;
const HIGH: u8 = 0x48;

fn halves(value: &impl Hash) -> ContentHash {
    let mut low = DefaultHasher::new();
    LOW.hash(&mut low);
    value.hash(&mut low);
    let mut high = DefaultHasher::new();
    HIGH.hash(&mut high);
    value.hash(&mut high);
    ContentHash((u128::from(high.finish()) << 64) | u128::from(low.finish()))
}

pub fn hash_of(value: &impl Hash) -> ContentHash {
    halves(value)
}

#[derive(Hash)]
struct Content<'a> {
    kind: &'a crate::node::Kind,
    style: Style,
    children: Vec<ContentHash>,
}

pub fn content_hash(node: &Node) -> ContentHash {
    match node {
        Node::Text(text) => hash_of(text),
        Node::Element(element) => element_content_hash(element),
    }
}

pub fn element_content_hash(element: &Element) -> ContentHash {
    hash_of(&Content {
        kind: &element.kind,
        style: element.style.without_placement(),
        children: element.children.iter().map(content_hash).collect(),
    })
}

pub fn full_hash(node: &Node) -> ContentHash {
    hash_of(node)
}
