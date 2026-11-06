use opsis_types::workflow::Position;

pub const SNAP_GRID: f64 = 15.0;
pub const FILE_STAGGER: f64 = 30.0;

pub fn snap_to_grid(at: Position) -> Position {
    Position {
        x: (at.x / SNAP_GRID).round() * SNAP_GRID,
        y: (at.y / SNAP_GRID).round() * SNAP_GRID,
    }
}

pub fn staggered(at: Position, index: usize) -> Position {
    let step = f64::from(u32::try_from(index).unwrap_or(u32::MAX)) * FILE_STAGGER;
    snap_to_grid(Position {
        x: at.x + step,
        y: at.y + step,
    })
}
