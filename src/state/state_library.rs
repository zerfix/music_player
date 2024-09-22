use color_eyre::eyre::eyre;
use enum_iterator::Sequence;
use enum_iterator::next_cycle;
use enum_iterator::previous_cycle;
use rayon::prelude::*;
use std::time::SystemTime;
use crate::enums::enum_input::InputGlobalEffect;
use crate::enums::enum_input::InputLocal;
use crate::enums::enum_input::InputLocalEffect;
use crate::enums::enum_input::InputEffect;
use crate::types::types_library_entry::LibraryFilterEntry;
use crate::types::types_library_entry::TrackFile;
use crate::ui::models::model_component_list_state::SortedListState;

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
#[derive(Sequence)]
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

impl<'a> StateLibrary {
    pub fn init() -> StateLibrary {
        let mut filter = SortedListState::new(true);
        filter.add(LibraryFilterEntry::All);
        StateLibrary{
            tracks : vec![],
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
            InputLocal::Right  => match self.selected_column {
                LibraryColumn::Filter => local(InputLocalEffect::Right),
                LibraryColumn::Tracks => global(InputGlobalEffect::AppendTrack(self.list_tracks.selected_entry().unwrap().clone())),
            },
            InputLocal::PgUp   => local(InputLocalEffect::Up(10)),
            InputLocal::PgDown => local(InputLocalEffect::Down(10)),
            InputLocal::Home   => local(InputLocalEffect::Home),
            InputLocal::End    => local(InputLocalEffect::End),
            InputLocal::Tab    => local(InputLocalEffect::NextTab),
            InputLocal::RevTab => local(InputLocalEffect::PrevTab),
            InputLocal::Space  => match self.selected_column {
                LibraryColumn::Filter => InputEffect::None,
                LibraryColumn::Tracks => {
                    let entry = match self.list_tracks.selected_entry() {
                         None => {return InputEffect::None},
                         Some(entry) => entry,
                    };
                    let tracks = self.list_tracks.entries().iter()
                        .filter(|t| t.album_artist == entry.album_artist)
                        .cloned()
                        .collect::<Vec<TrackFile>>();
                    global(InputGlobalEffect::AppendTracks(tracks))
                },
            },
            InputLocal::Enter => match self.selected_column {
                LibraryColumn::Filter => local(InputLocalEffect::Right),
                LibraryColumn::Tracks => {
                    let entry = match self.list_tracks.selected_entry() {
                         None => {return InputEffect::None},
                         Some(entry) => entry,
                    };
                    let tracks = self.list_tracks.entries().iter()
                        .filter(|t| t.album_artist == entry.album_artist)
                        .cloned()
                        .collect::<Vec<TrackFile>>();
                    let index = self.list_tracks.selected_index();
                    global(InputGlobalEffect::ReplaceTracksAndPlay{tracks, index})
                },
            },
        }
    }

    pub fn handle_input_effect(&mut self, effect: InputLocalEffect) {
        match self.selected_column {
            LibraryColumn::Filter => {
                match effect {
                    InputLocalEffect::Up(steps)   => {
                        self.list_filter.select_prev(steps);
                        self.refresh_tracks_list();
                    },
                    InputLocalEffect::Down(steps) => {
                        self.list_filter.select_next(steps);
                        self.refresh_tracks_list();
                    },
                    InputLocalEffect::Left        => {},
                    InputLocalEffect::Right       => self.selected_column = LibraryColumn::Tracks,
                    InputLocalEffect::Home        => self.list_filter.select_start(),
                    InputLocalEffect::End         => self.list_filter.select_end(),
                    InputLocalEffect::NextTab => {
                        self.selected_tab = next_cycle(&self.selected_tab);
                        self.refresh_filter_list();
                    },
                    InputLocalEffect::PrevTab => {
                        self.selected_tab = previous_cycle(&self.selected_tab);
                        self.refresh_filter_list();
                    },
                };
            },
            LibraryColumn::Tracks => match effect {
                InputLocalEffect::Up(steps)   => self.list_tracks.select_prev(steps),
                InputLocalEffect::Down(steps) => self.list_tracks.select_next(steps),
                InputLocalEffect::Left        => self.selected_column = LibraryColumn::Filter,
                InputLocalEffect::Right       => {},
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
            LibraryTab::Artists => LibraryFilterEntry::Artist{name: track.album_artist.clone()},
            LibraryTab::Year => LibraryFilterEntry::Year{year: track.year},
        };
        self.list_filter.add(filter);

        // add to track list
        let add_to_track_list = match self.list_filter.selected_entry().ok_or(eyre!("Filter is empty")).unwrap() {
            LibraryFilterEntry::All => true,
            LibraryFilterEntry::Artist{name} => *name == track.album_artist,
            LibraryFilterEntry::Year{year} => *year == track.year,
        };
        if add_to_track_list {
            if let None = self.list_tracks.entries().iter().find(|e| e.album_title == track.album_title) {
                let mut track = track.clone();
                track.is_album_padding = true;
                self.list_tracks.add(track);
            }
            self.list_tracks.add(track.clone());
        }

        // add to tracks
        if let Err(index) = self.tracks.binary_search(&track) {
            self.tracks.insert(index, track);
        }
    }

    /// full refresh of filter list
    fn refresh_filter_list(&mut self) {
        let now = SystemTime::now();
        let mut filters = vec![LibraryFilterEntry::All]
            .into_par_iter()
            .chain(self.tracks.par_iter()
            .map(|track| match self.selected_tab {
                LibraryTab::Artists => LibraryFilterEntry::Artist { name: track.album_artist.clone() },
                LibraryTab::Year => LibraryFilterEntry::Year { year: track.year },
            }))
            .collect::<Vec<_>>();
        filters.sort_unstable();
        filters.dedup();
        self.list_filter.replace_all(filters);
        info!("Assemble filter list: {:?}", SystemTime::now().duration_since(now).unwrap_or_default());
        self.refresh_tracks_list();
    }

    /// full refresh of track list
    fn refresh_tracks_list(&mut self) {
        let now = SystemTime::now();
        let mut tracks = self.tracks.iter()
            .filter(|track| match self.list_filter.selected_entry() {
                None => false,
                Some(LibraryFilterEntry::All) => true,
                Some(LibraryFilterEntry::Artist{name}) => *name == track.album_artist,
                Some(LibraryFilterEntry::Year{year}) => *year == track.year,
            })
            .cloned()
            .flat_map(|track| {
                match track.track_number {
                    Some(1) => {
                        let mut header = track.clone();
                        header.is_album_padding = true;
                        vec![
                            header,
                            track,
                        ]
                    },
                    _ => vec![
                        track
                    ]
                }
            })
            .collect::<Vec<TrackFile>>();
        tracks.sort_unstable();
        info!("Assemble track list: {:?}", SystemTime::now().duration_since(now).unwrap_or_default());
        self.list_tracks.replace_all(tracks);
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
