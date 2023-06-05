use anyhow::Error;
use crate::enums::enum_navigate::Navigate;
use crate::state::state_interface::StateInterface;
use crate::state::state_app::AppState;
use crate::tasks::listener_playback::PlaybackActions;
use crate::types::types_library_entry::LibraryTrackEntry;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use std::sync::mpsc::Receiver;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum StateActions {
    Navigate(Navigate),
    IsScanning {
        is_scanning: bool,
    }, // used to show user if scanning on startup
    AddSong {
        track: TrackFile,
    },
    PlayFrom,
    GetState {
        iteration: usize,
        body_height: u16,
    },
    Exit,
}

pub fn start_state_listener(rx: Receiver<StateActions>, tx: MsgChannels, state: AppState) {
    if let Err(err) = state_loop(rx, &tx, state) {
        warn!("error: {:?}", err)
    }

    info!("exit");
    tx.tx_exit.send(()).unwrap();
}

fn state_loop(rx: Receiver<StateActions>, tx: &MsgChannels, mut state: AppState) -> Result<(), Error> {
    let tx_playback = &tx.tx_playback;

    while let Ok(msg) = rx.recv() {
        match msg {
            StateActions::Navigate(nav) => state.mut_library(|lib| lib.navigate(nav)),
            StateActions::PlayFrom => {
                let (index, tracks) = state.library.track_state();
                let playlist = tracks.iter()
                    .skip(index.unwrap_or(0))
                    .filter_map(|t| match t {
                        LibraryTrackEntry::Album(_) => None,
                        LibraryTrackEntry::Track(t) => Some(t),
                    });
                for track in playlist {
                    tx_playback.send(PlaybackActions::AppendTrack { track: track.clone() }).unwrap();
                }
            },
            StateActions::IsScanning { is_scanning } => {
                state.mut_interface(|interface: &mut StateInterface| {
                    interface.is_scanning = is_scanning;
                })
            }
            StateActions::AddSong { track } => state.mut_library(|lib| {
                lib.new_track(track);
                info!("{}", lib.tracks.len());
            }),
            StateActions::GetState {
                iteration,
                body_height,
            } => {
                state.library.update_scroll(body_height as usize);
                match iteration != state.iteration {
                    true => tx.tx_tui.send(Some(state.clone())),
                    false => tx.tx_tui.send(None),
                }
                .unwrap();
            }
            StateActions::Exit => break,
        }
    };
    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
