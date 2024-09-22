use crate::types::types_library_entry::TrackFile;

//-//////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct StatePlaylist {
    pub list: Vec<TrackFile>,
    pub selected: usize,
}

impl StatePlaylist {
    pub fn init() -> StatePlaylist {
        StatePlaylist{
            list: vec![],
            selected: 0,
        }
    }

    pub fn get_current_track(&self) -> Option<&TrackFile> {
        self.list.get(self.selected)
    }

    pub fn get_next_track(&self) -> Option<&TrackFile> {
        self.list.get(self.selected+1)
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
}
//-//////////////////////////////////////////////////////////////////
