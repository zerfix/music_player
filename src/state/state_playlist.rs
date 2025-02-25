use crate::types::types_library_entry::LibraryFilterEntry;
use crate::types::types_library_entry::TrackFile;

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
    Queued,
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
            LibraryFilterEntry::Artist(artist) => self.list.iter()
                .enumerate()
                .filter(|(_, track)| track.id_artist == artist.artist_id)
                .map(|(index, _)| match index as isize - self.selected as isize {
                    ..0 => PlaybackState::Played,
                     0  => PlaybackState::Playing,
                    1.. => PlaybackState::Queued,
                })
                .fold(PlaybackState::None, |acc, state| match (acc, state) {
                    (PlaybackState::Playing, _) | (_, PlaybackState::Playing) => PlaybackState::Playing,
                    (PlaybackState::Queued , _) | (_, PlaybackState::Queued ) => PlaybackState::Queued,
                    (PlaybackState::Played , _) | (_, PlaybackState::Played ) => PlaybackState::Played,
                    (_, _) => PlaybackState::None,
                }),
            LibraryFilterEntry::Year{year} => self.list.iter()
                .enumerate()
                .filter(|(_, track)| track.year == year)
                .map(|(index, _)| match index as isize - self.selected as isize {
                    ..0 => PlaybackState::Played,
                     0  => PlaybackState::Playing,
                    1.. => PlaybackState::Queued,
                })
                .fold(PlaybackState::None, |acc, state| match (acc, state) {
                    (PlaybackState::Playing, _) | (_, PlaybackState::Playing) => PlaybackState::Playing,
                    (PlaybackState::Queued , _) | (_, PlaybackState::Queued ) => PlaybackState::Queued,
                    (PlaybackState::Played , _) | (_, PlaybackState::Played ) => PlaybackState::Played,
                    (_, _) => PlaybackState::None,
                }),
        }
    }

    pub fn get_playback_state_for_track(&self, track_id: u64) -> PlaybackState {
        match self.list.iter().enumerate().find(|(_, track)| track.id_track == track_id) {
            Some((index, _)) => match index as isize - self.selected as isize {
                ..0 => PlaybackState::Played,
                 0  => PlaybackState::Playing,
                1.. => PlaybackState::Queued,
            },
            None => PlaybackState::None,
        }
    }
}
//-//////////////////////////////////////////////////////////////////
