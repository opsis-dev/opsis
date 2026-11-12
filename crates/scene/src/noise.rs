use opsis_types::numbers::{float, truncate};

use crate::random::random;

const HALF: usize = 256;
const F3: f64 = 1.0 / 3.0;
const G3: f64 = 1.0 / 6.0;
const FALLOFF: f64 = 0.6;
const SCALE: f64 = 32.0;
const GRADIENTS: [[f64; 3]; 12] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoiseField {
    perm: Vec<u8>,
}

fn permutation(mut next: impl FnMut() -> f64) -> Vec<u8> {
    let mut table: Vec<u8> = (0..=u8::MAX).collect();
    for i in 0..HALF - 1 {
        let offset: usize = truncate(next() * float(HALF - i));
        table.swap(i, i + offset);
    }
    let copy = table.clone();
    table.extend(copy);
    table
}

#[comemo::memoize]
fn seeded(key: String) -> NoiseField {
    let mut draws = 0_u32;
    NoiseField {
        perm: permutation(|| {
            draws += 1;
            random(format!("{key}:{draws}"))
        }),
    }
}

fn floor_index(value: f64) -> i32 {
    truncate(value.floor())
}

fn wrapped(value: i32) -> usize {
    usize::try_from(value & 255).unwrap_or(0)
}

type Corner = (usize, usize, usize);

const fn corners(x0: f64, y0: f64, z0: f64) -> (Corner, Corner) {
    if x0 >= y0 {
        if y0 >= z0 {
            ((1, 0, 0), (1, 1, 0))
        } else if x0 >= z0 {
            ((1, 0, 0), (1, 0, 1))
        } else {
            ((0, 0, 1), (1, 0, 1))
        }
    } else if y0 < z0 {
        ((0, 0, 1), (0, 1, 1))
    } else if x0 < z0 {
        ((0, 1, 0), (0, 1, 1))
    } else {
        ((0, 1, 0), (1, 1, 0))
    }
}

impl NoiseField {
    pub fn new(seed: &str) -> Self {
        seeded(seed.to_owned())
    }

    fn gradient(&self, index: usize) -> [f64; 3] {
        GRADIENTS[usize::from(self.perm[index]) % GRADIENTS.len()]
    }

    #[expect(
        clippy::suboptimal_flops,
        reason = "simplex-noise in the film rounds every step, and the pictures have to match it"
    )]
    fn corner(&self, cell: Corner, offset: Corner, at: [f64; 3]) -> f64 {
        let reach = FALLOFF - at[0] * at[0] - at[1] * at[1] - at[2] * at[2];
        if reach < 0.0 {
            return 0.0;
        }
        let (ii, jj, kk) = cell;
        let inner = usize::from(self.perm[kk + offset.2]);
        let middle = usize::from(self.perm[jj + offset.1 + inner]);
        let slope = self.gradient(ii + offset.0 + middle);
        let dot = slope[0] * at[0] + slope[1] * at[1] + slope[2] * at[2];
        let squared = reach * reach;
        squared * squared * dot
    }

    #[expect(
        clippy::suboptimal_flops,
        reason = "simplex-noise in the film rounds every step, and the pictures have to match it"
    )]
    pub fn at(&self, x: f64, y: f64, z: f64) -> f64 {
        let skew = (x + y + z) * F3;
        let (cell_x, cell_y, cell_z) = (
            floor_index(x + skew),
            floor_index(y + skew),
            floor_index(z + skew),
        );
        let unskew = f64::from(cell_x + cell_y + cell_z) * G3;
        let x0 = x - (f64::from(cell_x) - unskew);
        let y0 = y - (f64::from(cell_y) - unskew);
        let z0 = z - (f64::from(cell_z) - unskew);
        let (first, second) = corners(x0, y0, z0);
        let step = |o: usize| f64::from(u8::try_from(o).unwrap_or(0));
        let at1 = [
            x0 - step(first.0) + G3,
            y0 - step(first.1) + G3,
            z0 - step(first.2) + G3,
        ];
        let at2 = [
            x0 - step(second.0) + 2.0 * G3,
            y0 - step(second.1) + 2.0 * G3,
            z0 - step(second.2) + 2.0 * G3,
        ];
        let at3 = [
            x0 - 1.0 + 3.0 * G3,
            y0 - 1.0 + 3.0 * G3,
            z0 - 1.0 + 3.0 * G3,
        ];
        let cell = (wrapped(cell_x), wrapped(cell_y), wrapped(cell_z));
        SCALE
            * (self.corner(cell, (0, 0, 0), [x0, y0, z0])
                + self.corner(cell, first, at1)
                + self.corner(cell, second, at2)
                + self.corner(cell, (1, 1, 1), at3))
    }
}
