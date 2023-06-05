use audiotags::AudioTag;
use audiotags::Tag;
use crate::functions::functions_hash::hash;
use crate::traits::trait_listable::Listable;
use std::cmp::Ordering;
use std::hash::Hash;
use std::path::PathBuf;
use tui::widgets::ListItem;

//-////////////////////////////////////////////////////////////////////////////
//  Raw Entry
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct TrackFile {
    pub id_artist  : u64,
    pub id_album   : u64,
    pub id_track   : u64,
    pub path       : PathBuf,
    pub artist     : Option<String>,
    pub album_title: Option<String>,
    pub disc       : Option<u16>,
    pub track      : Option<u16>,
    pub title      : Option<String>,
    pub duration   : Option<f64>,
    pub year       : Option<i32>,
}

impl TrackFile {
    pub fn new(path: PathBuf, tag: Box<dyn AudioTag>) -> TrackFile {
        let artist      = tag.artist().map(|a| a.to_string());
        let album_title = tag.album_title().map(|at| at.to_string());
        let disc        = tag.disc_number();
        let track       = tag.track_number();
        let title       = tag.title().map(|t| t.to_string());
        let duration    = tag.duration();
        let year        = tag.year();

        let id_artist = {
            let artist = artist.clone().unwrap_or_default().to_lowercase();
            hash(artist.as_bytes())
        };

        let id_album = {
            let artist      = artist.clone().unwrap_or_default().to_lowercase();
            let album_title = album_title.clone().unwrap_or_default().to_lowercase();
            hash(format!("{}{}", artist, album_title).as_bytes())
        };

        let id_track = hash(path.to_str().unwrap().to_lowercase().as_bytes());

        TrackFile{
            path,
            artist,
            album_title,
            disc,
            track,
            title,
            duration,
            year,
            id_artist,
            id_album,
            id_track,
        }
    }

    pub fn try_from_path(path: PathBuf) -> Option<TrackFile> {
        match Tag::new().read_from_path(&path) {
            Ok(tag) => Some(TrackFile::new(path, tag)),
            Err(_) => None,
        }
    }
}


impl PartialOrd for TrackFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((
            self.year,
            &self.album_title,
            self.disc,
            self.track,
        ).cmp(&(
            other.year,
            &other.album_title,
            other.disc,
            other.track,
        )))
    }
}

impl Ord for TrackFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for TrackFile {
    fn eq(&self, other: &Self) -> bool {
        self.id_track == other.id_track
    }
}

impl Eq for TrackFile { }

impl Hash for TrackFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}
//-////////////////////////////////////////////////////////////////////////////
//  Filter Entry
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryFilterEntry {
    All,
    Artist{name: String},
    Year{year: i32},
}

impl LibraryFilterEntry {
    pub fn name(&self) -> String {
        match self {
            LibraryFilterEntry::All => "all".to_string(),
            LibraryFilterEntry::Artist { name } => name.clone(),
            LibraryFilterEntry::Year { year } => year.to_string(),
        }
    }
}

impl Listable for LibraryFilterEntry {
    fn is_selectable(&self) -> bool {
        true
    }

    fn to_list_item(&self, _width: usize) -> tui::widgets::ListItem {
        ListItem::new(self.name())
    }
}

//-////////////////////////////////////////////////////////////////////////////
//  Track Entry
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum LibraryTrackEntry {
    Album(LibraryAlbumEntryData),
    Track(LibraryTrackEntryData)
}

impl LibraryTrackEntry {
    pub fn album_name(&self) -> &str {
        match self {
            LibraryTrackEntry::Album(album) => &album.name,
            LibraryTrackEntry::Track(track) => &track.album_name,
        }
    }

    pub fn year(&self) -> Option<i32> {
        match self {
            LibraryTrackEntry::Album(album) => album.year,
            LibraryTrackEntry::Track(track) => track.year,
        }
    }

    pub fn is_album(&self) -> bool {
        match self {
            LibraryTrackEntry::Album(_) => true,
            LibraryTrackEntry::Track(_) => false,
        }
    }
}

impl PartialOrd for LibraryTrackEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if let (LibraryTrackEntry::Track(this_track), LibraryTrackEntry::Track(other_track)) = (self, other) {
            let cmp = (
                self.album_name(),
                self.year(),
                !self.is_album(),
            ).cmp(&(
                other.album_name(),
                other.year(),
                !other.is_album(),
            ));
            if let Ordering::Equal = cmp {
                return Some(this_track.cmp(other_track))
            }
        }

        Some((
            self.album_name(),
            self.year(),
            !self.is_album(),
        ).cmp(&(
            other.album_name(),
            other.year(),
            !other.is_album(),
        )))
    }
}

impl Ord for LibraryTrackEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Listable for LibraryTrackEntry {
    fn is_selectable(&self) -> bool {
        match self {
            LibraryTrackEntry::Album(_) => false,
            LibraryTrackEntry::Track(_) => true,
        }
    }

    fn to_list_item(&self, width: usize) -> ListItem {
        ListItem::new(self.render(width))
    }
}

#[derive(Clone)]
#[derive(Debug)]
pub struct LibraryAlbumEntryData {
    pub id: u64,
    pub name: String,
    pub year: Option<i32>,
}

impl PartialOrd for LibraryAlbumEntryData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.year, &self.name).cmp(&(other.year, &other.name)))
    }
}

impl Ord for LibraryAlbumEntryData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for LibraryAlbumEntryData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for LibraryAlbumEntryData {}

#[derive(Clone)]
#[derive(Debug)]
pub struct LibraryTrackEntryData {
    pub id: u64,
    pub disc: Option<u16>,
    pub album_name: String,
    pub track: Option<u16>,
    pub name: Option<String>,
    pub year: Option<i32>,
    pub duration: Option<f64>,
    pub path: PathBuf,
}

impl Ord for LibraryTrackEntryData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for LibraryTrackEntryData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((
            self.year,
            &self.album_name,
            self.disc,
            self.track,
        ).cmp(&(
            other.year,
            &other.album_name,
            other.disc,
            other.track,
        )))
    }
}

impl PartialEq for LibraryTrackEntryData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for LibraryTrackEntryData { }

impl LibraryTrackEntry {
    pub fn album_from_track(track: &TrackFile) -> LibraryTrackEntry {
        LibraryTrackEntry::Album(LibraryAlbumEntryData{
            id  : track.id_album,
            name: track.album_title.clone().unwrap_or("".to_string()),
            year: track.year,
        })
    }

    pub fn track_from_track(track: &TrackFile) -> LibraryTrackEntry {
        LibraryTrackEntry::Track(LibraryTrackEntryData{
            year      : track.year,
            album_name: track.album_title.clone().unwrap_or_default(),
            id        : track.id_track,
            disc      : track.disc,
            track     : track.track,
            name      : track.title.clone(),
            duration  : track.duration,
            path      : track.path.clone(),
        })
    }

    pub fn render(&self, width: usize) -> String {
        match self {
            LibraryTrackEntry::Album(album) => {
                let name = album.name.clone();
                let year = match album.year {
                    Some(year) => year.to_string(),
                    None => "----".to_string(),
                };
                let padding: String =  vec![' '; width.saturating_sub(1 + name.chars().count() + year.chars().count())].into_iter().collect();
                format!(" {}{}{}", name, padding, year)
            },
            LibraryTrackEntry::Track(track) => {
                let name = match &track.name {
                    Some(name) => name.clone(),
                    None => "<no name>".to_string(),
                };
                let duration = match track.duration {
                    Some(dur) => {
                        let dur = dur.round() as usize;
                        let seconds = dur % 60;
                        let minutes = dur / 60;
                        format!("{:02}:{:02}", minutes, seconds)
                    },
                    None => "--:--".to_string(),
                };
                let track = format!("{:02}", track.track.unwrap_or(0));
                let padding: String =  vec![' '; width.saturating_sub(3 + name.chars().count() + duration.chars().count() + track.chars().count())].into_iter().collect();
                format!("  {} {}{}{}", track, name, padding, duration)
            },
        }
    }
}


//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
