use crate::state::state_library::LibraryColumn;
use crate::state::state_library::LibrarySelectMode;
use crate::state::state_library::LibraryTab;
use crate::state::state_playlist::PlaybackState;
use crate::tasks::listener_tui::RenderDataCommon;
use crate::types::types_library_entry::LibraryFilterEntry;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_tui::Color;
use crate::types::types_tui::Format;
use crate::types::types_tui::TermState;
use crate::ui::utils::ui_loading_icon_util::loading_icon;
use crate::ui::utils::ui_time_util::render_duration;
use arrayvec::ArrayString;
use std::fmt::Write;
use std::iter::repeat;
use std::str::FromStr;
use unicode_width::UnicodeWidthStr;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct RenderDataViewLibrary {
    pub column_selected: LibraryColumn,
    pub tab_selected: LibraryTab,
    pub track_select_mode: LibrarySelectMode,
    pub list_height: usize,
    pub left: Vec<LibraryFilterEntry>,
    pub left_selected: usize,
    pub right: Vec<TrackFile>,
    pub right_selected: usize,
}

pub fn draw_library_view(
    output: &mut TermState,
    common: &RenderDataCommon,
    view: RenderDataViewLibrary,
) {
    let width = common.term_size.width;
    let filter_width = (width / 3).min(45);
    let track_width = width - filter_width - 1;

    render_header(
        output,
        width,
        filter_width,
        common,
        view.tab_selected,
        view.track_select_mode,
    );

    for i in 0..view.list_height {
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
            None => {
                output.format(Format { fg: Color::Default, bg: Color::Default, bold: false });
                output.frame.extend(repeat(' ').take(filter_width));
            },
        };

        output.format(Format{fg: Color::Blue, bg: Color::Default, bold: false});
        output.frame.push('┃');

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
            None => {
                output.format(Format { fg: Color::Default, bg: Color::Default, bold: false });
                output.frame.extend(repeat(' ').take(track_width));
            },
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
fn render_header(
    output: &mut TermState,
    width: usize,
    filter_width: usize,
    common: &RenderDataCommon,
    library_tab: LibraryTab,
    select_mode: LibrarySelectMode,
) {
    output.format(Format{fg: Color::Black, bg: Color::Blue, bold: true});

    // loading icon
    {
        let loading_icon = match common.is_scanning {
            true  => loading_icon(common.interval),
            false => ' ',
        };
        output.frame.push(loading_icon);
    }

    output.frame.push(' ');

    // filter tab
    {
        let tab_name: &'static str = library_tab.into();
        output.fit_str(Some("filter by "), tab_name, filter_width.saturating_sub(2));
    }

    // select mode
    {
        let select_name: &'static str = select_mode.into();
        output.fit_str(Some("   select "), select_name, width.saturating_sub(filter_width));
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
    let format = match (is_selected, is_active) {
        (false, _    ) => Format{fg: Color::Default, bg: Color::Default, bold: false},
        (true , false) => Format{fg: Color::Black  , bg: Color::Red    , bold: false},
        (true , true ) => Format{fg: Color::Black  , bg: Color::Cyan   , bold: false},
    };

    // playback indicator
    {
        let format = match (is_selected, playback_state) {
            (true , _                     ) => format,
            (false, PlaybackState::None   ) => Format{fg: Color::Default, bg: Color::Default, bold: false},
            (false, PlaybackState::Played ) => Format{fg: Color::Red    , bg: Color::Default, bold: false},
            (false, PlaybackState::Playing) => Format{fg: Color::Yellow , bg: Color::Default, bold: false},
            (false, PlaybackState::Queued ) => Format{fg: Color::Green  , bg: Color::Default, bold: false},
        };
        let icon = match playback_state {
            PlaybackState::None    => ' ',
            PlaybackState::Played  => '-',
            PlaybackState::Playing => '>',
            PlaybackState::Queued  => '+',
        };
        output.format(format);
        output.frame.push(icon);
    }

    // filter name
    {
        let name = entry.name();
        let width = width.saturating_sub(3);

        output.format(format);
        output.frame.push(' ');
        output.fit_str(None, &name, width);
        output.frame.push(' ');
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
fn render_album_row(
    output: &mut TermState,
    width: usize,
    track: TrackFile,
) {
    let len_padding = 3;
    let len_year    = 4;
    let len_dynamic = width.saturating_sub(len_padding + len_year);

    output.frame.push(' ');

    // album name
    let len_line = {
        let format = Format{fg: Color::Default, bg: Color::Default, bold: true};
        let album_name = track.album_title.unwrap_or(ArrayString::from_str("<missing>").unwrap());
        let len_album = match len_dynamic.saturating_sub(album_name.width()) {
            ..2 => len_dynamic,
            2.. => album_name.width(),
        };
        output.format(format);
        output.fit_str(None, &album_name, len_album);

        len_dynamic.saturating_sub(album_name.width())
    };

    // blue line
    match len_line {
        0   => {},
        1   => output.frame.push(' '),
        2.. => {
            output.frame.push(' ');
            output.format(Format{fg: Color::Cyan, bg: Color::Default, bold: true});
            output.frame.extend(repeat('⎯').take(len_line-1));
        }
    }

    output.frame.push(' ');

    // year
    {
        let format = Format{fg: Color::Default, bg: Color::Default, bold: true};
        output.format(format);
        match track.year {
            Some(year) => output.frame.push_str(output.num_buf.format(year)),
            None       => output.frame.extend(repeat('-').take(4))
        };
    }

    output.frame.push(' ');
}

fn render_track_row(
    output: &mut TermState,
    width: usize,
    track: TrackFile,
    playback_state: PlaybackState,
    is_active: bool,
    is_selected: bool,
) {
    let len_padding  = 5;
    let len_playback = 1;
    let len_track    = 2;
    let len_duration = 5 + 3 * (track.duration.as_secs() > 3600) as usize;
    let len_dynamic  = width.saturating_sub(len_padding + len_playback + len_track + len_duration);

    let format_white = match (is_selected, is_active) {
        (false, _    ) => Format{fg: Color::Default, bg: Color::Default, bold: false},
        (true , false) => Format{fg: Color::Black  , bg: Color::Red    , bold: false},
        (true , true ) => Format{fg: Color::Black  , bg: Color::Cyan   , bold: false},
    };
    let format_yellow = match (is_selected, is_active) {
        (false, _    ) => Format{fg: Color::Yellow, bg: Color::Default, bold: false},
        (true , false) => Format{fg: Color::Black , bg: Color::Red    , bold: false},
        (true , true ) => Format{fg: Color::Black , bg: Color::Cyan   , bold: false},
    };

    // playback indicator
    {
        let format = match (is_selected, playback_state) {
            (true , _                     ) => format_white,
            (false, PlaybackState::None   ) => Format{fg: Color::Default, bg: Color::Default, bold: false},
            (false, PlaybackState::Played ) => Format{fg: Color::Red    , bg: Color::Default, bold: false},
            (false, PlaybackState::Playing) => Format{fg: Color::Yellow , bg: Color::Default, bold: false},
            (false, PlaybackState::Queued ) => Format{fg: Color::Green  , bg: Color::Default, bold: false},
        };
        let icon = match playback_state {
            PlaybackState::None    => ' ',
            PlaybackState::Played  => '-',
            PlaybackState::Playing => '>',
            PlaybackState::Queued  => '+',
        };
        output.format(format);
        output.frame.push(icon);
    }

    output.frame.push(' ');

    // track number
    {
        output.format(format_yellow);
        match track.track_number {
            None        => output.frame.extend(repeat('-').take(2)),
            Some(track) => {
                output.text_buf.clear();
                write!(&mut output.text_buf, "{:02}", track).unwrap();
                output.frame.push_str(&output.text_buf);
            },
        };
    }

    output.frame.push(' ');

    // track name
    let len_artist = {
        let track_name = track.track_title;
        let len_track = match len_dynamic.saturating_sub(track_name.width()) {
            0   => len_dynamic,
            1.. => track_name.width(),
        };
        output.format(format_white);
        output.fit_str(None, &track_name, len_track);
        len_dynamic.saturating_sub(track_name.width())
    };

    // artist name
    match (len_artist, track.track_artist != track.album_artist, track.track_artist) {
        ( 0 , _    , _   ) => {},
        (_  , false, _   ) |
        (_  , _    , None) => output.frame.extend(repeat(' ').take(len_artist)),
        (1.., true , Some(artist)) => {
            let format = match is_selected {
                true  => format_white,
                false => Format{fg: Color::Gray, bg: Color::Default, bold: false},
            };
            output.format(format);
            output.fit_str(Some(" - "), &artist, len_artist);
        },
    }

    output.frame.push(' ');

    // track duration
    {
        output.text_buf.clear();
        render_duration(&mut output.text_buf, track.duration);
        output.format(format_yellow);
        output.frame.push_str(&output.text_buf);
    }

    output.frame.extend(repeat(' ').take(2));
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
