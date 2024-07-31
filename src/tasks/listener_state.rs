use anyhow::Error;
use std::sync::mpsc::Receiver;
use std::time::SystemTime;
use crate::enums::enum_navigate::Navigate;
use crate::state::state_app::AppState;
use crate::state::state_interface::StateInterface;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum StateActions {
    PlaybackPlayFrom,
    PlaybackResumePause,
    PlaybackStop,
    PlaybackStopAndClear,
    PlaybackNextCallback,
    PlaybackNextTrack,
    PlaybackPreviousTrack,
    InputNavigate(Navigate),
    ScanIsScanning{is_scanning: bool}, // used to show user if scanning on startup
    ScanAddSong{track: TrackFile},
    TuiUpdateTermHeight{render_start: SystemTime, term_height: usize},
    Render{render_start: SystemTime},
    Exit,
}

pub fn start_state_listener(rx: Receiver<StateActions>, tx: MsgChannels) {
    if let Err(err) = state_loop(rx, tx.clone()) {
        error!("error: {:?}", err)
    }

    info!("exiting");
    tx.tx_exit.send(()).unwrap();
}

fn state_loop(rx: Receiver<StateActions>, tx: MsgChannels) -> Result<(), Error> {
    let mut state = AppState::init();
    let tx_playback = tx.tx_playback;
    let tx_tui      = tx.tx_tui;
    let tx_state    = tx.tx_state;

    while let Ok(msg) = rx.recv() {
        match msg {
            StateActions::InputNavigate(nav) => {
                state.mut_library(|lib| lib.navigate(nav));
            },
            StateActions::PlaybackPlayFrom => {
                // let (index, tracks) = state.library.track_state();
                // playlist_index = 0;
                // playlist = tracks.iter()
                //     .skip(index.unwrap_or(0))
                //     .filter_map(|t| match t {
                //         LibraryTrackEntry::Album(_) => None,
                //         LibraryTrackEntry::Track(t) => Some(t),
                //     })
                //     .cloned()
                //     .collect::<Vec<LibraryTrackEntryData>>();
                // manager.clear();
                // let x = playlist.first().and_then(|track|{

                // });
            },
            StateActions::PlaybackResumePause => {},
            StateActions::PlaybackStop => {},
            StateActions::PlaybackStopAndClear => {},
            StateActions::PlaybackNextCallback => {},
            StateActions::PlaybackNextTrack => {},
            StateActions::PlaybackPreviousTrack => {},
            StateActions::ScanIsScanning { is_scanning } => {
                state.mut_interface(|interface: &mut StateInterface| {
                    interface.is_scanning = is_scanning;
                })
            },
            StateActions::ScanAddSong { track } => {
                state.mut_library(|lib| {
                    lib.new_track(track);
                    info!("{} tracks", lib.tracks.len());
                })
            },
            StateActions::TuiUpdateTermHeight{render_start, term_height} => {
                info!("updating term height: {}", term_height);
                state.term_height = term_height;
                tx_state.send(StateActions::Render{render_start}).unwrap();
            },
            StateActions::Render{render_start} => {
                if let Some((common, view)) = state.render_state(false) {
                    tx_tui.send(RenderActions::RenderFrame{render_start, common, view}).unwrap();
                }
            },
            StateActions::Exit => break,
        }
    };

    tx_tui.send(super::listener_tui::RenderActions::Exit).unwrap();

    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
