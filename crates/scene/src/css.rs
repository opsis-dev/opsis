use opsis_types::numbers::exact_eq;

use crate::color::{Color, ColorSpace, HueMethod, Interpolation};
use crate::paint::{
    ColorStop, Gradient, Image, ImageLayer, LayerSize, LinearDirection, RadialShape, RadialSize,
    Repeat, Shadow, Side,
};
use crate::style::Style;
use crate::values::{Angle, Length, Num, Position2};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Background {
    pub color: Option<Color>,
    pub layers: Vec<ImageLayer>,
}

pub fn split_top(text: &str, at: impl Fn(char) -> bool) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 && at(c) => {
                parts.push(&text[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
        .into_iter()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect()
}

pub(crate) fn words(text: &str) -> Vec<&str> {
    split_top(text, char::is_whitespace)
}

fn number(text: &str) -> Option<f64> {
    text.parse::<f64>().ok().filter(|n| n.is_finite())
}

fn unit<'a>(text: &'a str, suffix: &str) -> Option<&'a str> {
    let (head, tail) = text.split_at_checked(text.len().checked_sub(suffix.len())?)?;
    tail.eq_ignore_ascii_case(suffix).then_some(head)
}

type MakeLength = fn(f64) -> Length;

pub fn length(text: &str) -> Option<Length> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("auto") {
        return Some(Length::Auto);
    }
    let units: [(&str, MakeLength); 9] = [
        ("px", Length::px),
        ("%", Length::percent),
        ("rem", |n| Length::Rem(Num(n))),
        ("em", Length::em),
        ("vw", |n| Length::Vw(Num(n))),
        ("vh", |n| Length::Vh(Num(n))),
        ("ch", |n| Length::Ch(Num(n))),
        ("ex", |n| Length::Ex(Num(n))),
        ("lh", |n| Length::Lh(Num(n))),
    ];
    for (suffix, make) in units {
        if let Some(n) = unit(text, suffix).and_then(number) {
            return Some(make(n));
        }
    }
    number(text).filter(|n| exact_eq(*n, 0.0)).map(Length::px)
}

pub fn angle(text: &str) -> Option<Angle> {
    let text = text.trim();
    if let Some(n) = unit(text, "deg").and_then(number) {
        return Some(Angle::deg(n));
    }
    if let Some(n) = unit(text, "grad").and_then(number) {
        return Some(Angle::deg(n * 0.9));
    }
    if let Some(n) = unit(text, "rad").and_then(number) {
        return Some(Angle::Rad(Num(n)));
    }
    if let Some(n) = unit(text, "turn").and_then(number) {
        return Some(Angle::Turn(Num(n)));
    }
    number(text)
        .filter(|n| exact_eq(*n, 0.0))
        .map(|_| Angle::Zero)
}

fn keyword_x(word: &str) -> Option<Length> {
    match word.to_ascii_lowercase().as_str() {
        "left" => Some(Length::percent(0.0)),
        "center" => Some(Length::percent(50.0)),
        "right" => Some(Length::percent(100.0)),
        _ => None,
    }
}

fn keyword_y(word: &str) -> Option<Length> {
    match word.to_ascii_lowercase().as_str() {
        "top" => Some(Length::percent(0.0)),
        "center" => Some(Length::percent(50.0)),
        "bottom" => Some(Length::percent(100.0)),
        _ => None,
    }
}

pub fn position(parts: &[&str]) -> Option<Position2> {
    match parts {
        [one] => {
            if let Some(y) = keyword_y(one).filter(|_| keyword_x(one).is_none()) {
                return Some(Position2 {
                    x: Length::percent(50.0),
                    y,
                });
            }
            Some(Position2 {
                x: keyword_x(one).or_else(|| length(one))?,
                y: Length::percent(50.0),
            })
        }
        [a, b] => {
            if keyword_y(a).is_some() && keyword_x(a).is_none()
                || keyword_x(b).is_some() && keyword_y(b).is_none()
            {
                return Some(Position2 {
                    x: keyword_x(b).or_else(|| length(b))?,
                    y: keyword_y(a).or_else(|| length(a))?,
                });
            }
            Some(Position2 {
                x: keyword_x(a).or_else(|| length(a))?,
                y: keyword_y(b).or_else(|| length(b))?,
            })
        }
        _ => None,
    }
}

fn space(word: &str) -> Option<ColorSpace> {
    Some(match word.to_ascii_lowercase().as_str() {
        "srgb" => ColorSpace::Srgb,
        "srgb-linear" => ColorSpace::SrgbLinear,
        "display-p3" => ColorSpace::DisplayP3,
        "hsl" => ColorSpace::Hsl,
        "hwb" => ColorSpace::Hwb,
        "lab" => ColorSpace::Lab,
        "lch" => ColorSpace::Lch,
        "oklab" => ColorSpace::Oklab,
        "oklch" => ColorSpace::Oklch,
        "xyz" | "xyz-d65" => ColorSpace::XyzD65,
        _ => return None,
    })
}

fn hue(word: &str) -> Option<HueMethod> {
    Some(match word.to_ascii_lowercase().as_str() {
        "shorter" => HueMethod::Shorter,
        "longer" => HueMethod::Longer,
        "increasing" => HueMethod::Increasing,
        "decreasing" => HueMethod::Decreasing,
        _ => return None,
    })
}

pub(crate) fn take_interpolation<'a>(parts: &[&'a str]) -> Option<(Interpolation, Vec<&'a str>)> {
    let Some(at) = parts.iter().position(|w| w.eq_ignore_ascii_case("in")) else {
        return Some((Interpolation::DEFAULT, parts.to_vec()));
    };
    let space = space(parts.get(at + 1)?)?;
    let mut rest: Vec<&str> = parts[..at].to_vec();
    let mut method = HueMethod::Shorter;
    let mut next = at + 2;
    if let Some(h) = parts.get(next).and_then(|w| hue(w)) {
        if !parts.get(next + 1)?.eq_ignore_ascii_case("hue") {
            return None;
        }
        method = h;
        next += 2;
    }
    rest.extend_from_slice(&parts[next..]);
    Some((Interpolation { space, hue: method }, rest))
}

fn stops(args: &[&str]) -> Option<Vec<ColorStop>> {
    let mut out = Vec::new();
    for arg in args {
        let parts = words(arg);
        let Some((first, positions)) = parts.split_first() else {
            continue;
        };
        let Some(color) = Color::css(first) else {
            if parts.len() == 1 && length(first).is_some() {
                continue;
            }
            return None;
        };
        let at = match positions.first() {
            Some(p) => Some(length(p)?),
            None => None,
        };
        let until = match positions.get(1) {
            Some(p) => Some(length(p)?),
            None => None,
        };
        out.push(ColorStop { color, at, until });
    }
    (out.len() >= 2).then_some(out)
}

fn side(word: &str) -> Option<Side> {
    Some(match word.to_ascii_lowercase().as_str() {
        "top" => Side::Top,
        "right" => Side::Right,
        "bottom" => Side::Bottom,
        "left" => Side::Left,
        _ => return None,
    })
}

fn linear_direction(parts: &[&str]) -> Option<LinearDirection> {
    match parts {
        [] => Some(LinearDirection::Angle(Angle::deg(180.0))),
        [one] => angle(one).map(LinearDirection::Angle),
        [to, rest @ ..] if to.eq_ignore_ascii_case("to") && !rest.is_empty() => {
            let mut horizontal = None;
            let mut vertical = None;
            for word in rest {
                match side(word)? {
                    s @ (Side::Left | Side::Right) if horizontal.is_none() => horizontal = Some(s),
                    s @ (Side::Top | Side::Bottom) if vertical.is_none() => vertical = Some(s),
                    _ => return None,
                }
            }
            Some(LinearDirection::ToSide {
                horizontal,
                vertical,
            })
        }
        _ => None,
    }
}

fn split_at_word<'a>(parts: &[&'a str], word: &str) -> (Vec<&'a str>, Option<Vec<&'a str>>) {
    parts
        .iter()
        .position(|w| w.eq_ignore_ascii_case(word))
        .map_or_else(
            || (parts.to_vec(), None),
            |i| (parts[..i].to_vec(), Some(parts[i + 1..].to_vec())),
        )
}

fn radial_extent(word: &str) -> Option<RadialSize> {
    Some(match word.to_ascii_lowercase().as_str() {
        "closest-side" => RadialSize::ClosestSide,
        "closest-corner" => RadialSize::ClosestCorner,
        "farthest-side" => RadialSize::FarthestSide,
        "farthest-corner" => RadialSize::FarthestCorner,
        _ => return None,
    })
}

fn radial_shape(parts: &[&str]) -> Option<(RadialShape, RadialSize)> {
    let mut shape = None;
    let mut size = None;
    let mut lengths = Vec::new();
    for word in parts {
        match word.to_ascii_lowercase().as_str() {
            "circle" => shape = Some(RadialShape::Circle),
            "ellipse" => shape = Some(RadialShape::Ellipse),
            _ => {
                if let Some(extent) = radial_extent(word) {
                    size = Some(extent);
                } else {
                    lengths.push(length(word)?);
                }
            }
        }
    }
    match lengths.as_slice() {
        [] => {}
        [r] => {
            shape = Some(RadialShape::Circle);
            size = Some(RadialSize::Explicit(r.clone(), r.clone()));
        }
        [x, y] => {
            shape = Some(RadialShape::Ellipse);
            size = Some(RadialSize::Explicit(x.clone(), y.clone()));
        }
        _ => return None,
    }
    Some((shape.unwrap_or_default(), size.unwrap_or_default()))
}

fn function<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let text = text.trim();
    let open = text.find('(')?;
    let (head, rest) = text.split_at(open);
    if !head.trim().eq_ignore_ascii_case(name) {
        return None;
    }
    rest.strip_prefix('(')?.strip_suffix(')')
}

fn gradient_of(text: &str, kind: &str, repeating: bool) -> Option<Gradient> {
    let name = if repeating {
        format!("repeating-{kind}-gradient")
    } else {
        format!("{kind}-gradient")
    };
    let args = split_top(function(text, &name)?, |c| c == ',');
    let (first, rest) = args.split_first()?;
    let lead = words(first);
    let starts_with_color = Color::css(lead.first()?).is_some();
    let (prelude, stop_args): (Vec<&str>, &[&str]) = if starts_with_color {
        (Vec::new(), &args)
    } else {
        (lead, rest)
    };
    let (interpolation, prelude) = take_interpolation(&prelude)?;
    let stops = stops(stop_args)?;
    match kind {
        "linear" => Some(Gradient::Linear {
            direction: linear_direction(&prelude)?,
            stops,
            repeating,
            interpolation,
        }),
        "radial" => {
            let (shape_words, at) = split_at_word(&prelude, "at");
            let (shape, size) = radial_shape(&shape_words)?;
            let center = at.map_or(Some(Position2::center()), |p| position(&p))?;
            Some(Gradient::Radial {
                shape,
                size,
                center,
                stops,
                repeating,
                interpolation,
            })
        }
        _ => {
            let (from_words, at) = split_at_word(&prelude, "at");
            let from = match from_words.as_slice() {
                [] => Angle::Zero,
                [word, value] if word.eq_ignore_ascii_case("from") => angle(value)?,
                _ => return None,
            };
            let center = at.map_or(Some(Position2::center()), |p| position(&p))?;
            Some(Gradient::Conic {
                from,
                center,
                stops,
                repeating,
                interpolation,
            })
        }
    }
}

pub fn gradient(text: &str) -> Option<Gradient> {
    ["linear", "radial", "conic"]
        .iter()
        .find_map(|kind| gradient_of(text, kind, false).or_else(|| gradient_of(text, kind, true)))
}

fn url(text: &str) -> Option<String> {
    let inner = function(text, "url")?.trim();
    let unquoted = inner
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| inner.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
        .unwrap_or(inner);
    Some(unquoted.to_owned())
}

fn image(text: &str) -> Option<Image> {
    gradient(text)
        .map(Image::Gradient)
        .or_else(|| url(text).map(|u| Image::Url(crate::paint::AssetRef(u))))
}

fn repeat(word: &str) -> Option<(Repeat, Repeat)> {
    Some(match word.to_ascii_lowercase().as_str() {
        "repeat" => (Repeat::Repeat, Repeat::Repeat),
        "no-repeat" => (Repeat::NoRepeat, Repeat::NoRepeat),
        "repeat-x" => (Repeat::Repeat, Repeat::NoRepeat),
        "repeat-y" => (Repeat::NoRepeat, Repeat::Repeat),
        "space" => (Repeat::Space, Repeat::Space),
        "round" => (Repeat::Round, Repeat::Round),
        _ => return None,
    })
}

fn layer_size(parts: &[&str]) -> Option<LayerSize> {
    match parts {
        [one] if one.eq_ignore_ascii_case("cover") => Some(LayerSize::Cover),
        [one] if one.eq_ignore_ascii_case("contain") => Some(LayerSize::Contain),
        [one] => Some(LayerSize::Explicit(length(one)?, Length::Auto)),
        [w, h] => Some(LayerSize::Explicit(length(w)?, length(h)?)),
        _ => None,
    }
}

fn spaced_slashes(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 4);
    let mut depth = 0_usize;
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            '/' if depth == 0 => {
                out.push_str(" / ");
                continue;
            }
            _ => {}
        }
        out.push(c);
    }
    out
}

fn layer(text: &str) -> Option<(Option<ImageLayer>, Option<Color>)> {
    let spaced = spaced_slashes(text);
    let parts = words(&spaced);
    let mut found = None;
    let mut color = None;
    let mut placement = Vec::new();
    let mut repeats = None;
    for part in &parts {
        if let Some(i) = image(part) {
            found = Some(i);
        } else if let Some(r) = repeat(part) {
            repeats = Some(r);
        } else if let Some(c) = Color::css(part).filter(|_| keyword_x(part).is_none()) {
            color = Some(c);
        } else {
            placement.push(*part);
        }
    }
    let Some(found) = found else {
        return placement.is_empty().then_some((None, color));
    };
    let mut out = ImageLayer::of(found);
    if let Some(r) = repeats {
        out.repeat = r;
    }
    let slash = placement.iter().position(|w| *w == "/");
    let (at, size) = slash.map_or((placement.as_slice(), None), |i| {
        (&placement[..i], Some(&placement[i + 1..]))
    });
    if !at.is_empty() {
        out.position = position(at)?;
    }
    if let Some(size) = size {
        out.size = layer_size(size)?;
    }
    Some((Some(out), color))
}

pub fn background(text: &str) -> Option<Background> {
    let text = text.trim();
    if text.is_empty() || text.eq_ignore_ascii_case("none") {
        return Some(Background::default());
    }
    let mut out = Background::default();
    let parts = split_top(text, |c| c == ',');
    let last = parts.len().saturating_sub(1);
    for (i, part) in parts.iter().enumerate() {
        let (found, color) = layer(part)?;
        if color.is_some() && i != last {
            return None;
        }
        out.color = out.color.or(color);
        out.layers.extend(found);
    }
    Some(out)
}

#[must_use]
pub fn with_background(style: Style, text: &str) -> Style {
    let Some(parsed) = background(text) else {
        return style;
    };
    let style = style.background(parsed.layers);
    match parsed.color {
        Some(color) => style.background_color(color),
        None => style,
    }
}

fn shadow(text: &str) -> Option<Shadow> {
    let mut inset = false;
    let mut color = None;
    let mut lengths = Vec::new();
    for word in words(text) {
        if word.eq_ignore_ascii_case("inset") {
            inset = true;
        } else if let Some(l) = length(word).filter(|l| *l != Length::Auto) {
            lengths.push(l);
        } else {
            color = Some(Color::css(word)?);
        }
    }
    if !(2..=4).contains(&lengths.len()) {
        return None;
    }
    let at = |i: usize| lengths.get(i).cloned().unwrap_or(Length::ZERO);
    Some(Shadow {
        inset,
        x: at(0),
        y: at(1),
        blur: at(2),
        spread: at(3),
        color: color.unwrap_or(Color::CurrentColor),
    })
}

pub fn shadows(text: &str) -> Option<Vec<Shadow>> {
    let text = text.trim();
    if text.is_empty() || text.eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }
    split_top(text, |c| c == ',')
        .into_iter()
        .map(shadow)
        .collect()
}

pub fn font_family(text: &str) -> Vec<String> {
    split_top(text, |c| c == ',')
        .into_iter()
        .map(|name| {
            name.trim_matches(|c| c == '"' || c == '\'')
                .trim()
                .to_owned()
        })
        .filter(|name| !name.is_empty())
        .collect()
}
