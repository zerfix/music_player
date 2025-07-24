use crate::types::types_style::Theme;
use crate::CONFIG;
use crate::ENVIRONMENT;
use crate::config::conf_color::ConfColor;
use crate::ui::utils::ui_text_util::fit_text_to_term;
use crate::types::types_style::Color;

//-////////////////////////////////////////////////////////////////////////////
fn bit4(color: Color, fg: bool, _conf_color: &ConfColor, output: &mut String, _buffer: &mut itoa::Buffer) {
    let code = match (fg, color) {
        (_    , Color::Default      ) => {return;}

        (true , Color::Black        ) =>  "30",
        (true , Color::GrayDark     ) =>  "90",
        (true , Color::GrayLight    ) =>  "37",
        (true , Color::White        ) =>  "97",

        (true , Color::Red          ) =>  "31",
        (true , Color::Yellow       ) =>  "33",
        (true , Color::Green        ) =>  "32",
        (true , Color::Cyan         ) =>  "36",
        (true , Color::Blue         ) =>  "34",
        (true , Color::Magenta      ) =>  "35",

        (true , Color::BrightRed    ) =>  "91",
        (true , Color::BrightYellow ) =>  "93",
        (true , Color::BrightGreen  ) =>  "92",
        (true , Color::BrightCyan   ) =>  "96",
        (true , Color::BrightBlue   ) =>  "94",
        (true , Color::BrightMagenta) =>  "95",

        (false, Color::Black        ) =>  "40",
        (false, Color::GrayDark     ) => "100",
        (false, Color::GrayLight    ) =>  "47",
        (false, Color::White        ) => "107",

        (false, Color::Red          ) =>  "41",
        (false, Color::Yellow       ) =>  "43",
        (false, Color::Green        ) =>  "42",
        (false, Color::Cyan         ) =>  "46",
        (false, Color::Blue         ) =>  "44",
        (false, Color::Magenta      ) =>  "45",

        (false, Color::BrightRed    ) => "101",
        (false, Color::BrightYellow ) => "103",
        (false, Color::BrightGreen  ) => "102",
        (false, Color::BrightCyan   ) => "106",
        (false, Color::BrightBlue   ) => "104",
        (false, Color::BrightMagenta) => "105",
    };

    output.push(';');
    output.push_str(code);
}

fn bit24(color: Color, fg: bool, conf_color: &ConfColor, output: &mut String, buffer: &mut itoa::Buffer) {
    let prefix = match fg {
        true  => 38,
        false => 48,
    };
    let [r, g, b] = match color {
        Color::Default => {return;}

        Color::Black     => conf_color.black,
        Color::GrayDark  => conf_color.gray_dark,
        Color::GrayLight => conf_color.gray_light,
        Color::White     => conf_color.white,

        Color::Red     => conf_color.red,
        Color::Yellow  => conf_color.yellow,
        Color::Green   => conf_color.green,
        Color::Cyan    => conf_color.cyan,
        Color::Blue    => conf_color.blue,
        Color::Magenta => conf_color.magenta,

        Color::BrightRed     => conf_color.bright_red,
        Color::BrightYellow  => conf_color.bright_yellow,
        Color::BrightGreen   => conf_color.bright_green,
        Color::BrightCyan    => conf_color.bright_cyan,
        Color::BrightBlue    => conf_color.bright_blue,
        Color::BrightMagenta => conf_color.bright_magenta,
    };

    for n in [prefix,2,r,g,b] {
        output.push(';');
        output.push_str(buffer.format(n));
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub struct TermState {
    pub frame       : String,
    pub text_buf    : String,
    pub num_buf     : itoa::Buffer,
    pub conf_color  : ConfColor,
    pub background  : Color,
    pub render_color: fn(Color, bool, &ConfColor, &mut String, &mut itoa::Buffer),
}

impl TermState {
    pub fn new() -> TermState {
        let config = CONFIG.get().unwrap();
        let env    = ENVIRONMENT.get().unwrap();
        TermState {
            frame: String::with_capacity(32 * 1024),
            text_buf: String::with_capacity(256),
            num_buf: itoa::Buffer::new(),
            conf_color: config.color,
            background: config.theme.background,
            render_color: match env.truecolor && config.color.custom_rgb_colors {
                false => bit4,
                true  => bit24,
            },
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
    pub fn style(&mut self, fg: Color, bg: Color, bold: bool) {
        let bg = match bg {
            Color::Default => self.background,
            _ => bg,
        };

        self.frame.push_str("\x1B[0");
        if bold {self.frame.push_str(";1")}
        (self.render_color)(fg, true , &self.conf_color, &mut self.frame, &mut self.num_buf);
        (self.render_color)(bg, false, &self.conf_color, &mut self.frame, &mut self.num_buf);
        self.frame.push('m');
    }

    pub fn style_theme(&mut self, theme: Theme) {
        match theme.is_selected {
           true  => self.style(Color::Black    , theme.color_selected, theme.bold),
           false => self.style(theme.color_base, Color::Default      , theme.bold),
        }
    }

    pub fn style_empty(&mut self) {
        self.style(
            Color::Default,
            Color::Default,
            false,
        )
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
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
