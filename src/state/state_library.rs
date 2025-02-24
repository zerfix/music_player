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
use rayon::prelude::*;
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
    pub selected_tab: LibraryTab,
    pub selected_column: LibraryColumn,
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

impl StateLibrary {
    pub fn init() -> StateLibrary {
        let mut filter = SortedListState::new(true);
        filter.add(LibraryFilterEntry::All);
        StateLibrary{
            tracks: vec![],
            selected_tab: LibraryTab::Artists,
            selected_column: LibraryColumn::Filter,
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
                    let tracks = self.list_tracks.entries().iter()
                        .filter(|t| t.is_selectable() && t.id_artist == entry.id_artist)
                        .cloned()
                        .collect::<Vec<TrackFile>>();
                    let album_offset = self.list_tracks.entries().iter()
                        .take(self.list_tracks.selected_index())
                        .filter(|t| !t.is_selectable())
                        .count();
                    let index = self.list_tracks.selected_index() - album_offset;
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
                    let tracks = self.list_tracks.entries().iter()
                        .filter(|t| t.is_selectable() && t.id_artist == entry.id_artist)
                        .cloned()
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
                InputLocalEffect::Right       => {return},
                InputLocalEffect::Home        => self.list_tracks.select_start(),
                InputLocalEffect::End         => self.list_tracks.select_end(),
                InputLocalEffect::NextTab     => {},
                InputLocalEffect::PrevTab     => {},
            }
        }
    }

    // -- Mutate Data ---------------------------------------------------------

    pub fn new_track(&mut self, track: TrackFile) {
        // add to filter list
        let filter = match self.selected_tab {
            LibraryTab::Artists => LibraryFilterEntry::Artist(LibraryArtistEntry::from_track(track)),
            LibraryTab::Year => LibraryFilterEntry::Year{year: track.year},
        };
        self.list_filter.add(filter);

        // add to track list
        let add_to_track_list = match self.list_filter.selected_entry().ok_or(eyre!("Filter is empty")).unwrap() {
            LibraryFilterEntry::All => true,
            LibraryFilterEntry::Artist(artist) => artist.artist_id == track.id_artist,
            LibraryFilterEntry::Year{year} => *year == track.year,
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
        let mut filters = vec![LibraryFilterEntry::All].into_par_iter()
            .chain(self.tracks.par_iter()
            .map(|track| match self.selected_tab {
                LibraryTab::Artists => LibraryFilterEntry::Artist(LibraryArtistEntry::from_track(*track)),
                LibraryTab::Year => LibraryFilterEntry::Year { year: track.year },
            }))
            .collect::<Vec<_>>();
        filters.sort_unstable();
        filters.dedup();
        self.list_filter.replace_all(filters);
        self.refresh_tracks_list();
    }

    /// full refresh of track list
    fn refresh_tracks_list(&mut self) {
        let mut tracks = self.tracks.iter()
            .filter(|track| match self.list_filter.selected_entry() {
                None => false,
                Some(LibraryFilterEntry::All) => true,
                Some(LibraryFilterEntry::Artist(album)) => album.artist_id == track.id_artist,
                Some(LibraryFilterEntry::Year{year}) => *year == track.year,
            })
            .cloned()
            .flat_map(|track| match track.track_number {
                Some(1) => {
                    let mut header = track;
                    header.is_album_padding = true;
                    vec![header, track]
                },
                _ => vec![track],
            })
            .collect::<Vec<TrackFile>>();
        tracks.sort_unstable();
        self.list_tracks.replace_all(tracks);
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
