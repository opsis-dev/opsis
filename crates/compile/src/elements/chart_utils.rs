use opsis_scene::random::js_number_text;
use opsis_types::numbers::exact_eq;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Domain {
    pub min: f64,
    pub max: f64,
    pub span: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotArea {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plotted {
    pub x: f64,
    pub y: f64,
    pub value: f64,
    pub label: Option<String>,
}

fn domain_of(values: &[f64], include_zero: bool) -> Domain {
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return Domain {
            min: 0.0,
            max: 1.0,
            span: 1.0,
        };
    }
    let mut min = finite.iter().copied().fold(f64::INFINITY, f64::min);
    let mut max = finite.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if include_zero {
        min = min.min(0.0);
        max = max.max(0.0);
    }
    if exact_eq(min, max) {
        let padding = if exact_eq(max, 0.0) { 1.0 } else { max.abs() };
        return Domain {
            min: min - padding,
            max: max + padding,
            span: padding * 2.0,
        };
    }
    Domain {
        min,
        max,
        span: max - min,
    }
}

fn nice_step(raw: f64) -> f64 {
    if raw <= 0.0 {
        return 1.0;
    }
    let magnitude = 10_f64.powf(raw.log10().floor());
    let normalized = raw / magnitude;
    let step = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    step * magnitude
}

pub fn nice_domain(values: &[f64], include_zero: bool, ticks: f64) -> (Domain, Vec<f64>) {
    let raw = domain_of(values, include_zero);
    let step = nice_step(raw.span / ticks.max(1.0));
    let min = (raw.min / step).floor() * step;
    let max = (raw.max / step).ceil() * step;
    let mut out = Vec::new();
    let mut index = 0.0;
    loop {
        let tick = f64::mul_add(index, step, min);
        if tick > step.mul_add(1e-6, max) {
            break;
        }
        out.push(format!("{tick:.11e}").parse().unwrap_or(tick));
        index += 1.0;
    }
    let span = max - min;
    (
        Domain {
            min,
            max,
            span: if exact_eq(span, 0.0) { 1.0 } else { span },
        },
        out,
    )
}

fn scale_value(value: f64, min: f64, max: f64) -> f64 {
    if exact_eq(max, min) {
        return 0.0;
    }
    (value - min) / (max - min)
}

pub fn plot_area(
    width: f64,
    height: f64,
    (top, right, bottom, left): (f64, f64, f64, f64),
) -> PlotArea {
    PlotArea {
        left,
        top,
        right: width - right,
        bottom: height - bottom,
        width: (width - left - right).max(1.0),
        height: (height - top - bottom).max(1.0),
    }
}

pub fn plot_points(
    points: &[(f64, f64, Option<String>)],
    plot: &PlotArea,
    y: &Domain,
) -> Vec<Plotted> {
    let xs: Vec<f64> = points.iter().map(|p| p.0).collect();
    let x = domain_of(&xs, false);
    points
        .iter()
        .map(|(px, py, label)| Plotted {
            x: scale_value(*px, x.min, x.max).mul_add(plot.width, plot.left),
            y: scale_value(*py, y.min, y.max)
                .clamp(0.0, 1.0)
                .mul_add(-plot.height, plot.bottom),
            value: *py,
            label: label.clone(),
        })
        .collect()
}

fn round2(value: f64) -> String {
    js_number_text(opsis_scene::clock::js_round(value * 100.0) / 100.0)
}

pub fn line_path(points: &[(f64, f64)]) -> String {
    points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            format!(
                "{} {} {}",
                if i == 0 { "M" } else { "L" },
                round2(p.0),
                round2(p.1)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn smooth_path(points: &[(f64, f64)], tension: f64) -> String {
    if points.len() < 3 {
        return line_path(points);
    }
    let mut parts = vec![format!("M {} {}", round2(points[0].0), round2(points[0].1))];
    for index in 0..points.len() - 1 {
        let previous = if index == 0 {
            points[0]
        } else {
            points[index - 1]
        };
        let current = points[index];
        let next = points[index + 1];
        let after = points.get(index + 2).copied().unwrap_or(next);
        let (low, high) = (current.1.min(next.1), current.1.max(next.1));
        let clamp_y = |v: f64| v.clamp(low, high);
        let c1x = ((next.0 - previous.0) / 6.0 * tension).mul_add(2.0, current.0);
        let c1y = clamp_y(((next.1 - previous.1) / 6.0 * tension).mul_add(2.0, current.1));
        let c2x = ((after.0 - current.0) / 6.0 * tension).mul_add(-2.0, next.0);
        let c2y = clamp_y(((after.1 - current.1) / 6.0 * tension).mul_add(-2.0, next.1));
        parts.push(format!(
            "C {} {}, {} {}, {} {}",
            round2(c1x),
            round2(c1y),
            round2(c2x),
            round2(c2y),
            round2(next.0),
            round2(next.1)
        ));
    }
    parts.join(" ")
}

pub fn area_path(line: &str, points: &[(f64, f64)], baseline: f64) -> String {
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return String::new();
    };
    if line.is_empty() {
        return String::new();
    }
    format!(
        "{line} L {} {} L {} {} Z",
        round2(last.0),
        round2(baseline),
        round2(first.0),
        round2(baseline)
    )
}
