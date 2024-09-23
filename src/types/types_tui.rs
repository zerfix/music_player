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

#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub enum TermColor {
    Default,
    
    Red,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
    Black,
    White,
    
    BrightRed,
    BrightYellow ,
    BrightGreen,
    BrightCyan,
    BrightBlue,
    BrightMagenta,
    BrightBlack,
    BrightWhite,
    
    RGB{r: u8, g: u8, b: u8},
}

pub struct TermState {
    output: String,
    fg    : TermColor,
    bg    : TermColor,
}

impl TermState{
    pub fn new() -> TermState {
        TermState {
            output: String::with_capacity(4 * 1024),
            fg    : TermColor::Default,
            bg    : TermColor::Default,
        }
    }

    /// Prepares for new view
    pub fn clear(&mut self) {
        self.output.clear();
        self.fg = TermColor::Default;
        self.bg = TermColor::Default;
    }

    pub fn push(&mut self, s: &str) {
        self.output.push_str(s);
    }

    /// Finish rendering and output
    pub fn output(&mut self) -> &str {
        self.output.push_str("\x1B[0m");
        &self.output
    }

    /// Set foreground color
    pub fn fg(&mut self, color: TermColor) {
        if self.fg != color {
            let code = match color {
                TermColor::Default => "\x1B[39m",
 
                TermColor::Red     => "\x1B[31m",
                TermColor::Yellow  => "\x1B[33m",
                TermColor::Green   => "\x1B[32m",
                TermColor::Cyan    => "\x1B[36m",
                TermColor::Blue    => "\x1B[34m",
                TermColor::Magenta => "\x1B[35m",
                TermColor::Black   => "\x1B[30m",
                TermColor::White   => "\x1B[37m",

                TermColor::BrightRed     => "\x1B[91m",
                TermColor::BrightYellow  => "\x1B[93m",
                TermColor::BrightGreen   => "\x1B[92m",
                TermColor::BrightCyan    => "\x1B[96m",
                TermColor::BrightBlue    => "\x1B[94m",
                TermColor::BrightMagenta => "\x1B[95m",
                TermColor::BrightBlack   => "\x1B[90m",
                TermColor::BrightWhite   => "\x1B[97m",

                TermColor::RGB{r,g,b} => &format!{"\x1B[32;2;{};{};{}m", r, g, b},
            };
            self.fg = color;
            self.output.push_str(code);
        } 
    }

    /// Set background color
    pub fn bg(&mut self, color: TermColor) {
        if self.bg != color {
            let code = match color {
                TermColor::Default => "\x1B[49m",
 
                TermColor::Red     => "\x1B[41m",
                TermColor::Yellow  => "\x1B[43m",
                TermColor::Green   => "\x1B[42m",
                TermColor::Cyan    => "\x1B[46m",
                TermColor::Blue    => "\x1B[44m",
                TermColor::Magenta => "\x1B[45m",
                TermColor::Black   => "\x1B[40m",
                TermColor::White   => "\x1B[47m",

                TermColor::BrightRed     => "\x1B[101m",
                TermColor::BrightYellow  => "\x1B[103m",
                TermColor::BrightGreen   => "\x1B[102m",
                TermColor::BrightCyan    => "\x1B[106m",
                TermColor::BrightBlue    => "\x1B[104m",
                TermColor::BrightMagenta => "\x1B[105m",
                TermColor::BrightBlack   => "\x1B[100m",
                TermColor::BrightWhite   => "\x1B[107m",

                TermColor::RGB{r,g,b} => &format!{"\x1B[48;2;{};{};{}m", r, g, b},
            };
            self.bg = color;
            self.output.push_str(code);
        } 
    }

    /// Set bold state
    pub fn bold(&mut self) {
        self.output.push_str("\x1B[1m");
    }

    /// Set italic state
    pub fn italic(&mut self, change: bool) {
        self.output.push_str("\x1B[3m");
    }

    pub fn reset_format(&mut self) {
        self.fg = TermColor::Default;
        self.bg = TermColor::Default;
        self.output.push_str("\x1B[m");
    }
}
