use opsis_compile::elements::terminal::sprites::{Cell, Shape, is_sprite, sprite_for};
use opsis_golden::{differences, read};
use serde_json::{Value, json};

const FIXTURE: &str = "scene/terminal-sprites.json";

fn shown(shape: &Shape) -> Value {
    match shape {
        Shape::Rect {
            x,
            y,
            width,
            height,
            opacity,
        } => {
            json!({ "kind": "rect", "x": x, "y": y, "width": width, "height": height, "opacity": opacity })
        }
        Shape::Fill {
            path,
            opacity,
            clip,
        } => {
            json!({ "kind": "fill", "d": path.d(), "opacity": opacity.unwrap_or(1.0), "clip": clip })
        }
        Shape::Stroke {
            path,
            width,
            clip,
            inner,
        } => {
            json!({ "kind": "stroke", "d": path.d(), "width": width, "clip": clip, "inner": inner })
        }
        Shape::Inverted(shapes) => {
            json!({ "kind": "inverted", "shapes": shapes.iter().map(shown).collect::<Vec<_>>() })
        }
    }
}

#[test]
fn terminal_sprites_match_what_the_film_page_draws() {
    let recorded = read(FIXTURE);
    let cells = recorded["cells"].as_array().map_or(&[][..], Vec::as_slice);
    let made = json!({
        "cells": cells.iter().map(|entry| {
            let cell = Cell {
                width: entry["cell"]["width"].as_f64().expect("a width"),
                height: entry["cell"]["height"].as_f64().expect("a height"),
                thickness: entry["cell"]["thickness"].as_f64().expect("a thickness"),
            };
            let sprites: Vec<Value> = entry["sprites"]
                .as_array()
                .map_or(&[][..], Vec::as_slice)
                .iter()
                .map(|sprite| {
                    let code = u32::try_from(sprite["code"].as_u64().expect("a code")).expect("a code point");
                    assert!(is_sprite(code), "{code:x} is drawn as a sprite");
                    let shapes = sprite_for(code, cell)
                        .map(|list| Value::Array(list.iter().map(shown).collect()));
                    json!({ "code": code, "shapes": shapes })
                })
                .collect();
            json!({ "cell": entry["cell"], "sprites": sprites })
        }).collect::<Vec<_>>(),
    });
    let found = differences(&recorded, &made);
    assert!(
        found.is_empty(),
        "{}",
        found
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
