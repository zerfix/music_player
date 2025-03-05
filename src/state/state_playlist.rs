use crate::types::types_library_entry::LibraryFilterEntry;
use crate::types::types_library_entry::TrackFile;
use std::time::Duration;
use std::time::Instant;

//-//////////////////////////////////////////////////////////////////
//
//-//////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct StatePlaylist {
    pub list: Vec<TrackFile>,
    pub selected: usize,
    pub playing_since: Option<Instant>,
    pub played_acc: Duration,
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
            playing_since: None,
            played_acc: Duration::default(),
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
        self.playing_since = Some(Instant::now());
        self.played_acc = Duration::default();
    }

    pub fn previous(&mut self) {
        match self.selected {
            0 => {},
            _ => self.selected -= 1,
        }
        self.playing_since = Some(Instant::now());
        self.played_acc = Duration::default();
    }

    pub fn replay(&mut self) {
        self.playing_since = Some(Instant::now());
        self.played_acc = Duration::default();
    }

    pub fn pause(&mut self) {
        if let Some(playing_since) = self.playing_since {
            self.played_acc += playing_since.elapsed();
            self.playing_since = None;
        }
    }

    pub fn resume(&mut self) {
        if self.playing_since.is_none() {
            self.playing_since = Some(Instant::now());
        }
    }

    pub fn elapsed(&self) -> Duration {
        let add = match self.playing_since {
            None                => Duration::default(),
            Some(playing_since) => playing_since.elapsed(),
        };
        self.played_acc + add
    }

    pub fn playback_progress(&self) -> Option<(bool, Duration, f64, Duration)> {
        let track    = self.list.get(self.selected)?;
        let playing  = self.playing_since.is_some();
        let elapsed  = self.elapsed();
        let progress = elapsed.as_secs_f64() / track.duration.as_secs_f64();
        let total    = track.duration;

        Some((playing, elapsed, progress, total))
    }

    pub fn replace(&mut self, list: Vec<TrackFile>, selected: usize) {
        self.list = list;
        self.selected = selected;
        self.playing_since = None;
        self.played_acc = Duration::default();
    }

    pub fn append(&mut self, track: TrackFile) {
        self.list.push(track);
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.selected = 0;
        self.playing_since = None;
        self.played_acc = Duration::default();
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
//
//-//////////////////////////////////////////////////////////////////
