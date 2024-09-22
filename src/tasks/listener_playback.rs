use color_eyre::Result;
use awedio::Sound;
use awedio::backends::CpalBackend;
use awedio::manager::Manager;
use awedio::sounds::MemorySound;
use awedio::sounds::open_file;
use awedio::sounds::wrappers::Controller;
use awedio::sounds::wrappers::Pausable;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;
use crate::types::types_msg_channels::MsgChannels;
use crate::tasks::listener_state::StateActions;

//-//////////////////////////////////////////////////////////////////
pub enum PlaybackActions {
    Play{path: Box<Path>, start_at: Option<Duration>},
    Que{path: Box<Path>},
    Pause,
    Resume,
    /// duration from start of track
    Forward(Duration),
    Next,
    Callback,
    Clear,
}

pub fn start_playback_listener(rx: Receiver<PlaybackActions>, tx: MsgChannels) {
    let tx_state = tx.tx_state;
    let tx_playback = tx.tx_playback;
    let mut state = PlaybackManager::new().unwrap();

    while let Ok(msg) = rx.recv() {
        match msg {
            PlaybackActions::Play { path, start_at } => {
                if let Err(err) = state.que(&path) {
                    tx_state.send(StateActions::PlaybackNextTrack{error: Some(err)}).unwrap();
                    continue;
                }
                state.start(tx_playback.clone(), start_at);
            },
            PlaybackActions::Que { path } => {
                if let Err(err) = state.que(&path) {
                    tx_state.send(StateActions::PlaybackNextTrack{error: Some(err)}).unwrap()
                }
            },
            PlaybackActions::Callback => {
                state.next(tx_playback.clone());
                tx_state.send(StateActions::PlaybackNextTrack{error: None}).unwrap();
            },
            PlaybackActions::Pause => {
                state.pause();
            },
            PlaybackActions::Resume => {
                state.resume();
            },
            PlaybackActions::Forward(duration) => {
                state.start(tx_playback.clone(), Some(duration));
            },
            PlaybackActions::Next => {
                state.next(tx_playback.clone());
            },
            PlaybackActions::Clear => {
                state.clear();
            },
        }
    }
}

struct PlaybackManager {
    manager: Manager,
    backend: CpalBackend,
    current_controller: Option<Controller<Pausable<MemorySound>>>,
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

    pub fn clear(&mut self) {
        info!("stopping playback");
        if let Some(controller) = &mut self.current_controller {
            controller.set_paused(true);
        }
        self.current_controller = None;
        self.current_notifier   = None;
        self.que.clear();
    }

    pub fn que(&mut self, path: &Path) -> Result<()> {
        let sound = open_file(path)?.into_memory_sound()?;
        self.que.push_front(sound);
        Ok(())
    }

    pub fn start(
        &mut self,
        tx_playback: Sender<PlaybackActions>,
        start_at: Option<Duration>,
    ) {
        if let Some(sound) = self.que.front() {
            let (sound, controller) = sound.clone()
                .pausable()
                .controllable();
            let (mut sound, notifier) = sound
                .with_completion_notifier();

            if let Some(duration) = start_at {
                let _ = sound.skip(duration);
            }

            let notifier = thread::spawn(move || {
                info!("Listening to notifier");
                notifier.recv().unwrap();
                info!("Notifier callback ok");
                if let Err(err) = tx_playback.send(PlaybackActions::Callback) {
                    warn!("{:?}", err);
                };
            });

            if self.current_controller.is_some() {self.clear();}
            self.current_controller = Some(controller);
            self.current_notifier = Some(notifier);
            self.manager.play(Box::new(sound));
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
