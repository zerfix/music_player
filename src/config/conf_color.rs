use serde::Deserialize;
use serde::Serialize;

//-////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Copy)]
pub struct ConfRgb {
    r: u8,
    g: u8,
    b: u8,
}
impl ConfRgb {
    pub fn to_tuple(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Copy)]
pub struct ConfColor {
    pub custom_rgb_colors: bool,

    pub black         : [u8;3],
    pub gray_dark     : [u8;3],
    pub gray_light    : [u8;3],
    pub white         : [u8;3],

    pub red           : [u8;3],
    pub yellow        : [u8;3],
    pub green         : [u8;3],
    pub cyan          : [u8;3],
    pub blue          : [u8;3],
    pub magenta       : [u8;3],

    pub bright_red    : [u8;3],
    pub bright_yellow : [u8;3],
    pub bright_green  : [u8;3],
    pub bright_cyan   : [u8;3],
    pub bright_blue   : [u8;3],
    pub bright_magenta: [u8;3],
}

impl ConfColor {
    pub fn init() -> ConfColor {
        ConfColor {
            custom_rgb_colors: false,

            black         : [  0,   0,   0],
            gray_dark     : [150, 150, 150],
            gray_light    : [200, 200, 200],
            white         : [255, 255, 255],

            red           : [224,  83,  83],
            yellow        : [210, 184,  67],
            green         : [147, 191,  83],
            cyan          : [100, 212, 192],
            blue          : [060, 165, 210],
            magenta       : [204, 127, 193],

            bright_red    : [234, 086, 108],
            bright_yellow : [211, 225,  73],
            bright_green  : [119, 246, 101],
            bright_cyan   : [126, 235, 233],
            bright_blue   : [ 95, 179, 244],
            bright_magenta: [241, 137, 215],
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
