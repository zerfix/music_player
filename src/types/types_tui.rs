use color_eyre::Result;

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

    pub fn term_format_codes(&self) -> String {
        let mut buffer = String::with_capacity(42);
        buffer.push_str("\x1B[0");
        if self.bold {
            buffer.push_str(";1");
        }
        if self.italic {
            buffer.push_str(";3");
        }
        if let Some((r,g,b)) = self.fg.rgb() {
            buffer.push_str(&format!(";38;2;{};{};{}",r,g,b));
        }
        if let Some((r,g,b)) = self.bg.rgb() {
            buffer.push_str(&format!(";48;2;{};{};{}",r,g,b));
        }
        buffer.push('m');
        buffer
    }
}

pub struct TermState {
    output: String,
    format: Format,
}

impl TermState{
    pub fn new() -> TermState {
        TermState {
            output: String::new(),
            format: Format::new(),
        }
    }

    /// Prepares for new view
    pub fn clear(&mut self) {
        self.output.clear();
        self.reset_format();
    }

    pub fn push(&mut self, s: &str) {
        self.output.push_str(s);
    }

    pub fn newline(&mut self) {
        self.push("\r\n");
    }

    /// Finish rendering and output
    pub fn output(&mut self) -> &str {
        self.format(Format::new());
        &self.output
    }

    pub fn format(&mut self, format: Format) {
        if format == self.format {return}
        self.format = format;
        let term_codes = &format.term_format_codes();
        self.output.push_str(&term_codes);
    }

    pub fn go_to(&mut self, row: usize, column: usize) {
        self.push(&format!("\x1B[{},{}H", row, column));
    }

    pub fn reset_format(&mut self) {
        self.format(Format::new());
    }

    pub fn current_format(&self) -> &Format {
        &self.format
    }

    pub fn capacity(&self) -> usize {
        self.output.capacity()
    }
}
