use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use crate::tasks::listener_updater::UpdateActions;
use crate::types::types_msg_channels::MsgChannels;

use super::terminal_state::GlobalUiState;

//-////////////////////////////////////////////////////////////////////////////
static PLAYBACK_IS_LOADING  : AtomicBool = AtomicBool::new(false);
static PLAYBACK_PLAYING_ID  : AtomicU64  = AtomicU64 ::new(0);
static PLAYBACK_LOADING_ID  : AtomicU64  = AtomicU64 ::new(0);
static PLAYBACK_STATE       : AtomicU8   = AtomicU8  ::new(0);
static PLAYBACK_SINCE_NANO  : AtomicU32  = AtomicU32 ::new(0);
static PLAYBACK_ELAPSED_NANO: AtomicU32  = AtomicU32 ::new(0);
static PLAYBACK_LENGTH_NANO : AtomicU32  = AtomicU32 ::new(0);
static PLAYBACK_SINCE_SEC   : AtomicU64  = AtomicU64 ::new(0);
static PLAYBACK_ELAPSED_SEC : AtomicU64  = AtomicU64 ::new(0);
static PLAYBACK_LENGTH_SEC  : AtomicU64  = AtomicU64 ::new(0);
//-////////////////////////////////////////////////////////////////////////////
pub struct GlobalPlayback {}
#[derive(Debug)]
pub struct GlobalPlaybackSnapshot {
    pub playing : Option<u64>,
    pub loading : Option<u64>,
    pub since   : SystemTime,
    pub elapsed : Duration,
    pub state   : PlaybackState,
    pub duration: Duration,
    pub progress: f64,
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
impl GlobalPlayback {
    // -- Store -------------------------------------------
    pub fn start_playback(track_id: u64, elapsed: Duration, length: Duration) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        PLAYBACK_SINCE_SEC .store(now.as_secs(), Ordering::Relaxed);
        PLAYBACK_SINCE_NANO.store(now.subsec_nanos(), Ordering::Relaxed);
        PLAYBACK_ELAPSED_SEC .store(elapsed.as_secs(), Ordering::Relaxed);
        PLAYBACK_ELAPSED_NANO.store(elapsed.subsec_nanos(), Ordering::Relaxed);
        PLAYBACK_LENGTH_SEC .store(length.as_secs(), Ordering::Relaxed);
        PLAYBACK_LENGTH_NANO.store(length.subsec_nanos(), Ordering::Relaxed);
        PLAYBACK_STATE.store(PlaybackState::Playing as u8, Ordering::Relaxed);
        PLAYBACK_PLAYING_ID.store(track_id, Ordering::Relaxed);
    }

    pub fn pause_playback() {
        let state = GlobalPlayback::state();
        if state != PlaybackState::Playing {
            return;
        }

        let elapsed = GlobalPlayback::elapsed();
        PLAYBACK_ELAPSED_SEC .store(elapsed.as_secs(), Ordering::Relaxed);
        PLAYBACK_ELAPSED_NANO.store(elapsed.subsec_nanos(), Ordering::Relaxed);
        PLAYBACK_STATE.store(PlaybackState::Paused as u8, Ordering::Relaxed);
    }

    pub fn resume_playback() {
        if PlaybackState::get_state() != PlaybackState::Paused {
            return;
        };

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        PLAYBACK_SINCE_SEC .store(now.as_secs(), Ordering::Relaxed);
        PLAYBACK_SINCE_NANO.store(now.subsec_nanos(), Ordering::Relaxed);
        PLAYBACK_STATE.store(PlaybackState::Playing as u8, Ordering::Relaxed);
    }

    pub fn stop_playback() {
        if PlaybackState::get_state() == PlaybackState::Stopped {
            return;
        };

        PLAYBACK_ELAPSED_NANO.store(0, Ordering::Relaxed);
        PLAYBACK_ELAPSED_SEC.store(0, Ordering::Relaxed);
        PLAYBACK_STATE.store(PlaybackState::Stopped as u8, Ordering::Relaxed);
    }

    pub fn set_loading(track_id: Option<u64>, tx: &MsgChannels) {
        PLAYBACK_IS_LOADING.store(track_id.is_some(), Ordering::Relaxed);
        PLAYBACK_LOADING_ID.store(track_id.unwrap_or_default(), Ordering::Relaxed);
        tx.update.send(UpdateActions::LoadingTrack(track_id.is_some())).unwrap();
    }

    // -- Read --------------------------------------------
    pub fn state()   -> PlaybackState {PlaybackState::get_state()}
    pub fn playing() -> u64           {PLAYBACK_PLAYING_ID.load(Ordering::Relaxed)}
    pub fn loading() -> Option<u64>   {match PLAYBACK_IS_LOADING.load(Ordering::Relaxed) {
        true  => Some(PLAYBACK_LOADING_ID.load(Ordering::Relaxed)),
        false => None,
    }}
    pub fn since() -> SystemTime {UNIX_EPOCH + Duration::new(
        PLAYBACK_SINCE_SEC .load(Ordering::Relaxed),
        PLAYBACK_SINCE_NANO.load(Ordering::Relaxed),
    )}
    pub fn elapsed() -> Duration {
        let since = GlobalPlayback::since();
        let elapsed = Duration::new(
            PLAYBACK_ELAPSED_SEC .load(Ordering::Relaxed),
            PLAYBACK_ELAPSED_NANO.load(Ordering::Relaxed),
        );
        elapsed + since.elapsed().unwrap()
    }
    pub fn duration() -> Duration {Duration::new(
        PLAYBACK_LENGTH_SEC .load(Ordering::Relaxed),
        PLAYBACK_LENGTH_NANO.load(Ordering::Relaxed),
    )}
    pub fn progress() -> f64 {
        GlobalPlayback::elapsed().as_secs_f64() / GlobalPlayback::duration().as_secs_f64()
    }

    pub fn snapshot() -> GlobalPlaybackSnapshot {
        let state = GlobalPlayback::state();
        GlobalPlaybackSnapshot{
            state,
            playing: match state {
                PlaybackState::Stopped => None,
                PlaybackState::Loading |
                PlaybackState::Paused  |
                PlaybackState::Playing => Some(GlobalPlayback::playing()),
            },
            loading : GlobalPlayback::loading(),
            since   : GlobalPlayback::since(),
            elapsed : GlobalPlayback::elapsed(),
            duration: GlobalPlayback::duration(),
            progress: GlobalPlayback::progress(),
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum PlaybackState {
    Stopped = 0,
    Loading = 1,
    Paused  = 2,
    Playing = 3,
}

impl PlaybackState {
    fn get_state() -> PlaybackState {
        let raw = PLAYBACK_STATE.load(Ordering::Relaxed);
        let loading = PLAYBACK_IS_LOADING.load(Ordering::Relaxed);
        match (raw, loading) {
            (0, false) => PlaybackState::Stopped,
            (0, true ) |
            (1, _    ) => PlaybackState::Loading,
            (2, _    ) => PlaybackState::Paused,
            (3, _    ) => PlaybackState::Playing,
            _ => panic!("Playback state not defined for {:?}", raw),
        }
    }

    /// (width, character)
    pub fn icon(&self) -> char {
        match self {
            PlaybackState::Stopped => '⏹',
            PlaybackState::Loading => GlobalUiState::loading_icon(),
            PlaybackState::Paused  => '⏸',
            PlaybackState::Playing => '⏵',
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
