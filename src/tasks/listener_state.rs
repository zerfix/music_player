use crate::enums::enum_input::InputEffect;
use crate::enums::enum_input::InputGlobal;
use crate::enums::enum_input::InputGlobalEffect;
use crate::enums::enum_input::InputLocal;
use crate::state::state_app::AppState;
use crate::tasks::listener_playback::PlaybackActions;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use color_eyre::Report;
use color_eyre::Result;
use crossbeam_channel::Receiver;
use std::time::Instant;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum StateActions {
    InputLocal(InputLocal),
    InputGlobal(InputGlobal),
    PlaybackNextTrack{error: Option<Report>},
    PlaybackPlay(),
    PlaybackPause(),
    ScanIsScanning{is_scanning: bool}, // used to show user if scanning on startup
    ScanAddSong{track: Box<TrackFile>},
    ScanIndicatorRotate(u8),
    Resize{height: u16, width: u16},
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
                                match playlist.playing_since.is_some() {
                                    true => {
                                        tx.playback.send(PlaybackActions::Pause).unwrap();
                                        playlist.pause();
                                        tx.update.send(false).unwrap();
                                    },
                                    false => {
                                        tx.playback.send(PlaybackActions::Resume(playlist.played_acc)).unwrap();
                                        playlist.resume();
                                        tx.update.send(true).unwrap();
                                    },
                                }
                            },
                            InputGlobal::Previous => {
                                playlist.previous();
                                tx.playback.send(PlaybackActions::Clear).unwrap();
                                if let Some(track) = playlist.get_current_track() {
                                    tx.playback.send(PlaybackActions::Play{track: Box::new(track), start_at: None}).unwrap();
                                }
                                if let Some(track) = playlist.get_next_track() {
                                    tx.playback.send(PlaybackActions::Que{track: Box::new(track)}).unwrap();
                                }
                            },
                            InputGlobal::Next => {
                                tx.playback.send(PlaybackActions::Next).unwrap();
                            },
                            InputGlobal::Stop => {
                                playlist.clear();
                                tx.playback.send(PlaybackActions::Clear).unwrap();
                                tx.update.send(false).unwrap();
                            },
                            InputGlobal::SkipBackward => todo!(),
                            InputGlobal::SkipForward  => todo!(),
                        });
                    },
                    StateActions::PlaybackNextTrack{error} => {
                        state.mutate(|_, _, playlist| {
                            playlist.next();
                            if let Some(track) = playlist.get_next_track() {
                                tx.playback.send(PlaybackActions::Que{track: Box::new(track)}).unwrap();
                            }
                        });
                    },
                    StateActions::PlaybackPlay()  => {
                        state.mutate(|_, _, playlist| playlist.resume());
                        tx.update.send(true).unwrap();
                    },
                    StateActions::PlaybackPause() => {
                        state.mutate(|_, _, playback| playback.pause());
                        tx.update.send(false).unwrap();
                    },
                    StateActions::ScanIsScanning{is_scanning} => state.mutate(|interface, _, _| {
                        interface.is_scanning = is_scanning;
                    }),
                    StateActions::ScanAddSong{track} => state.mutate(|_, library, _| {
                        library.new_track(*track);
                        info!("{} tracks", library.tracks.len());
                    }),
                    StateActions::ScanIndicatorRotate(interval) => {
                        state.mutate(|interface, _, _| {
                            interface.interval = interval;
                        });
                    },
                    StateActions::Resize{height, width} => {
                        state.mutate(|interface, _, _| {
                            interface.term_size.height = height as usize;
                            interface.term_size.width  = width as usize;
                        });
                    },
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
