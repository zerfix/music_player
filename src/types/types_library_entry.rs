use audiotags::AudioTag;
use audiotags::Tag;
use std::cmp::Ordering;
use std::hash::Hash;
use std::path::PathBuf;
use tui::widgets::ListItem;
use crate::functions::functions_hash::hash;
use crate::traits::trait_listable::Listable;
use crate::ui::utils::ui_text_util::term_text_line;

//-////////////////////////////////////////////////////////////////////////////
//  Raw Entry
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct TrackFile {
    /// for treating element as a album header in library view
    pub is_album_padding: bool,
    pub id_artist   : u64,
    pub id_album    : u64,
    pub id_track    : u64,
    pub path        : PathBuf,
    pub album_artist: Option<String>,
    pub album_title : Option<String>,
    /// only for when it differs from album_artist
    pub track_artist: Option<String>,
    pub disc        : Option<u16>,
    pub track       : Option<u16>,
    pub track_title : Option<String>,
    pub duration    : Option<f64>,
    pub year        : Option<i32>,
}

impl TrackFile {
    pub fn new(path: PathBuf, tag: Box<dyn AudioTag>) -> TrackFile {
        let album_artist = tag.album_artist().filter(|a| !a.is_empty());
        let track_artist = tag.artist()      .filter(|a| !a.is_empty());
        let album_title  = tag.album_title() .filter(|a| !a.is_empty());
        let track_title  = tag.title()       .filter(|a| !a.is_empty());

        let album_artist = album_artist.or(track_artist).map(|a| a.to_string());
        let track_artist = track_artist.filter(|a| Some(*a) != album_artist.as_deref()).map(|a| a.to_string());
        let album_title  = album_title.map(|at| at.to_string());
        let disc         = tag.disc_number();
        let track        = tag.track_number();
        let track_title  = track_title.map(|t| t.to_string());
        let duration     = tag.duration();
        let year         = tag.year();

        let id_artist = {
            let artist = album_artist.clone().unwrap_or_default().to_lowercase();
            hash(artist.as_bytes())
        };

        let id_album = {
            let artist      = album_artist.clone().unwrap_or_default().to_lowercase();
            let album_title = album_title.clone().unwrap_or_default().to_lowercase();
            hash(format!("{}{}", artist, album_title).as_bytes())
        };

        let id_track = hash(path.to_str().unwrap().to_lowercase().as_bytes());

        TrackFile{
            is_album_padding: false,
            path,
            album_artist,
            album_title,
            disc,
            track,
            track_artist,
            track_title,
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

    pub fn to_track_entry(&self) -> LibraryTrackEntry {
        match self.is_album_padding {
            true => LibraryTrackEntry::Album(LibraryAlbumEntryData{
                id  : self.id_album,
                name: self.album_title.clone().unwrap_or("<missing>".to_string()),
                year: self.year,
            }),
            false => LibraryTrackEntry::Track(LibraryTrackEntryData{
                year        : self.year,
                album_artist: self.album_artist.clone(),
                album_name  : self.album_title.clone().unwrap_or("<missing>".to_string()),
                id          : self.id_track,
                disc        : self.disc,
                track       : self.track,
                track_artist: self.track_artist.clone(),
                track_name  : self.track_title.clone().unwrap_or(self.path.to_str().unwrap_or("<missing>").to_string()),
                duration    : self.duration,
                path        : self.path.clone(),
            })
        }
    }
}

impl Listable for TrackFile {
    fn to_list_item<'a>(self, _width: usize) -> ListItem<'a>{
        todo!()
    }

    fn is_selectable(&self) -> bool {
        !self.is_album_padding
    }
}

impl PartialOrd for TrackFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((
            self.year,
            &self.album_title,
            self.disc,
            !self.is_album_padding,
            self.track,
        ).cmp(&(
            other.year,
            &other.album_title,
            other.disc,
            !other.is_album_padding,
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
    Artist{name: Option<String>},
    Year{year: Option<i32>},
}

impl LibraryFilterEntry {
    pub fn name(&self) -> String {
        match self {
            LibraryFilterEntry::All => "ALL".to_string(),
            LibraryFilterEntry::Artist { name } => match name {
                Some(name) => name.clone(),
                None       => "<missing>".to_string(),
            },
            LibraryFilterEntry::Year { year } => match year {
                Some(year) => year.to_string(),
                None       => "----".to_string(),
            },
        }
    }
}

impl Listable for LibraryFilterEntry {
    fn is_selectable(&self) -> bool {
        true
    }

    fn to_list_item<'a>(self, width: usize) -> tui::widgets::ListItem<'a> {
        ListItem::new(term_text_line(" ", self.name(), " ", width))
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
    Track(LibraryTrackEntryData),
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

    fn to_list_item<'a>(self, width: usize) -> ListItem<'a> {
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
    pub album_artist: Option<String>,
    pub album_name: String,
    pub track: Option<u16>,
    pub track_artist: Option<String>,
    pub track_name: String,
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
            &self.album_artist,
            self.year,
            &self.album_name,
            self.disc,
            self.track,
        ).cmp(&(
            &other.album_artist,
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
    pub fn render(&self, width: usize) -> String {
        match self {
            LibraryTrackEntry::Album(album) => {
                let year_str = album.year.map(|y| y.to_string()).unwrap_or("----".to_string());
                let year = format!(" {} ", year_str);

                term_text_line(" ", album.name.clone(), &year, width)
            },
            LibraryTrackEntry::Track(track) => {
                let num = format!("  {:02} ", track.track.unwrap_or(0));
                let name = track.track_name.clone();
                let duration = match track.duration {
                    Some(dur) => {
                        let dur = dur.round() as usize;
                        let seperate = |time: usize| (time % 60, time / 60);
                        match dur {
                            ..=3599 => {
                                let (seconds, minutes) = seperate(dur);
                                format!(" {:02}:{:02}  ", minutes, seconds)
                            },
                            3600.. => {
                                let (seconds, minutes) = seperate(dur);
                                let (minutes, hours  ) = seperate(minutes);
                                format!(" {:02}:{:02}:{:02}  ", hours, minutes, seconds)
                            },
                        }
                    },
                    None => " --:--  ".to_string(),
                };

                term_text_line(&num, name, &duration, width)
            },
        }
    }
}

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
