use std::sync::LazyLock;

use opsis_types::actions::{ActionLog, CellGrid, Point, Rect, Viewport};
use opsis_types::numbers::whole;
use opsis_types::terminal::{TerminalCursor, TerminalLook, TerminalTheme};
use serde::Deserialize;

pub const DEFAULT_THEME_NAME: &str = "Default";
pub const TERMINAL_FONTS: [&str; 5] = [
    "Opsis Terminal",
    "Opsis Terminal Symbols",
    "Opsis Terminal Symbols 2",
    "Opsis Terminal CJK",
    "Opsis Terminal Emoji",
];
pub const DEFAULT_FONT_SIZE: u32 = 20;
pub const DEFAULT_LINE_HEIGHT: f64 = 1.2;
pub const DEFAULT_PADDING: u32 = 24;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ThemeEntry {
    pub name: String,
    pub fg: String,
    pub bg: String,
    pub cursor: String,
    pub palette: [String; 16],
    pub dark: bool,
}

impl ThemeEntry {
    pub fn theme(&self) -> TerminalTheme {
        TerminalTheme {
            name: self.name.clone(),
            fg: self.fg.clone(),
            bg: self.bg.clone(),
            cursor: self.cursor.clone(),
            palette: self.palette.clone(),
        }
    }
}

static THEMES: LazyLock<Vec<ThemeEntry>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("themes.generated.json")).expect("the terminal themes parse")
});

pub fn default_theme() -> TerminalTheme {
    TerminalTheme {
        name: DEFAULT_THEME_NAME.to_owned(),
        fg: "#dddddd".to_owned(),
        bg: "#171717".to_owned(),
        cursor: "#dddddd".to_owned(),
        palette: [
            "#282a2e", "#d74e6f", "#31bb71", "#d3e561", "#8056ff", "#ed61d7", "#04d7d7", "#bfbfbf",
            "#4d4d4d", "#fe5f86", "#00d787", "#ebff71", "#9b79ff", "#ff7aea", "#00fefe", "#e6e6e6",
        ]
        .map(str::to_owned),
    }
}

pub fn terminal_themes() -> &'static [ThemeEntry] {
    &THEMES
}

pub fn theme_named(name: &str) -> Option<TerminalTheme> {
    if name == DEFAULT_THEME_NAME {
        return Some(default_theme());
    }
    THEMES
        .iter()
        .find(|t| t.name == name)
        .map(ThemeEntry::theme)
}

pub fn theme_names() -> Vec<String> {
    std::iter::once(DEFAULT_THEME_NAME.to_owned())
        .chain(THEMES.iter().map(|t| t.name.clone()))
        .collect()
}

pub fn default_look(theme: Option<TerminalTheme>) -> TerminalLook {
    TerminalLook {
        theme: theme.unwrap_or_else(default_theme),
        font_size: DEFAULT_FONT_SIZE,
        line_height: DEFAULT_LINE_HEIGHT,
        letter_spacing: 0.0,
        padding: DEFAULT_PADDING,
        cursor: TerminalCursor::Program,
        cursor_blink: true,
    }
}

pub fn grid(look: &TerminalLook, cols: u16, rows: u16) -> CellGrid {
    CellGrid {
        cols,
        rows,
        width: look.cell_width(),
        height: look.cell_height(),
        padding: f64::from(look.padding),
    }
}

pub fn screen_size(grid: &CellGrid) -> Viewport {
    Viewport {
        width: whole(f64::from(grid.cols).mul_add(grid.width, 2.0 * grid.padding)),
        height: whole(f64::from(grid.rows).mul_add(grid.height, 2.0 * grid.padding)),
    }
}

struct Regrid {
    from: Option<CellGrid>,
    to: CellGrid,
}

impl Regrid {
    fn point(&self, p: Point) -> Point {
        let (col, row) = self.from.map_or((p.x, p.y), |g| (g.col(p.x), g.row(p.y)));
        Point {
            x: self.to.x(col),
            y: self.to.y(row),
        }
    }

    fn scale_x(&self) -> f64 {
        self.to.width / self.from.map_or(1.0, |g| g.width)
    }

    fn scale_y(&self) -> f64 {
        self.to.height / self.from.map_or(1.0, |g| g.height)
    }

    fn rect(&self, r: Rect) -> Rect {
        let at = self.point(Point { x: r.x, y: r.y });
        Rect {
            x: at.x,
            y: at.y,
            w: r.w * self.scale_x(),
            h: r.h * self.scale_y(),
        }
    }

    fn action(&self, a: &mut opsis_types::actions::Action) {
        a.point = a.point.map(|p| self.point(p));
        a.rect = a.rect.map(|r| self.rect(r));
        a.drop = a.drop.map(|r| self.rect(r));
        if let Some(path) = a.path.as_mut() {
            for p in path.iter_mut() {
                *p = self.point(*p);
            }
        }
    }
}

pub fn regrid_log(log: &mut ActionLog, to: CellGrid) {
    let regrid = Regrid {
        from: log.cells,
        to,
    };
    for action in &mut log.actions {
        regrid.action(action);
    }
    for sample in log.cursor_track.iter_mut().flatten() {
        let p = regrid.point(Point {
            x: sample.x,
            y: sample.y,
        });
        sample.x = p.x;
        sample.y = p.y;
    }
    log.viewport = screen_size(&to);
    log.cells = Some(to);
}

pub fn regrid_actions(actions: &mut [opsis_types::actions::Action], from: CellGrid, to: CellGrid) {
    let regrid = Regrid {
        from: Some(from),
        to,
    };
    for action in actions {
        regrid.action(action);
    }
}
