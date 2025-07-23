use crate::globals::playback_state::GlobalPlayback;
use crate::globals::playback_state::PlaybackState;
use crate::spawn_thread;
use crate::tasks::listener_state::StateActions;
use crate::tasks::listener_updater::UpdateActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use awedio::backends::CpalBackend;
use awedio::manager::Manager;
use awedio::sounds::open_file;
use awedio::sounds::wrappers::CompletionNotifier;
use awedio::sounds::wrappers::Controller;
use awedio::sounds::MemorySound;
use awedio::Sound;
use color_eyre::eyre::Context;
use color_eyre::Result;
use color_eyre::Section;
use crossbeam_channel::Receiver;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::Instant;

//-//////////////////////////////////////////////////////////////////
pub enum PlaybackActions {
    NewTrack{track_id: u64, path: Box<Path>},
    Play{track: Box<TrackFile>, start_at: Option<Duration>},
    Que{track: Box<TrackFile>},
    Replay,
    Pause,
    Resume(Duration),
    Next,
    Callback,
    Clear,
}

pub fn start_playback_listener(tx: MsgChannels, rx: Receiver<PlaybackActions>) {
    if let Err(err) = playback_loop(rx, &tx) {
        error!("Playback error: {}", err);
        tx.exit.send(Err(err)).unwrap();
    }
}

pub fn playback_loop(rx: Receiver<PlaybackActions>, tx: &MsgChannels) -> Result<()> {
    let mut state = PlaybackManager::new(tx.clone()).context("Crating playback manager")?;
    let mut tracks: BTreeMap<u64, Box<Path>> = BTreeMap::new();

    loop {
        match rx.recv() {
            Err(err) => return Err(err.into()),
            Ok(msg) => match msg {
                PlaybackActions::NewTrack{track_id, path} => {
                    tracks.insert(track_id, path);
                },
                PlaybackActions::Play { track, start_at } => {
                    let path = match tracks.get(&track.id_track) {
                        Some(path) => path,
                        None => {
                            error!(
                                "Could not find path for track: {:?} {:?} {:?} {:?}, id: {}",
                                track.album_artist,
                                track.album_title,
                                track.track_number,
                                track.track_title,
                                track.id_track,
                            );
                            continue;
                        },
                    };
                    if let Err(err) = state.que(*track, path) {
                        tx.state.send((Instant::now(), StateActions::PlaybackNextTrack{error: Some(err)}))?;
                        continue;
                    }
                    state.start(start_at)?;
                    debug_assert!(GlobalPlayback::state() == PlaybackState::Playing);
                },
                PlaybackActions::Que { track } => {
                    let path = match tracks.get(&track.id_track) {
                        Some(path) => path,
                        None => {
                            error!(
                                "Could not find path for track: {:?} {:?} {:?} {:?}, id: {}",
                                track.album_artist,
                                track.album_title,
                                track.track_number,
                                track.track_title,
                                track.id_track,
                            );
                            continue;
                        },
                    };
                    if let Err(err) = state.que(*track, path) {
                        tx.state.send((Instant::now(), StateActions::PlaybackNextTrack{error: Some(err)}))?;
                    }
                },
                PlaybackActions::Callback => {
                    state.next()?;
                    tx.state.send((Instant::now(), StateActions::PlaybackNextTrack{error: None}))?;
                },
                PlaybackActions::Replay => {
                    state.start(None)?;
                    debug_assert!(GlobalPlayback::state() == PlaybackState::Playing);
                },
                PlaybackActions::Pause => {
                    state.pause();
                    debug_assert!(GlobalPlayback::state() == PlaybackState::Paused);
                },
                PlaybackActions::Resume(play_at) => {
                    state.resume(play_at)?;
                    debug_assert!(GlobalPlayback::state() == PlaybackState::Playing);
                },
                PlaybackActions::Next => {
                    state.next()?;
                    tx.state.send((Instant::now(), StateActions::PlaybackNextTrack { error: None }))?;
                },
                PlaybackActions::Clear => {
                    state.clear();
                    debug_assert!(GlobalPlayback::state() == PlaybackState::Stopped);
                },
            },
        }
    }
}

struct PlaybackManager {
    channels  : MsgChannels,
    playback  : Option<Playback>,
    que       : VecDeque<(TrackFile, MemorySound)>,
}
struct Playback {
    pub manager   : Manager,
    pub backend   : CpalBackend,
    pub controller: Controller<CompletionNotifier<MemorySound>>,
}

impl PlaybackManager {
    pub fn new(channels: MsgChannels) -> Result<PlaybackManager> {
        Ok(PlaybackManager {
            channels,
            playback: None,
            que: VecDeque::new(),
        })
    }

    pub fn stop(&mut self) {
        self.playback = None;
    }

    pub fn clear(&mut self) {
        self.stop();
        self.que.clear();
        GlobalPlayback::stop_playback();
    }

    pub fn que(&mut self, track: TrackFile, path: &Path) -> Result<()> {
        info!("queuing track {:?}", path);
        GlobalPlayback::set_loading(Some(track.id_track), &self.channels);
        let sound = open_file(path)?.into_memory_sound()?;
        GlobalPlayback::set_loading(None, &self.channels);
        self.que.push_back((track, sound));
        Ok(())
    }

    pub fn start(
        &mut self,
        start_at: Option<Duration>,
    ) -> Result<()> {
        if self.que.is_empty() {
            self.clear();
            return Ok(());
        }
        self.stop();
        if let Some((track, sound)) = self.que.front() {
            let (    sound, notifier  ) = sound.clone().with_completion_notifier();
            let (mut sound, controller) = sound.controllable();

            if let Some(duration) = start_at {
                let _ = sound.skip(duration);
            }

            // Note that output is always sent to alsa as long as manager and backend lives.
            // Witch causes CPU usage and makes app show up in mixer.
            let (mut manager, backend) = awedio::start().context("Starting audio output")?;
            manager.play(Box::new(sound));
            GlobalPlayback::start_playback(track.id_track, start_at.unwrap_or_default(), track.duration);

            let track_id = track.id_track;
            spawn_thread!(self.channels.clone(), "play-callback", move |tx: MsgChannels| {
                let res = notifier.recv()
                    .context("Awaiting track playback completed callback")
                    .note("This error is expected when playback is intentionally stopped.");
                match res {
                    Err(err) => match GlobalPlayback::state() == PlaybackState::Playing && GlobalPlayback::current_track() == track_id  {
                        true  => error!("{:?}", err),
                        false => info!("No longer listening for end of track. Track has been stopped."),
                    },
                    Ok(_) => tx.playback.send(PlaybackActions::Callback).unwrap(),
                }
            })?;

            self.playback = Some(Playback{
                manager,
                backend,
                controller
            });
            self.channels.update.send(UpdateActions::Playback(true)).unwrap();
        }
        Ok(())
    }

    pub fn next(&mut self) -> Result<()> {
        self.que.pop_front();
        self.start(None)?;
        Ok(())
    }

    pub fn pause(&mut self) {
        GlobalPlayback::pause_playback();
        self.stop();
        self.channels.update.send(UpdateActions::Playback(false)).unwrap();
    }

    pub fn resume(&mut self, resume_at: Duration) -> Result<()> {
        self.start(Some(resume_at))?;
        self.channels.update.send(UpdateActions::Playback(true)).unwrap();
        Ok(())
    }
}
//-//////////////////////////////////////////////////////////////////
