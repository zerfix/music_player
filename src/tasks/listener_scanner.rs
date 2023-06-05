use anyhow::Error;
use crate::tasks::listener_state::StateActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use jwalk::WalkDir;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum ScannerActions {
    ScanDir{dir: PathBuf},
}

pub fn start_fs_scanner_listener(rx: Receiver<ScannerActions>, tx: MsgChannels) {
    if let Err(err) = scanner_loop(rx, &tx) {
        warn!("scan error: {:?}", err)
    }

    info!("exit");
    tx.tx_exit.send(()).unwrap();
}

fn scanner_loop(rx: Receiver<ScannerActions>, tx: &MsgChannels) -> Result<(), Error> {
    while let Ok(msg) = rx.recv() {
        match msg {
            ScannerActions::ScanDir{dir} => {
                tx.tx_state.send(StateActions::IsScanning { is_scanning: true }).unwrap();
                let tracks = WalkDir::new(dir).into_iter()
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_file())
                    .filter_map(|entry| TrackFile::try_from_path(entry.path()));
                for track in tracks {
                    tx.tx_state.send(StateActions::AddSong{track})?;
                }
                tx.tx_state.send(StateActions::IsScanning { is_scanning: false })?;
            },
        }
    }
    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
