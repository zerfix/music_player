use crate::enums::enum_navigate::Navigate;
use crate::types::types_library_entry::LibraryFilterEntry;
use crate::types::types_library_entry::LibraryTrackEntry;
use crate::types::types_library_entry::TrackFile;
use crate::ui::models::model_component_list_state::SortedListState;
use enum_iterator::next_cycle;
use enum_iterator::previous_cycle;
use enum_iterator::Sequence;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct StateLibrary {
    pub tracks: Vec<TrackFile>,
    pub selected_tab: LibraryTab,
    pub selected_column: LibraryColumn,
    pub list_filter: SortedListState<LibraryFilterEntry>,
    pub list_tracks: SortedListState<LibraryTrackEntry>,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(Sequence)]
pub enum LibraryTab {
    Artists,
    Year,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum LibraryColumn {
    Filter,
    Tracks,
}

impl<'a> StateLibrary {
    pub fn init() -> StateLibrary {
        let mut filter = SortedListState::new();
        filter.add(LibraryFilterEntry::All);
        StateLibrary{
            tracks : vec![],
            selected_tab: LibraryTab::Artists,
            selected_column: LibraryColumn::Filter,
            list_filter: filter,
            list_tracks: SortedListState::new(),
        }
    }

    // -- Navigate ------------------------------------------------------------

    pub fn navigate(&mut self, navigate: Navigate) {
        if let Navigate::Tab = navigate {
            self.selected_tab = next_cycle(&self.selected_tab).unwrap();
            self.refresh_filter_list();
            return;
        }
        if let Navigate::RevTab = navigate {
            self.selected_tab = previous_cycle(&self.selected_tab).unwrap();
            self.refresh_filter_list();
            return;
        }

        match self.selected_column {
            LibraryColumn::Filter => {
                let pre_filter = self.list_filter.selected_index();
                match navigate {
                    Navigate::Up     => self.list_filter.select_prev(1),
                    Navigate::Down   => self.list_filter.select_next(1),
                    Navigate::Left   => {},
                    Navigate::Right  => self.selected_column = LibraryColumn::Tracks,
                    Navigate::PgUp   => self.list_filter.select_prev(10),
                    Navigate::PgDown => self.list_filter.select_next(10),
                    Navigate::Home   => self.list_filter.select_start(),
                    Navigate::End    => self.list_filter.select_end(),
                    Navigate::Tab    => unreachable!(),
                    Navigate::RevTab => unreachable!(),
                }
                let post_filter = self.list_filter.selected_index();
                if pre_filter != post_filter {self.refresh_tracks_list()}
            },
            LibraryColumn::Tracks => {
                match navigate {
                    Navigate::Up     => self.list_tracks.select_prev(1),
                    Navigate::Down   => self.list_tracks.select_next(1),
                    Navigate::Left   => self.selected_column = LibraryColumn::Filter,
                    Navigate::Right  => {},
                    Navigate::PgUp   => self.list_tracks.select_prev(10),
                    Navigate::PgDown => self.list_tracks.select_next(10),
                    Navigate::Home   => self.list_tracks.select_start(),
                    Navigate::End    => self.list_tracks.select_end(),
                    Navigate::Tab    => unreachable!(),
                    Navigate::RevTab => unreachable!(),
                }
            },
        }
    }

    // -- Get State -----------------------------------------------------------

    pub fn selected_track(&self) -> Option<&LibraryTrackEntry> {
        self.list_tracks.selected_entry()
    }

    pub fn track_state(&'a self) -> (Option<usize>, &'a Vec<LibraryTrackEntry>) {
        self.list_tracks.selectable()
    }

    // -- Mutate Data ---------------------------------------------------------

    pub fn update_scroll(&mut self, render_height: usize) {
        self.list_filter.update_scroll(render_height);
        self.list_tracks.update_scroll(render_height);
    }

    pub fn new_track(&mut self, track: TrackFile) {

        // add to filter list
        let filter = match self.selected_tab {
            LibraryTab::Artists => LibraryFilterEntry::Artist{name: track.artist.clone().unwrap_or_default()},
            LibraryTab::Year => LibraryFilterEntry::Year{year: track.year.unwrap_or_default()},
        };
        self.list_filter.add(filter);

        // add to track list
        let add_to_track_list = match self.list_filter.selected_entry().unwrap() {
            LibraryFilterEntry::All => true,
            LibraryFilterEntry::Artist{name} => track.artist.as_ref().map(|artist| artist == name).unwrap_or(name.is_empty()),
            LibraryFilterEntry::Year{year} => *year == track.year.unwrap_or(0),
        };
        if add_to_track_list {
            self.list_tracks.add(LibraryTrackEntry::album_from_track(&track));
            self.list_tracks.add(LibraryTrackEntry::track_from_track(&track));
        }

        // add to tracks
        if let Err(index) = self.tracks.binary_search(&track) {
            self.tracks.insert(index, track);
        }
    }

    /// full refresh of filter list
    fn refresh_filter_list(&mut self) {
        let filters = vec![LibraryFilterEntry::All]
            .into_iter()
            .chain(self.tracks.iter()
            .map(|track| match self.selected_tab {
                LibraryTab::Artists => LibraryFilterEntry::Artist { name: track.artist.clone().unwrap_or_default() },
                LibraryTab::Year => LibraryFilterEntry::Year { year: track.year.unwrap_or_default() },
            }));
        self.list_filter.replace_all(filters);

        self.refresh_tracks_list();
    }

    /// full refresh of track list
    fn refresh_tracks_list(&mut self) {
        let tracks = self.tracks.iter()
            .filter(|track| match self.list_filter.selected_entry() {
                None => false,
                Some(LibraryFilterEntry::All) => true,
                Some(LibraryFilterEntry::Artist{name}) => track.artist.as_ref().map(|artist| artist == name).unwrap_or(name.is_empty()),
                Some(LibraryFilterEntry::Year{year}) => *year == track.year.unwrap_or_default(),
            })
            .flat_map(|track| [
                LibraryTrackEntry::album_from_track(track),
                LibraryTrackEntry::track_from_track(track),
            ]);
        self.list_tracks.replace_all(tracks);
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
