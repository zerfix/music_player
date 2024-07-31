use crate::types::types_library_entry::LibraryTrackEntry;

//-//////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct StatePlaylist {
    pub list: Vec<LibraryTrackEntry>,
    pub index: usize,
}

impl StatePlaylist {
    pub fn init() -> StatePlaylist {
        StatePlaylist{
            list: vec![],
            index: 0,
        }
    }

    pub fn next(&mut self) -> Option<LibraryTrackEntry> {
        if self.list.len() == self.index {
            return None;
        }
        self.index += 1;
        Some(self.list[self.index].clone())
    }

    pub fn previous(&mut self) -> Option<LibraryTrackEntry> {
        match self.index {
            0 => None,
            _ => {
                self.index -= 1;
                Some(self.list[self.index].clone())
            }
        }
    }

    pub fn append(&mut self, track: LibraryTrackEntry) {
        self.list.push(track);
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.index = 0;
    }
}
//-//////////////////////////////////////////////////////////////////
