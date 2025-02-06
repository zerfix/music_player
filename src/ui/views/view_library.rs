use std::iter::repeat;
use unicode_width::UnicodeWidthStr;
use crate::state::state_library::LibraryColumn;
use crate::state::state_library::LibraryTab;
use crate::state::state_playlist::PlaybackState;
use crate::tasks::listener_tui::RenderDataCommon;
use crate::types::types_library_entry::LibraryFilterEntry;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_tui::TermState;
use crate::types::types_tui::Color;
use crate::types::types_tui::Format;
use crate::types::types_tui::TermSize;
use crate::ui::utils::ui_text_util::term_text;


//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct RenderDataViewLibrary {
    pub column_selected: LibraryColumn,
    pub tabs: Vec<LibraryTab>,
    pub tab_selected: LibraryTab,
    pub left: Vec<LibraryFilterEntry>,
    pub left_selected: usize,
    pub right: Vec<TrackFile>,
    pub right_selected: usize,
}

pub fn draw_library_view(
    output: &mut TermState,
    size: TermSize,
    common: &RenderDataCommon,
    view: RenderDataViewLibrary,
) {
    let filter_width = (size.width / 3).min(45);
    let track_width = size.width - filter_width - 1;

    render_header(
        output,
        size.width,
        view.tab_selected,
    );

    for i in 0..size.height.saturating_sub(1) {
        output.newline();

        match view.left.get(i).copied() {
            Some(filter) => render_filter_row(
                output,
                filter_width,
                filter,
                common.playlist.get_playback_state_for_filter(filter),
                view.column_selected == LibraryColumn::Filter,
                i == view.left_selected,
            ),
            None => for _ in 0..filter_width {output.push(" ")} ,
        };

        output.format(Format::color(Color::Blue, Color::Default));
        output.push("┃");

        match view.right.get(i).map(|e| (e.is_album_padding, e)) {
            Some((true, track)) => render_album_row(
                output,
                track_width,
                *track,
            ),
            Some((false, track)) => render_track_row(
                output,
                track_width,
                *track,
                common.playlist.get_playback_state_for_track(track.id_track),
                view.column_selected == LibraryColumn::Tracks,
                i == view.right_selected,
            ),
            None => for _ in 0..track_width {output.push(" ")},
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
fn render_header(
    output: &mut TermState,
    width: usize,
    library_tab: LibraryTab,
) {
    output.format(Format{
        fg: Color::Black,
        bg: Color::Blue,
        bold: true,
        italic: false,
    });

    let name: &'static str = library_tab.into();
    let len = 2 + name.len();

    output.push("  ");
    output.push(name);

    for _ in 0..width-len {
        output.push(" ");
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
fn render_filter_row(
    output: &mut TermState,
    width: usize,
    entry: LibraryFilterEntry,
    playback_state: PlaybackState,
    is_active: bool,
    is_selected: bool,
) {
    let playback_fg = match (is_selected, playback_state) {
        (true , _                     ) => Color::Black,
        (false, PlaybackState::None   ) => Color::Default,
        (false, PlaybackState::Played ) => Color::Red,
        (false, PlaybackState::Playing) => Color::Yellow,
        (false, PlaybackState::Qued   ) => Color::Green,
    };
    let playback = match playback_state {
        PlaybackState::None    => " ",
        PlaybackState::Played  => "-",
        PlaybackState::Playing => ">",
        PlaybackState::Qued    => "+",
    };

    let (fg, bg) = match (is_selected, is_active) {
        (false, _    ) => (Color::Default, Color::Default),
        (true , false) => (Color::Black  , Color::Red    ),
        (true , true ) => (Color::Black  , Color::Cyan   ),
    };
    let name = term_text(entry.name().to_string(), width.saturating_sub(3));

    let mut format = Format::color(playback_fg, bg);
    output.format(format);
    output.push(playback);

    output.format(*format.fg(fg));
    output.push(" ");
    output.push(&name);
    output.push(" ");
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
fn render_album_row(
    output: &mut TermState,
    width: usize,
    track: TrackFile,
) {
    let album_name = track.album_title.map(|n| n.to_string()).unwrap_or("<missing>".to_string());
    let year = match track.year {
        Some(year) => &year.to_string(),
        None       => "----",
    };

    let album_len = album_name.width().min(width.saturating_sub(9));
    let album_name = term_text(album_name, album_len);

    let static_size = album_name.width() + year.width() + 4;
    let dyn_len = width.saturating_sub(static_size);

    let mut format = Format::new();
    format.bold = true;
    output.format(format);

    output.push(" ");
    output.push(&album_name);
    output.push(" ");

    output.format(*format.fg(Color::Cyan));
    output.extend(repeat('⎯').take(dyn_len));

    output.format(*format.fg(Color::Default));
    output.push(" ");
    output.push(year);
    output.push(" ");
}

fn render_track_row(
    output: &mut TermState,
    widht: usize,
    track: TrackFile,
    playback_state: PlaybackState,
    is_active: bool,
    is_selected: bool,
) {
    let playback = match playback_state {
        PlaybackState::None    => " ",
        PlaybackState::Played  => "-",
        PlaybackState::Playing => ">",
        PlaybackState::Qued    => "+",
    };

    let artist_name  = match track.track_artist {
        Some(artist) => " - ".to_string() + &artist,
        None         => "".to_string(),
    };
    let track_number = match track.track_number {
        Some(track) => &format!("{:02}", track),
        None        => "--",
    };

    let seperate = |time: u64| (time % 60, time / 60);
    let duration_sec = track.duration.as_secs();
    let duration = match duration_sec {
        ..3600 => {
            let (seconds, minutes) = seperate(duration_sec);
            format!("{:02}:{:02}", minutes, seconds)
        },
        3600.. => {
            let (seconds, minutes) = seperate(duration_sec);
            let (minutes, hours  ) = seperate(minutes);
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        },
    };

    let static_len  = track_number.width() + duration.width() + 7;
    let dynamic_len = widht - static_len;

    let artist_show = track.track_artist != track.album_artist;
    let track_len   = match artist_show {
        false => dynamic_len,
        true  => track.track_title.width().min(dynamic_len),
    };
    let artist_len  = match artist_show {
        false => 0,
        true  => dynamic_len.saturating_sub(track_len),
    };

    let track_name: &str = &term_text(track.track_title.to_string(), track_len);
    let artist_name = match artist_show {
        false => term_text("".to_string(), artist_len),
        true  => term_text(artist_name, artist_len),
    };

    let (fg_d, fg_y, bg) = match (is_selected, is_active) {
        (false, _    ) => (Color::Default, Color::Yellow, Color::Default),
        (true , false) => (Color::Black  , Color::Black , Color::Red    ),
        (true , true ) => (Color::Black  , Color::Black , Color::Cyan   ),
    };
    let playback_fg = match (is_selected, playback_state) {
        (true , _                     ) => Color::Black,
        (false, PlaybackState::None   ) => Color::Default,
        (false, PlaybackState::Played ) => Color::Red,
        (false, PlaybackState::Playing) => Color::Yellow,
        (false, PlaybackState::Qued   ) => Color::Green,
    };

    let mut format = Format::color(playback_fg, bg);
    output.format(format);
    output.push(playback);
    output.push(" ");

    output.format(*format.fg(fg_y));
    output.push(track_number);
    output.push(" ");

    output.format(*format.fg(fg_d));
    output.push(&track_name);
    output.push(" ");

    if artist_show {
        output.format(*format.fg(Color::Gray));
    }
    output.push(&artist_name);
    output.push(" ");

    output.format(*format.fg(fg_y));
    output.push(&duration);
    output.push("  ");
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
