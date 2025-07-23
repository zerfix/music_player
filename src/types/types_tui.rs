use crate::ui::utils::ui_text_util::fit_text_to_term;

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
    pub const fn rgb(self) -> Option<(u8,u8,u8)> {
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
    pub fg  : Color,
    pub bg  : Color,
    pub bold: bool,
}

impl Format {
    pub fn output_term_codes(&self, output: &mut String, buffer: &mut itoa::Buffer) {
        output.push_str("\x1B[0");
        if self.bold {
            output.push_str(";1");
        }
        if let Some((r,g,b)) = self.fg.rgb() {
            for n in [38,2,r,g,b] {
                output.push(';');
                output.push_str(buffer.format(n));
            }
        }
        if let Some((r,g,b)) = self.bg.rgb() {
            for n in [48,2,r,g,b] {
                output.push(';');
                output.push_str(buffer.format(n));
            }
        }
        output.push('m');
    }
}

pub struct TermState {
    pub frame   : String,
    pub text_buf: String,
    pub num_buf : itoa::Buffer,
}

impl TermState {
    pub fn new() -> TermState {
        TermState {
            frame: String::with_capacity(32 * 1024),
            text_buf: String::with_capacity(256),
            num_buf: itoa::Buffer::new(),
        }
    }

    /// Prepares for new view
    pub fn clear(&mut self) {
        self.frame.clear();
    }

    /// Add newline
    pub fn newline(&mut self) {
        self.frame.push_str("\r\n");
    }

    /// Add terminal formatting codes to buffer
    pub fn format(&mut self, format: Format) {
        format.output_term_codes(&mut self.frame, &mut self.num_buf);
    }

    /// Add text and make sure it fits within n cells
    pub fn fit_str(&mut self, prefix: Option<&str>, string: &str, len: usize) {
        self.text_buf.clear();
        if let Some(prefix) = prefix {
            self.text_buf.push_str(prefix);
        }
        self.text_buf.push_str(string);
        fit_text_to_term(&mut self.text_buf, len);
        self.frame.push_str(&self.text_buf);
    }

    /// Return buffer
    pub fn output(&mut self) -> &str {
        &self.frame
    }
}
