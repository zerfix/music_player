use crate::state::state_library::LibraryColumn;
use crate::state::state_library::LibraryTab;
use crate::types::types_library_entry::LibraryAlbumEntryData;
use crate::types::types_library_entry::LibraryFilterEntry;
use crate::types::types_library_entry::LibraryTrackEntry;
use crate::types::types_library_entry::LibraryTrackEntryData;
use crate::types::types_tui::TermState;
use crate::types::types_tui::TermColor;
use crate::types::types_tui::TermSize;
use crate::ui::utils::ui_text_util::term_text;


//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct RenderDataViewLibrary {
    pub column_selected: LibraryColumn,
    pub tabs: Vec<LibraryTab>,
    pub tabs_selected: LibraryTab,
    pub left: Vec<LibraryFilterEntry>,
    pub left_selected: usize,
    pub right: Vec<LibraryTrackEntry>,
    pub right_selected: usize,
}

pub fn draw_library_view(
    output: &mut TermState,
    size: TermSize,
    view: RenderDataViewLibrary,
) {
    let filter_width = 42;
    let track_width = size.width - filter_width - 1;
    info!("width: {}, filter: {}, separator: 1, track_width: {}", size.width, filter_width, track_width);

    for i in 0..size.height {
        match view.left.get(i) {
            Some(filter) => render_filter_row(
                output, 
                filter_width, 
                filter, 
                view.column_selected == LibraryColumn::Filter, 
                i == view.left_selected,
            ),
            None => for _ in 0..filter_width {output.push(" ")} ,
        };

        output.fg(TermColor::Cyan);
        output.bg(TermColor::Default);
        output.push("┃");

        match view.right.get(i) {
            Some(LibraryTrackEntry::Album(album)) => render_album_row(
                output, 
                track_width, 
                album,
            ),
            Some(LibraryTrackEntry::Track(track)) => render_track_row(
                output, 
                track_width, 
                track, 
                view.column_selected == LibraryColumn::Tracks, 
                i == view.right_selected,
            ),
            None => for _ in 0..track_width {output.push(" ")},
        }

        if i+1 != size.height {
            output.push("\r\n");
        }
    }
}

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
fn render_filter_row(
    output: &mut TermState,
    width: usize,
    entry: &LibraryFilterEntry,
    is_active: bool,
    is_selected: bool,
) {
    let name = term_text(entry.name().to_string(), width - 2);

    let (fg, bg) = match (is_active, is_selected) {
        (false, false) |
        (true , false) => (TermColor::Default, TermColor::Default),
        (false, true ) => (TermColor::Black  , TermColor::Red    ),
        (true , true ) => (TermColor::Black  , TermColor::Cyan   ),
    };
    output.fg(fg);
    output.bg(bg);
    
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
    entry: &LibraryAlbumEntryData,
) {
    let album_name = &entry.name;
    let year = match entry.year {
        Some(year) => &year.to_string(),
        None       => "----",
    };

    let static_size = album_name.len() + year.len() + 4;
    let mut dyn_len = width as isize - static_size as isize;

    output.fg(TermColor::Default);
    output.bg(TermColor::Default);
    output.bold();

    output.push(" ");
    output.push(album_name);
    output.push(" ");

    output.fg(TermColor::Cyan);
    while dyn_len > 0 {
        output.push("⎯");
        dyn_len -= 1;
    }

    output.fg(TermColor::Default);
    output.push(" ");
    output.push(year);
    output.push(" ");
    output.reset_format();
}

fn render_track_row(
    output: &mut TermState,
    widht: usize,
    entry: &LibraryTrackEntryData,
    is_active: bool,
    is_selected: bool,
) {
    let track      = match entry.track {
        Some(track) => &format!("{:02}", track),
        None => "--",
    };

    let seperate = |time: u64| (time % 60, time / 60);
    let duration_sec = entry.duration.as_secs();
    let duration = match duration_sec {
        ..=3599 => {
            let (seconds, minutes) = seperate(duration_sec);
            format!("{:02}:{:02}", minutes, seconds)
        },
        3600.. => {
            let (seconds, minutes) = seperate(duration_sec);
            let (minutes, hours  ) = seperate(minutes);
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        },
    };

    let static_len  = track.len() + duration.len() + 6;
    let dynamic_len = widht - static_len;

    let artist_len = dynamic_len as isize - entry.track_name.len() as isize - 3;
    let artist_show = entry.track_artist != entry.album_artist && artist_len >= 10;
    let track_name: &str = match artist_show { 
        false => &term_text(entry.track_name.to_string(), dynamic_len),
        true  => &entry.track_name,
    };
    let artist_name = match (artist_show, &entry.track_artist) {
        (_, None) |
        (false, _) => "",
        (true, Some(track_artist)) => & term_text(track_artist.to_string(), artist_len as usize)
    };

    let (fg_d, fg_y, bg) = match (is_active, is_selected) {
        (false, false) |
        (true , false) => (TermColor::Default, TermColor::Yellow, TermColor::Default),
        (false, true ) => (TermColor::Black  , TermColor::Black , TermColor::Red    ),
        (true , true ) => (TermColor::Black  , TermColor::Black , TermColor::Cyan   ),
    };

    output.bg(bg);
    output.fg(fg_y);
    
    output.push("  ");
    output.push(track);
    output.push(" ");
    
    output.fg(fg_d);
    output.push(&track_name);
    output.push(" ");

    if artist_show {
        output.fg(TermColor::BrightBlack);

        output.push(" - ");
        output.push(artist_name);
    }

    output.fg(fg_y);
    output.push(&duration);
    output.push("  ");
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
