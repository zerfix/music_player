use crate::enums::enum_input::InputEffect;
use crate::enums::enum_input::InputGlobalEffect;
use crate::enums::enum_input::InputLocal;
use crate::enums::enum_input::InputLocalEffect;
use crate::traits::trait_listable::Listable;
use crate::types::types_library_entry::LibraryArtistEntry;
use crate::types::types_library_entry::LibraryFilterEntry;
use crate::types::types_library_entry::TrackFile;
use crate::ui::models::model_component_list_state::SortedListState;
use color_eyre::eyre::eyre;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use strum_macros::IntoStaticStr;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct StateLibrary {
    pub tracks: Vec<TrackFile>,
    pub filters_artist: Vec<LibraryFilterEntry>,
    pub filters_years: Vec<LibraryFilterEntry>,
    pub selected_tab: LibraryTab,
    pub selected_column: LibraryColumn,
    pub select_mode: LibrarySelectMode,
    pub list_filter: SortedListState<LibraryFilterEntry>,
    pub list_tracks: SortedListState<TrackFile>,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(EnumIter, IntoStaticStr)]
pub enum LibraryTab {
    Artists,
    Year,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum LibraryColumn {
    Filter,
    Tracks,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(EnumIter, IntoStaticStr)]
pub enum LibrarySelectMode {
    All,
    Artist,
    Album,
    Track,
}

impl StateLibrary {
    pub fn init() -> StateLibrary {
        let mut filter = SortedListState::new(true);
        filter.add(LibraryFilterEntry::All);
        StateLibrary{
            tracks: vec![],
            filters_artist: vec![LibraryFilterEntry::All],
            filters_years: vec![LibraryFilterEntry::All],
            selected_tab: LibraryTab::Artists,
            selected_column: LibraryColumn::Filter,
            select_mode: LibrarySelectMode::Artist,
            list_filter: filter,
            list_tracks: SortedListState::new(false),
        }
    }

    // -- Navigate -----------------------------------------------------------

    pub fn handle_input(&self, input: InputLocal) -> InputEffect {
        let local  = |effect: InputLocalEffect | InputEffect::Local(effect);
        let global = |effect: InputGlobalEffect| InputEffect::Global(effect);
        match input {
            InputLocal::Up     => local(InputLocalEffect::Up(1)),
            InputLocal::Down   => local(InputLocalEffect::Down(1)),
            InputLocal::Left   => local(InputLocalEffect::Left),
            InputLocal::Right  => local(InputLocalEffect::Right),
            InputLocal::PgUp   => local(InputLocalEffect::Up(10)),
            InputLocal::PgDown => local(InputLocalEffect::Down(10)),
            InputLocal::Home   => local(InputLocalEffect::Home),
            InputLocal::End    => local(InputLocalEffect::End),
            InputLocal::Tab    => local(InputLocalEffect::NextTab),
            InputLocal::RevTab => local(InputLocalEffect::PrevTab),
            InputLocal::Select => match self.selected_column {
                LibraryColumn::Filter => local(InputLocalEffect::Right),
                LibraryColumn::Tracks => {
                    let entry = match self.list_tracks.selected_entry() {
                        None => return InputEffect::None,
                        Some(entry) => entry,
                    };
                    let tracks = self.list_tracks.entries().iter().copied()
                        .filter(|track| track.is_selectable())
                        .filter(|track| match self.select_mode {
                            LibrarySelectMode::All    => true,
                            LibrarySelectMode::Artist => track.id_artist == entry.id_artist,
                            LibrarySelectMode::Album  => track.id_album  == entry.id_album,
                            LibrarySelectMode::Track  => track.id_track  == entry.id_track,
                        })
                        .collect::<Vec<TrackFile>>();
                    let index = tracks.iter()
                        .enumerate()
                        .find_map(|(index, track)| match track.id_track == entry.id_track {
                            false => None,
                            true  => Some(index),
                        })
                        .unwrap_or(0);
                    global(InputGlobalEffect::ReplaceTracksAndPlay{tracks, index})
                },
            },
            InputLocal::SelectAlt => match self.selected_column {
                LibraryColumn::Filter => local(InputLocalEffect::Right),
                LibraryColumn::Tracks => {
                    let entry = match self.list_tracks.selected_entry() {
                        None => return InputEffect::None,
                        Some(entry) => entry,
                    };
                    let tracks = self.list_tracks.entries().iter().copied()
                        .filter(|track| track.is_selectable())
                        .filter(|track| match self.select_mode {
                            LibrarySelectMode::All    => true,
                            LibrarySelectMode::Artist => track.id_artist == entry.id_artist,
                            LibrarySelectMode::Album  => track.id_album  == entry.id_album,
                            LibrarySelectMode::Track  => track.id_track  == entry.id_track,
                        })
                        .collect::<Vec<TrackFile>>();
                    global(InputGlobalEffect::AppendTracks(tracks))
                },
            },
        }
    }

    pub fn handle_input_effect(&mut self, effect: InputLocalEffect) {
        match self.selected_column {
            LibraryColumn::Filter => {
                match effect {
                    InputLocalEffect::Up(steps)   => self.list_filter.select_prev(steps),
                    InputLocalEffect::Down(steps) => self.list_filter.select_next(steps),
                    InputLocalEffect::Left        => return,
                    InputLocalEffect::Right       => {self.selected_column = LibraryColumn::Tracks; return},
                    InputLocalEffect::Home        => self.list_filter.select_start(),
                    InputLocalEffect::End         => self.list_filter.select_end(),
                    InputLocalEffect::NextTab     => {
                        self.selected_tab = LibraryTab::iter()
                            .cycle()
                            .skip_while(|x| *x != self.selected_tab)
                            .nth(1)
                            .unwrap();
                        self.refresh_filter_list();
                    },
                    InputLocalEffect::PrevTab => {
                        self.selected_tab = LibraryTab::iter()
                            .rev()
                            .cycle()
                            .skip_while(|x| *x != self.selected_tab)
                            .nth(1)
                            .unwrap();
                        self.refresh_filter_list();
                    },
                };
                self.refresh_tracks_list()
            },
            LibraryColumn::Tracks => match effect {
                InputLocalEffect::Up(steps)   => self.list_tracks.select_prev(steps),
                InputLocalEffect::Down(steps) => self.list_tracks.select_next(steps),
                InputLocalEffect::Left        => {self.selected_column = LibraryColumn::Filter; return},
                InputLocalEffect::Right       => return,
                InputLocalEffect::Home        => self.list_tracks.select_start(),
                InputLocalEffect::End         => self.list_tracks.select_end(),
                InputLocalEffect::NextTab     => {
                        self.select_mode = LibrarySelectMode::iter()
                            .cycle()
                            .skip_while(|x| *x != self.select_mode)
                            .nth(1)
                            .unwrap();
                },
                InputLocalEffect::PrevTab => {
                        self.select_mode = LibrarySelectMode::iter()
                            .rev()
                            .cycle()
                            .skip_while(|x| *x != self.select_mode)
                            .nth(1)
                            .unwrap();
                },
            }
        }
    }

    // -- Mutate Data ---------------------------------------------------------

    pub fn new_track(&mut self, track: TrackFile) {
        // add to filter lists
        let artist = LibraryFilterEntry::Artist(LibraryArtistEntry::from_track(track));
        let year   = LibraryFilterEntry::Year{year: track.year};
        if let Err(index) = self.filters_artist.binary_search(&artist) {
            self.filters_artist.insert(index, artist);
        }
        if let Err(index) = self.filters_years.binary_search(&year) {
            self.filters_years.insert(index, year);
        }
        match self.selected_tab {
            LibraryTab::Artists => self.list_filter.add(artist),
            LibraryTab::Year    => self.list_filter.add(year),
        };

        // add to track list
        let add_to_track_list = match self.list_filter.selected_entry() {
            None |
            Some(LibraryFilterEntry::All           ) => true,
            Some(LibraryFilterEntry::Artist(artist)) => artist.artist_id == track.id_artist,
            Some(LibraryFilterEntry::Year{year}    ) => *year == track.year,
        };
        if add_to_track_list {
            if !self.list_tracks.entries().iter().any(|e| e.album_title == track.album_title) {
                let mut track = track;
                track.is_album_padding = true;
                self.list_tracks.add(track);
            }
            self.list_tracks.add(track);
        }

        // add to tracks
        if let Err(index) = self.tracks.binary_search(&track) {
            self.tracks.insert(index, track);
        }
    }

    /// full refresh of filter list
    fn refresh_filter_list(&mut self) {
        match self.selected_tab {
            LibraryTab::Artists => self.list_filter.replace_all(self.filters_artist.clone()),
            LibraryTab::Year    => self.list_filter.replace_all(self.filters_years.clone()),
        }
    }

    /// full refresh of track list
    fn refresh_tracks_list(&mut self) {
        let mut tracks = Vec::with_capacity(self.tracks.len()+self.tracks.len()/8);
        self.tracks.iter().copied()
            .filter(|track| match self.list_filter.selected_entry() {
                None |
                Some(LibraryFilterEntry::All          ) => true,
                Some(LibraryFilterEntry::Artist(album)) => album.artist_id == track.id_artist,
                Some(LibraryFilterEntry::Year{year}   ) => *year == track.year,
            })
            .for_each(|track: TrackFile| match track.track_number {
                Some(1) => {
                    let mut album = track;
                    album.is_album_padding = true;
                    tracks.push(album);
                    tracks.push(track);
                },
                _ => tracks.push(track),
            });
        self.list_tracks.replace_all(tracks);
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
