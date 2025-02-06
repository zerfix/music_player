use crate::types::types_library_entry::{LibraryFilterEntry, TrackFile};

//-//////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct StatePlaylist {
    pub list: Vec<TrackFile>,
    pub selected: usize,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum PlaybackState {
    None,
    Played,
    Playing,
    Qued,
}

impl StatePlaylist {
    pub fn init() -> StatePlaylist {
        StatePlaylist{
            list: vec![],
            selected: 0,
        }
    }

    pub fn get_current_track(&self) -> Option<TrackFile> {
        self.list.get(self.selected).copied()
    }

    pub fn get_next_track(&self) -> Option<TrackFile> {
        self.list.get(self.selected+1).copied()
    }

    pub fn next(&mut self) {
        self.selected += 1;
    }

    pub fn previous(&mut self) {
        match self.selected {
            0 => {},
            _ => self.selected -= 1,
        }
    }

    pub fn replace(&mut self, list: Vec<TrackFile>, selected: usize) {
        self.list = list;
        self.selected = selected;
    }

    pub fn append(&mut self, track: TrackFile) {
        self.list.push(track);
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.selected = 0;
    }

    pub fn get_playback_state_for_filter(&self, entry: LibraryFilterEntry) -> PlaybackState {
        match entry {
            LibraryFilterEntry::All => PlaybackState::None,
            LibraryFilterEntry::Artist(artist) => {
                self.list.iter()
                    .enumerate()
                    .fold(None as Option<PlaybackState>, |acc, (index, track)| {
                        if track.id_artist != artist.artist_id {
                            acc
                        } else {
                            match (&acc, index as isize - self.selected as isize) {
                                (&None                        , ..0) => Some(PlaybackState::Played ),
                                (&None                        ,  0 ) => Some(PlaybackState::Playing),
                                (&None                        , 0..) => Some(PlaybackState::Qued   ),
                                (&Some(PlaybackState::Played ),  0 ) => Some(PlaybackState::Playing),
                                (&Some(PlaybackState::Played ), 0..) => Some(PlaybackState::Qued   ),
                                (&Some(PlaybackState::Qued   ),  0 ) => Some(PlaybackState::Playing),
                                (_, _) => acc,
                            }
                        }
                    })
                    .unwrap_or(PlaybackState::None)
            }
            LibraryFilterEntry::Year{year} => {
                self.list.iter()
                    .enumerate()
                    .fold(None as Option<PlaybackState>, |acc, (index, track)| {
                        if track.year != year {
                            acc
                        } else {
                            match (&acc, index as isize - self.selected as isize) {
                                (&None                        , ..0) => Some(PlaybackState::Played ),
                                (&None                        ,  0 ) => Some(PlaybackState::Playing),
                                (&None                        , 0..) => Some(PlaybackState::Qued   ),
                                (&Some(PlaybackState::Played ),  0 ) => Some(PlaybackState::Playing),
                                (&Some(PlaybackState::Played ), 0..) => Some(PlaybackState::Qued   ),
                                (&Some(PlaybackState::Qued   ),  0 ) => Some(PlaybackState::Playing),
                                (_, _) => acc,
                            }
                        }
                    })
                    .unwrap_or(PlaybackState::None)
            }
        }
    }

    pub fn get_playback_state_for_track(&self, track_id: u64) -> PlaybackState {
        self.list.iter()
            .enumerate()
            .fold(None as Option<PlaybackState>, |acc, (index, track)| {
                match (acc, track.id_track == track_id) {
                    (Some(s), _    ) => Some(s),
                    (None   , false) => None,
                    (None   , true ) => match index as isize - self.selected as isize {
                        ..0 => Some(PlaybackState::Played),
                         0  => Some(PlaybackState::Playing),
                        0.. => Some(PlaybackState::Qued),
                    },
                }
            })
            .unwrap_or(PlaybackState::None)
    }
}
//-//////////////////////////////////////////////////////////////////
