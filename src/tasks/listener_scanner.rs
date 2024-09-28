use color_eyre::Result;
use jwalk::WalkDir;
use rayon::prelude::*;
use std::time::SystemTime;
use crate::tasks::listener_state::StateActions;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use crate::CONFIG;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn start_fs_scanner_listener(tx: MsgChannels) {
    if let Err(err) = scanner_loop(&tx) {
        error!("scan error: {:?}", err);
        tx.tx_tui.send(RenderActions::Exit).unwrap();
    }
}

// from: https://github.com/Serial-ATA/lofty-rs/blob/aa1ec31ea6f2d6f08cc034cea6aad50923fc5f07/lofty/src/file/file_type.rs#L130
const EXTENSIONS: [&'static str; 26] = [
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

fn scanner_loop(tx: &MsgChannels) -> Result<()> {
    let tx_state = &tx.tx_state;

    tx_state.send(StateActions::ScanIsScanning { is_scanning: true })?;
    info!("Starting scan of '{:?}'", CONFIG.media_dirs);
    let time = SystemTime::now();

    CONFIG.media_dirs.par_iter()
        .flat_map(|dir| WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .par_bridge()
        )
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| EXTENSIONS.contains(&path.extension().unwrap_or_default().to_str().unwrap_or_default()))
        .for_each(|path| match TrackFile::new(&path) {
            Ok(track) => tx_state.send(StateActions::ScanAddSong{track}).unwrap(),
            Err(e) => error!("Parse track error: {:?} {:?}", path, e),
        });

    info!("scan of all directories took: {:?}", SystemTime::now().duration_since(time)?);

    tx_state.send(StateActions::ScanIsScanning { is_scanning: false })?;

    info!("scan thread exit");
    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
