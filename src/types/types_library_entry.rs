use crate::functions::functions_hash::hash;
use crate::traits::trait_listable::Listable;
use arrayvec::ArrayString;
use color_eyre::eyre::OptionExt;
use color_eyre::Result;
use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::prelude::ItemKey;
use lofty::read_from_path;
use lofty::tag::Accessor;
use std::cmp::Ordering;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

//-////////////////////////////////////////////////////////////////////////////
//  Raw Entry
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct TrackFile {
    /// for treating element as a album header in library view
    pub is_album_padding: bool,
    pub id_artist : u64,
    pub id_album  : u64,
    pub id_track  : u64, // hash of file path

    pub duration     : Duration,
    pub year         : Option<u16>,
    pub album_artist : Option<ArrayString<64>>,
    pub album_title  : Option<ArrayString<64>>,
    pub album_number : Option<u8>,
    pub track_artist : Option<ArrayString<64>>,
    pub track_title  : ArrayString<128>,
    pub track_number : Option<u8>,
}

fn str_trunc<const L: usize>(text: &str) -> ArrayString<L> {
    let mut arr = ArrayString::<L>::new();
    if text.len() > L {
        arr.push_str(&text[..L-1]);
        arr.push_str(">");
    } else {
        arr.push_str(&text[..L.min(text.len())]);
    }
    arr
}

impl TrackFile {
    pub fn new(path: &Path) -> Result<TrackFile> {
        let file = read_from_path(path)?;

        let properties = file.properties();
        let primary    = file.primary_tag().ok_or_eyre("primary tags not found")?;

        let duration = properties.duration();
        let year     = primary.year().map(|y| y as u16);

        let track_artist = primary.artist().map(|s| str_trunc::<64>(&s));
        let track_title  = primary.title().map(|s| str_trunc::<128>(&s)).ok_or_eyre("missing track name")?;
        let track_number = primary.track().map(|t| t as u8);

        let album_artist = primary.get_string(&ItemKey::AlbumArtist).filter(|s| !s.is_empty()).map(|s| str_trunc(&s)).or(track_artist);
        let album_title  = primary.album().filter(|s| !s.is_empty()).map(|s| str_trunc(&s));
        let album_number = primary.disk().map(|n| n as u8);

        let id_artist = hash(&album_artist.unwrap_or_default().to_lowercase());
        let id_album  = hash(&(id_artist, album_title.unwrap_or_default().to_lowercase()));
        let id_track  = hash(&path.to_string_lossy().to_lowercase());

        Ok(TrackFile{
            is_album_padding: false,
            id_artist,
            id_album,
            id_track,

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

    fn compare_values(&self) -> (Option<u16>, Option<String>, Option<String>, Option<u8>, bool, Option<u8>) {
        (
            self.year,
            self.album_artist.map(|s| s.to_lowercase()),
            self.album_title.map(|s| s.to_lowercase()),
            self.album_number,
            !self.is_album_padding,
            self.track_number,
        )
    }
}

impl Listable for TrackFile {
    fn is_selectable(&self) -> bool {
        !self.is_album_padding
    }
}

impl PartialOrd for TrackFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare_values().cmp(&other.compare_values()))
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

impl Eq for TrackFile {}

//-////////////////////////////////////////////////////////////////////////////
//  Filter Entry
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct LibraryArtistEntry {
    pub artist_id: u64,
    pub name_compare: Option<ArrayString<64>>,
    pub name_display: Option<ArrayString<64>>,
}

impl LibraryArtistEntry {
    pub fn from_track(track: TrackFile) -> LibraryArtistEntry {
        LibraryArtistEntry{
            artist_id: track.id_artist,
            name_display: track.album_artist,
            name_compare: track.album_artist.map(|aa| {
                let lower = aa.to_lowercase();
                let mut arr = ArrayString::new();
                match lower.starts_with("the ") {
                    true => arr.push_str(&lower[4..]),
                    false => arr.push_str(&lower),
                };
                arr
            }),
        }
    }
}

impl PartialEq for LibraryArtistEntry {
    fn eq(&self, other: &Self) -> bool {
        self.artist_id == other.artist_id
    }
}

impl Eq for LibraryArtistEntry {}

impl PartialOrd for LibraryArtistEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name_compare.cmp(&other.name_compare))
    }
}

impl Ord for LibraryArtistEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name_compare.cmp(&other.name_compare)
    }
}

#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryFilterEntry {
    All,
    Artist(LibraryArtistEntry),
    Year{year: Option<u16>},
}

impl LibraryFilterEntry {
    pub fn name(&self) -> ArrayString<64> {
        match self {
            LibraryFilterEntry::All => ArrayString::from_str("ALL"),
            LibraryFilterEntry::Artist(artist) => match artist.name_display {
                Some(name) => Ok(name),
                None       => ArrayString::from_str("<missing>"),
            },
            LibraryFilterEntry::Year { year } => match year {
                Some(year) => ArrayString::from_str(&year.to_string()),
                None       => ArrayString::from_str("----"),
            },
        }.unwrap()
    }
}

impl Listable for LibraryFilterEntry {
    fn is_selectable(&self) -> bool {
        true
    }
}

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
