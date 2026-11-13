use opsis_types::numbers::exact_eq;

#[derive(Debug, Clone, Copy)]
pub struct Step<S> {
    pub at: f64,
    pub state: S,
    pub duration: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Transition<S> {
    pub from: S,
    pub to: S,
    pub eased: f64,
}

pub fn ease_out(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

pub fn transition<S: Copy>(steps: &[Step<S>], rest: S, frame: f64) -> Transition<S> {
    let mut ordered: Vec<(usize, &Step<S>)> = steps.iter().enumerate().collect();
    ordered.sort_by(|a, b| a.1.at.total_cmp(&b.1.at).then(a.0.cmp(&b.0)));
    let started: Vec<&Step<S>> = ordered
        .into_iter()
        .map(|(_, step)| step)
        .filter(|step| step.at <= frame)
        .collect();
    let kept: Vec<&Step<S>> = started
        .iter()
        .enumerate()
        .filter(|(i, step)| {
            started
                .get(i + 1)
                .is_none_or(|next| !exact_eq(next.at, step.at))
        })
        .map(|(_, step)| *step)
        .collect();
    let Some(to) = kept.last() else {
        return Transition {
            from: rest,
            to: rest,
            eased: ease_out(1.0),
        };
    };
    let from = kept
        .len()
        .checked_sub(2)
        .and_then(|i| kept.get(i))
        .map_or(rest, |step| step.state);
    let progress = if to.duration > 0.0 {
        ((frame - to.at) / to.duration).clamp(0.0, 1.0)
    } else {
        1.0
    };
    Transition {
        from,
        to: to.state,
        eased: ease_out(progress),
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film tweens each value with a separate multiply and add"
)]
pub fn tween(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
