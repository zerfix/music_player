use awedio::backends::CpalBackend;
use awedio::manager::Manager;
use awedio::Sound;
use awedio::sounds::MemorySound;
use awedio::sounds::open_file;
use awedio::sounds::wrappers::CompletionNotifier;
use awedio::sounds::wrappers::Controller;
use awedio::sounds::wrappers::Pausable;
use color_eyre::eyre::Context;
use color_eyre::Result;
use std::collections::VecDeque;
use std::path::Path;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;
use std::collections::BTreeMap;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use crate::tasks::listener_state::StateActions;

//-//////////////////////////////////////////////////////////////////
pub enum PlaybackActions {
    NewTrack{track_id: u64, path: Box<Path>},
    Play{track: Box<TrackFile>, start_at: Option<Duration>},
    Que{track: Box<TrackFile>},
    Pause,
    Resume,
    /// duration from start of track
    Forward(Duration),
    Next,
    Callback,
    Clear,
}

pub fn start_playback_listener(rx: Receiver<PlaybackActions>, tx: MsgChannels) {
    if let Err(err) = playback_loop(rx, &tx) {
        error!("Playback error: {}", err);
        tx.exit.send(Err(err)).unwrap();
    }
}

pub fn playback_loop(rx: Receiver<PlaybackActions>, tx: &MsgChannels) -> Result<()> {
    let mut state = PlaybackManager::new().context("Crating playback manager")?;
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
                    if let Err(err) = state.que(path) {
                        tx.state.send((Instant::now(), StateActions::PlaybackNextTrack{error: Some(err)}))?;
                        continue;
                    }
                    state.start(tx.playback.clone(), start_at);
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
                    if let Err(err) = state.que(path) {
                        tx.state.send((Instant::now(), StateActions::PlaybackNextTrack{error: Some(err)}))?;
                    }
                },
                PlaybackActions::Callback => {
                    state.next(tx.playback.clone());
                    tx.state.send((Instant::now(), StateActions::PlaybackNextTrack{error: None}))?;
                },
                PlaybackActions::Pause => {
                    state.pause();
                },
                PlaybackActions::Resume => {
                    state.resume();
                },
                PlaybackActions::Forward(duration) => {
                    state.start(tx.playback.clone(), Some(duration));
                },
                PlaybackActions::Next => {
                    state.next(tx.playback.clone());
                    tx.state.send((Instant::now(), StateActions::PlaybackNextTrack { error: None }))?;
                },
                PlaybackActions::Clear => {
                    state.clear();
                },
            }
        }
    }
}

struct PlaybackManager {
    manager: Manager,
    backend: CpalBackend,
    current_controller: Option<Controller<Pausable<CompletionNotifier<MemorySound>>>>,
    current_notifier: Option<JoinHandle<()>>,
    que: VecDeque::<MemorySound>
}

impl PlaybackManager {
    pub fn new() -> Result<PlaybackManager> {
        let (manager, backend) = awedio::start()?;
        Ok(PlaybackManager {
            manager,
            backend,
            current_controller: None,
            current_notifier: None,
            que: VecDeque::new(),
        })
    }

    pub fn stop(&mut self) {
        if let Some(controller) = &mut self.current_controller {
            controller.set_paused(true);
        }
        self.current_controller = None;
        self.current_notifier   = None;
    }

    pub fn clear(&mut self) {
        self.stop();
        self.que.clear();
    }

    pub fn que(&mut self, path: &Path) -> Result<()> {
        info!("queuing track {:?}", path);
        let sound = open_file(path)?.into_memory_sound()?;
        self.que.push_back(sound);
        Ok(())
    }

    pub fn start(
        &mut self,
        tx_playback: Sender<PlaybackActions>,
        start_at: Option<Duration>,
    ) {
        if let Some(sound) = self.que.front() {
            let (sound, notifier) = sound.clone()
                .with_completion_notifier();
            let (mut sound, controller) = sound
                .pausable()
                .controllable();

            if let Some(duration) = start_at {
                let _ = sound.skip(duration);
            }

            if self.current_controller.is_some() {self.stop()}
            self.manager.play(Box::new(sound));

            let notifier = thread::spawn(move || {
                notifier.recv().unwrap();
                tx_playback.send(PlaybackActions::Callback).unwrap();
            });

            self.current_controller = Some(controller);
            self.current_notifier = Some(notifier);
        }
    }

    pub fn next(&mut self, tx_playback: Sender<PlaybackActions>) {
        self.que.pop_front();
        self.start(tx_playback, None);
    }

    pub fn pause(&mut self) {
        if let Some(controller) = &mut self.current_controller {
            controller.set_paused(true);
        }
    }

    pub fn resume(&mut self) {
        if let Some(controller) = &mut self.current_controller {
            controller.set_paused(false);
        }
    }
}
//-//////////////////////////////////////////////////////////////////
