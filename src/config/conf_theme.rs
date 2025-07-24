use crate::types::types_style::Color;
use serde::Deserialize;
use serde::Serialize;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Copy)]
pub struct ConfTheme {
    pub background : Color,
    pub border     : Color,

    pub album_text        : Color,
    pub album_divider     : Color,
    pub track_highlight   : Color,
    pub track_artist_name : Color,

    pub selectable_normal            : Color,
    pub selectable_highlight_active  : Color,
    pub selectable_highlight_inactive: Color,

    pub icon_color_done   : Color,
    pub icon_color_playing: Color,
    pub icon_color_queued : Color,
}

impl ConfTheme {
    pub fn init() -> ConfTheme {
        ConfTheme {
            background: Color::Default,
            border    : Color::Blue,

            album_text       : Color::Default,
            album_divider    : Color::Cyan,
            track_highlight  : Color::Yellow,
            track_artist_name: Color::GrayDark,

            selectable_normal            : Color::Default,
            selectable_highlight_active  : Color::Cyan,
            selectable_highlight_inactive: Color::Red,

            icon_color_done   : Color::Red,
            icon_color_playing: Color::Yellow,
            icon_color_queued : Color::Green,
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
