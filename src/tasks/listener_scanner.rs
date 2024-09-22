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
