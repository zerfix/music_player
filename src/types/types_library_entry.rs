use color_eyre::Result;
use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;
use std::cmp::Ordering;
use std::hash::Hash;
use std::path::Path;
use std::time::Duration;
use lofty::read_from_path;
use lofty::prelude::ItemKey;
use crate::functions::functions_hash::hash;
use crate::traits::trait_listable::Listable;


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
    pub path        : Box<Path>,

    pub duration     : Duration,
    pub year         : Option<u16>,
    pub album_artist : Option<Box<str>>,
    pub album_title  : Option<Box<str>>,
    pub album_number : Option<u8>,
    pub track_artist : Option<Box<str>>,
    pub track_title  : Option<Box<str>>,
    pub track_number : Option<u8>,
}

impl TrackFile {
    pub fn new(path: &Path) -> Result<TrackFile> {
        let file = read_from_path(path)?;

        let properties = file.properties();
        let primary = file.primary_tag().unwrap();

        let duration = properties.duration();
        let year     = primary.year().map(|n| n as u16);

        let track_artist = primary.artist().map(|s| s.into_owned().into_boxed_str()).filter(|s| !s.is_empty());
        let track_title  = primary.title().map(|s| s.into_owned().into_boxed_str()).filter(|s| !s.is_empty());
        let track_number = primary.track().map(|n| n as u8);

        let album_artist = primary.get_string(&ItemKey::AlbumArtist).map(|s| s.to_owned().into_boxed_str()).filter(|s| !s.is_empty());
        let album_artist = album_artist.or(track_artist.clone());
        let album_title  = primary.album().map(|s| s.into_owned().into_boxed_str()).filter(|s| !s.is_empty());
        let album_number = primary.disk().map(|n| n as u8);

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

        let path = path.to_owned().into_boxed_path();

        Ok(TrackFile{
            is_album_padding: false,
            id_artist,
            id_album,
            id_track,
            path,

            duration,
            year,
            album_artist,
            album_title,
            album_number,
            track_artist,
            track_title,
            track_number,
        })
    }

    pub fn to_track_entry(&self) -> LibraryTrackEntry {
        match self.is_album_padding {
            true => LibraryTrackEntry::Album(LibraryAlbumEntryData{
                id  : self.id_album,
                name: self.album_title.clone().unwrap_or("<missing>".to_owned().into_boxed_str()),
                year: self.year,
            }),
            false => LibraryTrackEntry::Track(LibraryTrackEntryData{
                year        : self.year,
                album_artist: self.album_artist.clone(),
                album_name  : self.album_title.clone().unwrap_or("<missing>".to_owned().into_boxed_str()),
                id          : self.id_track,
                disc        : self.album_number,
                track       : self.track_number,
                track_artist: self.track_artist.clone(),
                track_name  : self.track_title.clone().unwrap_or(self.path.to_str().unwrap().to_owned().into_boxed_str()),
                duration    : self.duration,
                path        : self.path.clone(),
            })
        }
    }
}

impl Listable for TrackFile {
    fn is_selectable(&self) -> bool {
        !self.is_album_padding
    }
}

impl PartialOrd for TrackFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((
            self.year,
            &self.album_title,
            self.album_number,
            !self.is_album_padding,
            self.track_number,
        ).cmp(&(
            other.year,
            &other.album_title,
            other.album_number,
            !other.is_album_padding,
            other.track_number,
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
    Artist{name: Option<Box<str>>},
    Year{year: Option<u16>},
}

impl LibraryFilterEntry {
    pub fn name(&self) -> String {
        match self {
            LibraryFilterEntry::All => "ALL".to_string(),
            LibraryFilterEntry::Artist { name } => match name {
                Some(name) => name.to_string(),
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

    pub fn year(&self) -> Option<u16> {
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
}

#[derive(Clone)]
#[derive(Debug)]
pub struct LibraryAlbumEntryData {
    pub id: u64,
    pub name: Box<str>,
    pub year: Option<u16>,
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
    pub disc: Option<u8>,
    pub album_artist: Option<Box<str>>,
    pub album_name: Box<str>,
    pub track: Option<u8>,
    pub track_artist: Option<Box<str>>,
    pub track_name: Box<str>,
    pub year: Option<u16>,
    pub duration: Duration,
    pub path: Box<Path>,
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

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
