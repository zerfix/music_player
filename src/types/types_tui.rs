use color_eyre::Result;
use std::fmt::Write;

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub struct TermSize {
    pub width: usize,
    pub height: usize,
}

impl TermSize {
    pub fn new() -> Result<TermSize> {
        let (columns, rows) = crossterm::terminal::size()?;
        Ok(TermSize{
            width: columns as usize,
            height: rows as usize,
        })
    }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub enum Color {
    Default,

    Black,
    GrayDark,
    Gray,
    GrayLight,
    White,

    Red,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
}

impl Color {
    pub fn rgb(self) -> Option<(u8,u8,u8)> {
        match self {
            Color::Default => None,

            Color::Black     => Some((  0,  0,  0)),
            Color::GrayDark  => Some(( 50, 50, 50)),
            Color::Gray      => Some((100,100,100)),
            Color::GrayLight => Some((150,150,150)),
            Color::White     => Some((200,200,200)),

            Color::Red     => Some((224, 83, 82)),
            Color::Yellow  => Some((210,184, 67)),
            Color::Green   => Some((147,191, 83)),
            Color::Cyan    => Some((100,212,192)),
            Color::Blue    => Some(( 60,165,210)),
            Color::Magenta => Some((204,127,193)),
        }
    }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub struct Format {
    pub fg    : Color,
    pub bg    : Color,
    pub bold  : bool,
    pub italic: bool,
}

impl Format {
    pub fn new() -> Format {
        Format{
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
            italic: false,
        }
    }

    pub fn color(fg: Color, bg: Color) -> Format {
        Format{
            fg,
            bg,
            bold: false,
            italic: false,
        }
    }

    pub fn fg(&mut self, fg: Color) -> &mut Format {
        self.fg = fg;
        self
    }

    pub fn bg(&mut self, bg: Color) -> &mut Format {
        self.bg = bg;
        self
    }

    pub fn bold(&mut self, bold: bool) -> &mut Format {
        self.bold = bold;
        self
    }

    pub fn italic(&mut self, italic: bool) -> &mut Format {
        self.italic = italic;
        self
    }

    pub fn output_term_codes(&self, buffer: &mut String) {
        buffer.reserve(42);
        buffer.push_str("\x1B[0");
        if self.bold {
            buffer.push_str(";1");
        }
        if self.italic {
            buffer.push_str(";3");
        }
        if let Some((r,g,b)) = self.fg.rgb() {
            write!(buffer, ";38;2;{:03};{:03};{:03}",r,g,b).unwrap();
        }
        if let Some((r,g,b)) = self.bg.rgb() {
            write!(buffer, ";48;2;{:03};{:03};{:03}",r,g,b).unwrap();
        }
        buffer.push('m');
    }
}

pub struct TermState {
    output: String,
}

impl TermState{
    pub fn new() -> TermState {
        TermState {
            output: String::new(),
        }
    }

    /// Prepares for new view
    pub fn clear(&mut self) {
        self.output.clear();
    }

    /// Push string to buffer
    pub fn push(&mut self, s: &str) {
        self.output.push_str(s);
    }

    /// Repeat character to buffer
    pub fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        self.output.extend(iter);
    }

    /// Add newline
    pub fn newline(&mut self) {
        self.push("\r\n");
    }

    /// Add terminal formatting codes to buffer
    pub fn format(&mut self, format: Format) {
        format.output_term_codes(&mut self.output);
    }

    /// Return buffer
    pub fn output(&mut self) -> &str {
        &self.output
    }
}
