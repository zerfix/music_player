use crate::spawn_thread;
use crate::tasks::listener_state::StateActions;
use crate::types::types_library_entry::TrackFile;
use crate::types::types_msg_channels::MsgChannels;
use awedio::backends::CpalBackend;
use awedio::manager::Manager;
use awedio::sounds::open_file;
use awedio::sounds::wrappers::CompletionNotifier;
use awedio::sounds::wrappers::Controller;
use awedio::sounds::wrappers::Pausable;
use awedio::sounds::MemorySound;
use awedio::Sound;
use color_eyre::eyre::Context;
use color_eyre::Result;
use color_eyre::Section;
use crossbeam_channel::Receiver;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

static IS_PLAYING: AtomicBool = AtomicBool::new(false);

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
                    if let Err(err) = state.que(path) {
                        tx.state.send((Instant::now(), StateActions::PlaybackNextTrack{error: Some(err)}))?;
                        continue;
                    }
                    state.start(start_at)?;
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
                    state.next()?;
                    tx.state.send((Instant::now(), StateActions::PlaybackNextTrack{error: None}))?;
                },
                PlaybackActions::Pause => {
                    state.pause();
                },
                PlaybackActions::Resume => {
                    state.resume();
                },
                PlaybackActions::Forward(duration) => {
                    state.start(Some(duration))?;
                },
                PlaybackActions::Next => {
                    state.next()?;
                    tx.state.send((Instant::now(), StateActions::PlaybackNextTrack { error: None }))?;
                },
                PlaybackActions::Clear => {
                    state.clear();
                },
            },
        }
    }
}

struct PlaybackManager {
    channels  : MsgChannels,
    playback  : Option<Playback>,
    que       : VecDeque<MemorySound>,
}
struct Playback {
    pub manager   : Manager,
    pub backend   : CpalBackend,
    pub controller: Controller<Pausable<CompletionNotifier<MemorySound>>>,
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
        IS_PLAYING.store(false, Ordering::Relaxed);
        self.playback = None;
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
        start_at: Option<Duration>,
    ) -> Result<()> {
        if self.que.is_empty() {
            self.clear();
            return Ok(());
        }
        self.stop();
        if let Some(sound) = self.que.front() {
            let (    sound, notifier  ) = sound.clone().with_completion_notifier();
            let (mut sound, controller) = sound.pausable().controllable();

            if let Some(duration) = start_at {
                let _ = sound.skip(duration);
            }

            // Note that output is always sent to alsa as long as manager and backend lives.
            // Witch causes CPU usage and makes app show up in mixer.
            let (mut manager, backend) = awedio::start().context("Starting audio output")?;
            manager.play(Box::new(sound));
            IS_PLAYING.store(true, Ordering::Relaxed);

            spawn_thread!(self.channels.clone(), "song-complete-notifier", move |tx: MsgChannels| {
                let res = notifier.recv()
                    .context("Awaiting track playback completed callback")
                    .note("This error is expected when playback is intentionally stopped.");
                match res {
                    Err(err) => match IS_PLAYING.load(Ordering::Relaxed) {
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
            self.channels.state.send((Instant::now(), StateActions::PlaybackPlay()))?;
        }
        Ok(())
    }

    pub fn next(&mut self) -> Result<()> {
        self.que.pop_front();
        self.start(None)?;
        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(playback) = &mut self.playback {
            playback.controller.set_paused(true);
            self.channels.state.send((Instant::now(), StateActions::PlaybackPause())).unwrap();
        }
    }

    pub fn resume(&mut self) {
        if let Some(playback) = &mut self.playback {
            playback.controller.set_paused(false);
            self.channels.state.send((Instant::now(), StateActions::PlaybackPlay())).unwrap();
        }
    }
}
//-//////////////////////////////////////////////////////////////////
