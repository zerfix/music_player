use anyhow::Result;
use awedio::Sound;
use awedio::backends::CpalBackend;
use awedio::manager::Manager;
use awedio::sounds::MemorySound;
use awedio::sounds::open_file;
use awedio::sounds::wrappers::Controller;
use awedio::sounds::wrappers::Pausable;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;
use crate::types::types_msg_channels::MsgChannels;

//-//////////////////////////////////////////////////////////////////
pub enum PlaybackActions {
    Play{file_path: PathBuf, start_at: Option<Duration>},
    Que{file_path: PathBuf},
    Pause,
    Resume,
    /// duration from start of track
    Forward(Duration),
    Next,
    Callback,
    Clear,
}

pub fn start_playback_listener(rx: Receiver<PlaybackActions>, tx: MsgChannels) {
    let tx_playback = tx.tx_playback;
    let mut state = PlaybackManager::new().unwrap();

    while let Ok(msg) = rx.recv() {
        match msg {
            PlaybackActions::Play { file_path, start_at } => {
                state.que(file_path).unwrap(); // todo: handle file read error
                state.start(tx_playback.clone(), start_at);
            },
            PlaybackActions::Que { file_path } => {
                state.que(file_path).unwrap(); // todo: handle file read error
            },
            PlaybackActions::Callback => {
                // todo: inform state of track change
                state.next(tx_playback.clone())
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
        self.current_controller = None;
        self.current_notifier   = None;
        self.que.clear();
    }

    pub fn que(&mut self, path: PathBuf) -> Result<()> {
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
                let _ = notifier.recv();
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
