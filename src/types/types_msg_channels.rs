use std::sync::mpsc::Sender;
use crate::tasks::listener_playback::PlaybackActions;
use crate::tasks::listener_scanner::ScannerActions;
use crate::tasks::listener_state::StateActions;
use crate::tasks::listener_tui::RenderActions;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
/// Struct with all channels for communication between threads
#[derive(Clone)]
pub struct MsgChannels {
    pub tx_exit    : Sender<()>,
    pub tx_playback: Sender<PlaybackActions>,
    pub tx_scanner : Sender<ScannerActions>,
    pub tx_state   : Sender<StateActions>,
    pub tx_tui     : Sender<RenderActions>,
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
