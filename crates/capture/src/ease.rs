use opsis_types::numbers;

const NEWTON_ITERATIONS: usize = 4;
const NEWTON_MIN_SLOPE: f64 = 0.001;
const SUBDIVISION_PRECISION: f64 = 0.000_000_1;
const SUBDIVISION_MAX_ITERATIONS: usize = 10;
const SPLINE_TABLE_SIZE: usize = 11;
const SAMPLE_STEP: f64 = 1.0 / numbers::float(SPLINE_TABLE_SIZE - 1);
const CAMERA: Bezier = Bezier::new(0.2, 0.2, 0.15, 1.0);

fn coefficients(a1: f64, a2: f64) -> (f64, f64, f64) {
    (1.0 - 3.0 * a2 + 3.0 * a1, 3.0 * a2 - 6.0 * a1, 3.0 * a1)
}

fn calc(t: f64, a1: f64, a2: f64) -> f64 {
    let (a, b, c) = coefficients(a1, a2);
    ((a * t + b) * t + c) * t
}

fn slope(t: f64, a1: f64, a2: f64) -> f64 {
    let (a, b, c) = coefficients(a1, a2);
    3.0 * a * t * t + 2.0 * b * t + c
}

pub struct Bezier {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl Bezier {
    pub const fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    fn samples(&self) -> [f32; SPLINE_TABLE_SIZE] {
        std::array::from_fn(|i| {
            numbers::narrow(calc(numbers::float(i) * SAMPLE_STEP, self.x1, self.x2))
        })
    }

    fn subdivide(&self, x: f64, mut lo: f64, mut hi: f64) -> f64 {
        let mut t;
        let mut i = 0;
        loop {
            t = lo + (hi - lo) / 2.0;
            let current = calc(t, self.x1, self.x2) - x;
            if current > 0.0 {
                hi = t;
            } else {
                lo = t;
            }
            i += 1;
            if current.abs() <= SUBDIVISION_PRECISION || i >= SUBDIVISION_MAX_ITERATIONS {
                return t;
            }
        }
    }

    fn newton(&self, x: f64, mut t: f64) -> f64 {
        for _ in 0..NEWTON_ITERATIONS {
            let current_slope = slope(t, self.x1, self.x2);
            if current_slope == 0.0 {
                return t;
            }
            t -= (calc(t, self.x1, self.x2) - x) / current_slope;
        }
        t
    }

    fn t_for_x(&self, x: f64) -> f64 {
        let samples = self.samples();
        let mut start = 0.0;
        let mut current = 1;
        while current != SPLINE_TABLE_SIZE - 1 && f64::from(samples[current]) <= x {
            start += SAMPLE_STEP;
            current += 1;
        }
        current -= 1;
        let (here, next) = (f64::from(samples[current]), f64::from(samples[current + 1]));
        let guess = start + (x - here) / (next - here) * SAMPLE_STEP;
        let initial_slope = slope(guess, self.x1, self.x2);
        if initial_slope >= NEWTON_MIN_SLOPE {
            self.newton(x, guess)
        } else if initial_slope == 0.0 {
            guess
        } else {
            self.subdivide(x, start, start + SAMPLE_STEP)
        }
    }

    pub fn at(&self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        if self.x1 == self.y1 && self.x2 == self.y2 {
            return x;
        }
        if x == 0.0 || x == 1.0 {
            return x;
        }
        calc(self.t_for_x(x), self.y1, self.y2)
    }
}

pub fn camera_ease(t: f64) -> f64 {
    CAMERA.at(t)
}
