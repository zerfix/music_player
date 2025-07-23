use crate::globals::terminal_state::GlobalUiState;
use crate::tasks::listener_playback::PlaybackActions;
use crate::tasks::listener_state::StateActions;
use crate::tasks::listener_updater::UpdateActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use crate::CONFIG;
use color_eyre::eyre::OptionExt;
use color_eyre::Result;
use rayon::Scope;
use rayon::ThreadPoolBuilder;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use std::time::SystemTime;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
// from: https://github.com/Serial-ATA/lofty-rs/blob/aa1ec31ea6f2d6f08cc034cea6aad50923fc5f07/lofty/src/file/file_type.rs#L130
const EXTENSIONS: [&str; 26] = [
    "aac",
    "ape",
    "aiff",
    "aif",
    "afc",
    "aifc",
    "mp3",
    "mp2",
    "mp1",
    "wav",
    "wave",
    "wv",
    "opus",
    "flac",
    "ogg",
    "mp4",
    "m4a",
    "m4b",
    "m4p",
    "m4r",
    "m4v",
    "3gp",
    "mpc",
    "mp+",
    "mpp",
    "spx",
];
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn start_fs_scanner_listener(tx: MsgChannels) {
    if let Err(err) = scanner_loop(&tx) {
        error!("scan error: {:?}", err);
        tx.exit.send(Err(err)).unwrap();
    }
}

fn scan_directory(scope: &Scope, dir: PathBuf, tx: MsgChannels) {
    if let Ok(dir) = fs::read_dir(dir) {
        for entry in dir.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_dir() {
                if path.file_name().unwrap().to_string_lossy().starts_with(".") {
                    continue;
                }
                let tx = tx.clone();
                scope.spawn(move |scope| scan_directory(scope, path, tx));
                continue;
            }
            if path.is_file() {
                let extension = path.extension().unwrap_or_default().to_str().unwrap_or_default();
                if EXTENSIONS.contains(&extension) {
                    let tx_state = tx.state.clone();
                    let tx_playback = tx.playback.clone();
                    scope.spawn(move |_| match TrackFile::new(&path) {
                        Ok(track) => {
                            if track.album_title.map(|s| s.starts_with("Beautiful")).unwrap_or(false) {
                                info!("{:?}", &track);
                            }
                            tx_state
                                .send((Instant::now(), StateActions::ScanAddSong { track: Box::new(track) }))
                                .unwrap();
                            tx_playback
                                .send(PlaybackActions::NewTrack {
                                    track_id: track.id_track,
                                    path: path.into_boxed_path(),
                                })
                                .unwrap();
                        },
                        Err(e) => error!("Parse track error: {:?} {:?}", path, e),
                    });
                }
                continue;
            }
        }
    }
}

fn scanner_loop(tx: &MsgChannels) -> Result<()> {
    let dirs = &CONFIG.get().ok_or_eyre("Config not initialized")?.media_dirs;

    GlobalUiState::update_scanning_state(true);
    tx.update.send(UpdateActions::LoadingLibrary(true)).unwrap();
    info!("Starting scan of '{:?}'", dirs);

    let time = SystemTime::now();

    ThreadPoolBuilder::new().build().unwrap().scope(|scope: &Scope| {
        for dir in dirs.iter() {
            let tx = tx.clone();
            scope.spawn(|scope| scan_directory(scope, dir.clone(), tx));
        }
    });

    info!("scan of all directories took: {:?}", SystemTime::now().duration_since(time)?);
    GlobalUiState::update_scanning_state(false);
    tx.update.send(UpdateActions::LoadingLibrary(false)).unwrap();

    info!("scan thread exit");
    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
