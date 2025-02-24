use crate::types::types_library_entry::TrackFile;

//-//////////////////////////////////////////////////////////////////

#[derive(Clone, Copy)]
#[derive(Debug)]
/// Inputs with different behavior with context
pub enum InputLocal {
    Up,
    Down,
    Left,
    Right,

    PgUp,
    PgDown,
    Home,
    End,
    Tab,
    RevTab,

    Select,
    SelectAlt,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
/// Inputs that affects any context
pub enum InputGlobal {
    PlayPause,
    Stop,
    Next,
    Previous,
    SkipForward,
    SkipBackward,
}

//-//////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum InputLocalEffect {
    Up(usize),
    Down(usize),
    Left,
    Right,

    Home,
    End,
    NextTab,
    PrevTab,
}

#[derive(Debug)]
pub enum InputGlobalEffect {
    PlayPause,
    ReplaceTracksAndPlay{tracks: Vec<TrackFile>, index: usize},
    AppendTracks(Vec<TrackFile>),
    ClearTracks,
}

pub enum InputEffect {
    Local(InputLocalEffect),
    Global(InputGlobalEffect),
    None,
}
//-//////////////////////////////////////////////////////////////////
