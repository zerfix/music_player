use anyhow::Error;
use jwalk::WalkDir;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::SystemTime;
use crate::tasks::listener_state::StateActions;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum ScannerActions {
    ScanDir{dir: PathBuf},
}

pub fn start_fs_scanner_listener(rx: Receiver<ScannerActions>, tx: MsgChannels) {
    if let Err(err) = scanner_loop(rx, &tx) {
        error!("scan error: {:?}", err)
    }

    info!("exit");
    tx.tx_tui.send(RenderActions::Exit).unwrap();
}

fn scanner_loop(rx: Receiver<ScannerActions>, tx: &MsgChannels) -> Result<(), Error> {
    while let Ok(msg) = rx.recv() {
        match msg {
            ScannerActions::ScanDir{dir} => {
                info!("starting scan of '{:?}'", dir.clone());
                tx.tx_state.send(StateActions::ScanIsScanning { is_scanning: true })?;
                let time = SystemTime::now();

                WalkDir::new(dir.clone())
                    .into_iter()
                    .par_bridge()
                    .filter_map(Result::ok)
                    .filter(|entry| entry.path().is_file())
                    .filter_map(|entry| TrackFile::try_from_path(entry.path()))
                    .map(|track| tx.tx_state.send(StateActions::ScanAddSong{track}))
                    .collect::<Result<Vec<_>, _>>()?;

                info!("scan of '{:?}' took: {:?}", dir, SystemTime::now().duration_since(time)?);
                tx.tx_state.send(StateActions::ScanIsScanning { is_scanning: false })?;
            },
        }
    }
    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
