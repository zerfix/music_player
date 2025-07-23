use crate::enums::enum_input::InputEffect;
use crate::enums::enum_input::InputGlobal;
use crate::enums::enum_input::InputGlobalEffect;
use crate::enums::enum_input::InputLocal;
use crate::globals::playback_state::GlobalPlayback;
use crate::globals::playback_state::PlaybackState;
use crate::state::state_app::AppState;
use crate::tasks::listener_playback::PlaybackActions;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use color_eyre::Report;
use color_eyre::Result;
use crossbeam_channel::Receiver;
use std::time::Duration;
use std::time::Instant;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum StateActions {
    InputLocal(InputLocal),
    InputGlobal(InputGlobal),
    PlaybackNextTrack{error: Option<Report>},
    ScanAddSong{track: Box<TrackFile>},
    Update(),
    Render(),
}

pub fn start_state_listener(tx: MsgChannels, rx: Receiver<(Instant, StateActions)>) {
    if let Err(err) = state_loop(rx, tx.clone()) {
        error!("error: {:?}", err);
        tx.exit.send(Err(err)).unwrap();
    }

    info!("exiting");
}

fn state_loop(rx: Receiver<(Instant, StateActions)>, tx: MsgChannels) -> Result<()> {
    let mut state         = AppState::init();
    let mut render_queued = false;
    let mut render_last   = Instant::now();

    loop {
        match rx.recv() {
            Err(err) => return Err(err.into()),
            Ok((render_start, msg)) => {
                let render_received = render_start.elapsed();

                // Handle input
                match msg {
                    StateActions::InputLocal(input) => {
                        state.mutate(|_, library, playlist| {
                            let effect = library.handle_input(input);
                            match effect {
                                InputEffect::Local(effect) => library.handle_input_effect(effect),
                                InputEffect::Global(effect) => match effect {
                                    InputGlobalEffect::AppendTracks(tracks) => tracks.into_iter().for_each(|track| playlist.append(track)),
                                    InputGlobalEffect::ReplaceTracksAndPlay{tracks, index} => {
                                        playlist.replace(tracks, index);
                                        tx.playback.send(PlaybackActions::Clear).unwrap();
                                        if let Some(track) = playlist.get_current_track() {
                                            tx.playback.send(PlaybackActions::Play{track: Box::new(track), start_at: None}).unwrap();
                                        }
                                        if let Some(track) = playlist.get_next_track() {
                                            tx.playback.send(PlaybackActions::Que{track: Box::new(track)}).unwrap();
                                        }
                                    },
                                },
                                InputEffect::None => {},
                            };
                        });
                    },
                    StateActions::InputGlobal(input) => {
                        state.mutate(|_, _, playlist| match input {
                            InputGlobal::PlayPause => {
                                let progress = GlobalPlayback::read();
                                match progress.state {
                                    PlaybackState::Playing => tx.playback.send(PlaybackActions::Pause).unwrap(),
                                    PlaybackState::Paused  => tx.playback.send(PlaybackActions::Resume(progress.elapsed())).unwrap(),
                                    PlaybackState::Loading |
                                    PlaybackState::Stopped => {},
                                }
                            },
                            InputGlobal::Previous => {
                                let progress = GlobalPlayback::read();
                                match progress.elapsed() > Duration::from_secs(5) {
                                    true => tx.playback.send(PlaybackActions::Replay).unwrap(),
                                    false => {
                                        playlist.previous();
                                        tx.playback.send(PlaybackActions::Clear).unwrap();
                                        if let Some(track) = playlist.get_current_track() {
                                            tx.playback.send(PlaybackActions::Play{track: Box::new(track), start_at: None}).unwrap();
                                        }
                                        if let Some(track) = playlist.get_next_track() {
                                            tx.playback.send(PlaybackActions::Que{track: Box::new(track)}).unwrap();
                                        }
                                    },
                                }
                            },
                            InputGlobal::Next => {
                                tx.playback.send(PlaybackActions::Next).unwrap();
                            },
                            InputGlobal::Stop => {
                                playlist.clear();
                                tx.playback.send(PlaybackActions::Clear).unwrap();
                            },
                            InputGlobal::SkipBackward{sec} => {
                                let dur     = Duration::from_secs(sec as u64);
                                let elapsed = GlobalPlayback::read().elapsed();
                                match elapsed < dur {
                                    true => {
                                        tx.playback.send(PlaybackActions::Replay).unwrap();
                                    },
                                    false => {
                                        let new_elapsed = elapsed - dur;
                                        tx.playback.send(PlaybackActions::Resume(new_elapsed)).unwrap();
                                    },
                                }
                            },
                            InputGlobal::SkipForward{sec} => {
                                let dur     = Duration::from_secs(sec as u64);
                                let elapsed = GlobalPlayback::read().elapsed();
                                let new_elapsed = elapsed + dur;
                                tx.playback.send(PlaybackActions::Resume(new_elapsed)).unwrap();
                            },
                        });
                    },
                    StateActions::PlaybackNextTrack{error} => {
                        if let Some(err) = error {error!("{:?}", err)}
                        state.mutate(|_, _, playlist| {
                            playlist.next();
                            if let Some(track) = playlist.get_next_track() {
                                tx.playback.send(PlaybackActions::Que{track: Box::new(track)}).unwrap();
                            }
                        });
                    },
                    StateActions::ScanAddSong{track} => state.mutate(|_, library, _| {
                        library.new_track(*track);
                        info!("{} tracks", library.tracks.len());
                    }),
                    StateActions::Update() => {},
                    StateActions::Render() => render_queued = false,
                };

                // Render change
                match (render_queued, render_last.elapsed().as_millis()) {
                    (false, ..10) => {
                        tx.delay.send(render_last)?;
                        render_queued = true;
                    },
                    (false, 10..) => {
                        let render_changed = render_start.elapsed();
                        if let Some((common, view)) = state.render_state() {
                            tx.tui.send(RenderActions::RenderFrame{
                                render_start,
                                render_received,
                                render_changed,
                                render_copied: render_start.elapsed(),
                                common,
                                view,
                            })?;
                            render_last = Instant::now();
                        }
                    },
                    (_, _) => {},
                }
            },
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
