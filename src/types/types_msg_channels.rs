use crate::state::state_app::AppState;
use crate::tasks::listener_playback::PlaybackActions;
use crate::tasks::listener_scanner::ScannerActions;
use crate::tasks::listener_state::StateActions;
use std::sync::mpsc::Sender;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
/// Struct with all channels for communication between threads
#[derive(Clone)]
pub struct MsgChannels {
    pub tx_exit    : Sender<()>,
    pub tx_scanner : Sender<ScannerActions>,
    pub tx_playback: Sender<PlaybackActions>,
    pub tx_state   : Sender<StateActions>,
    pub tx_tui     : Sender<Option<AppState>>,
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
