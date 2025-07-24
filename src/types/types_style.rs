use serde::Deserialize;
use serde::Serialize;

/// Defines terminal color codes per color
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    Default,

    Black,
    GrayDark,
    GrayLight,
    White,

    Red,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,

    BrightRed,
    BrightYellow,
    BrightGreen,
    BrightCyan,
    BrightBlue,
    BrightMagenta,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct Theme {
    pub color_base    : Color,
    pub color_selected: Color,
    pub is_selected   : bool,
    pub bold          : bool,
}

impl Theme {
    pub fn recolor(&self, color: Color) -> Theme {
        Theme {
            color_base: color,
            color_selected: self.color_selected,
            is_selected: self.is_selected,
            bold: self.bold,
        }
    }
}
