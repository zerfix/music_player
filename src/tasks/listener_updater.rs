use crate::globals::playback_state::GlobalPlayback;
use crate::globals::terminal_state::GlobalUiState;
use crate::types::types_msg_channels::MsgChannels;
use crate::tasks::listener_state::StateActions;
use crate::ui::utils::ui_loading_icon_util::next_loading_rotation_interval;
use crossbeam_channel::Receiver;
use std::sync::atomic::AtomicBool;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::thread;
use std::sync::atomic::Ordering;

//-////////////////////////////////////////////////////////////////////////////
pub enum UpdateActions {
    Playback(bool),
    LoadingLibrary(bool),
    LoadingTrack(bool),
}

#[derive(PartialEq, Eq)]
enum UpdateType {
    Clock,
    Progress,
    Loading,
}
//-////////////////////////////////////////////////////////////////////////////
static CLOCK   : AtomicBool = AtomicBool::new(false);
static PROGRESS: AtomicBool = AtomicBool::new(false);
static LOADING : AtomicBool = AtomicBool::new(false);
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
/// Notifies state of interval updates
pub fn start_updater(tx: MsgChannels, rx: Receiver<UpdateActions>) {
    let mut playback        = false;
    let mut loading_library = false;
    let mut loading_track   = false;

    loop {
        match rx.recv().unwrap() {
            UpdateActions::Playback(run)       => playback        = run,
            UpdateActions::LoadingLibrary(run) => loading_library = run,
            UpdateActions::LoadingTrack(run)   => loading_track   = run,
        };

        clock_loop(playback, &tx);
        progress_loop(playback, &tx);
        loading_loop(loading_track || loading_library, &tx);
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
fn clock_loop(run: bool, tx: &MsgChannels) {
    let running = CLOCK.load(Ordering::Relaxed);
    match (running, run) {
        (true , true ) |
        (false, false) => {},
        (true , false) => CLOCK.store(false, Ordering::Relaxed),
        (false, true ) => {
            CLOCK.store(true, Ordering::Relaxed);
            let tx_state = tx.state.clone();
            thread::Builder::new()
                .name("upt_clock".to_string())
                .spawn(move || {
                    loop {
                        if !CLOCK.load(Ordering::Relaxed) {
                            break;
                        }

                        let elapsed  = GlobalPlayback::elapsed().as_secs_f64();
                        let interval = 1.0;
                        let next     = interval - (elapsed % interval);

                        sleep(Duration::from_secs_f64(next));
                        tx_state.send((Instant::now(), StateActions::Update())).unwrap();
                    }
                })
                .unwrap();
        },
    };
}

fn progress_loop(run: bool, tx: &MsgChannels) {
    let running = PROGRESS.load(Ordering::Relaxed);
    match (running, run) {
        (true , true ) |
        (false, false) => {},
        (true , false) => PROGRESS.store(false, Ordering::Relaxed),
        (false, true ) => {
            PROGRESS.store(true, Ordering::Relaxed);
            let tx_state = tx.state.clone();
            thread::Builder::new()
                .name("upt_progress".to_string())
                .spawn(move || {
                    loop {
                        if !PROGRESS.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }

                        let duration = GlobalPlayback::duration().as_secs_f64();
                        let elapsed  = GlobalPlayback::elapsed().as_secs_f64();
                        let width    = GlobalUiState::progress_width() as f64;

                        let interval = duration / width;
                        let next     = interval - (elapsed % interval);

                        sleep(Duration::from_secs_f64(next));
                        tx_state.send((Instant::now(), StateActions::Update())).unwrap();
                    }
                })
                .unwrap();
        },
    };
}

fn loading_loop(run: bool, tx: &MsgChannels) {
    let running = LOADING.load(std::sync::atomic::Ordering::Relaxed);
    match (running, run) {
        (true , true ) |
        (false, false) => {},
        (true , false) => LOADING.store(false, Ordering::Relaxed),
        (false, true ) => {
            LOADING.store(true, Ordering::Relaxed);
            let tx_state = tx.state.clone();
            thread::Builder::new()
                .name("upt_progress".to_string())
                .spawn(move || {
                    loop {
                        if !LOADING.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }

                        let next = next_loading_rotation_interval(SystemTime::now());
                        sleep(next);

                        GlobalUiState::increment_loading_rotation();
                        tx_state.send((Instant::now(), StateActions::Update())).unwrap();
                    }
                })
                .unwrap();
        },
    };
}
//-////////////////////////////////////////////////////////////////////////////
