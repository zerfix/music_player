use color_eyre::Result;
use color_eyre::Report;
use crossbeam_channel::Receiver;
use std::time::SystemTime;
use crate::enums::enum_input::InputGlobalEffect;
use crate::enums::enum_input::InputEffect;
use crate::enums::enum_input::InputGlobal;
use crate::enums::enum_input::InputLocal;
use crate::state::state_app::AppState;
use crate::tasks::listener_playback::PlaybackActions;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use crate::types::types_tui::TermSize;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum StateActions {
    InputLocal(InputLocal),
    InputGlobal(InputGlobal),
    PlaybackNextTrack{error: Option<Report>},
    ScanIsScanning{is_scanning: bool}, // used to show user if scanning on startup
    ScanAddSong{track: TrackFile},
    Render{
        render_target: SystemTime,
        render_start: SystemTime,
        render_request: SystemTime,
        term_size: TermSize},
    Exit,
}

pub fn start_state_listener(rx: Receiver<StateActions>, tx: MsgChannels) {
    if let Err(err) = state_loop(rx, tx.clone()) {
        error!("error: {:?}", err)
    }

    info!("exiting");
    tx.tx_exit.send(()).unwrap();
}

fn state_loop(rx: Receiver<StateActions>, tx: MsgChannels) -> Result<()> {
    let mut state = AppState::init();
    let tx_playback = tx.tx_playback;
    let tx_tui      = tx.tx_tui;

    while let Ok(msg) = rx.recv() {
        match msg {
            StateActions::InputLocal(input) => {
                state.mutate(|_, library, playlist| {
                    let effect = library.handle_input(input);
                    match effect {
                        InputEffect::Local(effect) => library.handle_input_effect(effect),
                        InputEffect::Global(effect) => match effect {
                            InputGlobalEffect::PlayPause => {todo!()},
                            InputGlobalEffect::ReplaceTracksAndPlay { tracks, index } => {
                                playlist.replace(tracks, index);
                                tx_playback.send(PlaybackActions::Clear).unwrap();
                                if let Some(track) = playlist.get_current_track() {
                                    tx_playback.send(PlaybackActions::Play {
                                        track,
                                        start_at: None,
                                    }).unwrap();
                                }
                                if let Some(track) = playlist.get_next_track() {
                                    tx_playback.send(PlaybackActions::Que {
                                        track,
                                    }).unwrap();
                                }
                            },
                            InputGlobalEffect::AppendTracks(tracks) => {todo!()},
                            InputGlobalEffect::AppendTrack(track) => {todo!()},
                            InputGlobalEffect::ClearTracks => {todo!()},
                        },
                        InputEffect::None => {},
                    };
                });
            },
            StateActions::InputGlobal(input) => {
                state.mutate(|_, _, playlist| {
                    match input {
                        InputGlobal::PlayPause => {},
                        InputGlobal::Previous => {
                            playlist.previous();
                            tx_playback.send(PlaybackActions::Clear).unwrap();
                            if let Some(track) = playlist.get_current_track() {
                                tx_playback.send(PlaybackActions::Play {
                                    track,
                                    start_at: None,
                                }).unwrap();
                            }
                            if let Some(track) = playlist.get_next_track() {
                                tx_playback.send(PlaybackActions::Que {
                                    track,
                                }).unwrap();
                            }
                        },
                        InputGlobal::Next => {
                            tx_playback.send(PlaybackActions::Next).unwrap();
                        },
                        InputGlobal::Stop => {
                            info!("stopping state");
                            playlist.clear();
                            tx_playback.send(PlaybackActions::Clear).unwrap()
                        },
                        InputGlobal::SkipBackward => {todo!()},
                        InputGlobal::SkipForward => {todo!()},
                    }
                });
            },
            StateActions::PlaybackNextTrack{error} => {
                state.mutate(|_, _, playlist| {
                    playlist.next();
                    if let Some(track) = playlist.get_next_track() {
                        tx_playback.send(PlaybackActions::Que{track}).unwrap();
                    }
                });
            },
            StateActions::ScanIsScanning { is_scanning } => {
                state.mutate(|interface, _, _| {
                    interface.is_scanning = is_scanning;
                })
            },
            StateActions::ScanAddSong { track } => {
                state.mutate(|_, library, _| {
                    library.new_track(track);
                    info!("{} tracks", library.tracks.len());
                })
            },
            StateActions::Render{render_target, render_start, render_request, term_size} => {
                if let Some((common, view)) = state.render_state(false, term_size) {
                    tx_tui.send(RenderActions::RenderFrame{
                        render_target,
                        render_start,
                        render_request,
                        render_state: SystemTime::now(),
                        common,
                        view,
                    }).unwrap();
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
