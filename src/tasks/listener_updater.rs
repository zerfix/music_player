use crate::types::types_msg_channels::MsgChannels;
use crate::tasks::listener_state::StateActions;
use crossbeam_channel::Receiver;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

//-////////////////////////////////////////////////////////////////////////////
// 
//-////////////////////////////////////////////////////////////////////////////
/// Spams the state thread to trigger updates of playback elapsed and playback progress
pub fn start_updater(tx: MsgChannels, rx: Receiver<bool>) {
    loop {
        if let true = rx.recv().unwrap() {
            loop {
                tx.state.send((Instant::now(), StateActions::Update())).unwrap();
                if let Ok(false) = rx.try_recv() {
                    break;
                }

                sleep(Duration::from_secs_f64(1.0/3.0));
            }
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
// 
//-////////////////////////////////////////////////////////////////////////////
