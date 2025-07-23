use crate::tasks::listener_playback::PlaybackActions;
use crate::tasks::listener_state::StateActions;
use crate::tasks::listener_tui::RenderActions;
use crate::tasks::listener_updater::UpdateActions;
use color_eyre::Result;
use crossbeam_channel::Sender;
use std::time::Instant;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
/// Struct with all channels for communication between threads
#[derive(Clone)]
pub struct MsgChannels {
    pub exit    : Sender<Result<String>>,
    pub playback: Sender<PlaybackActions>,
    pub state   : Sender<(Instant, StateActions)>,
    pub update  : Sender<UpdateActions>,
    pub delay   : Sender<Instant>,
    pub tui     : Sender<RenderActions>,
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
