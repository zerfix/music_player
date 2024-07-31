use std::thread::sleep;
use std::time::Duration;
use std::time::SystemTime;
use crate::tasks::listener_state::StateActions;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_msg_channels::MsgChannels;

//-//////////////////////////////////////////////////////////////////
pub fn start_intervals(tx: MsgChannels) {
    let tx_state = tx.tx_state;
    let tx_tui   = tx.tx_tui;

    let frametime = Duration::from_secs_f64(1.0 / 99.982);
    let mut next_frame = SystemTime::now() + frametime;

    loop {
        sleep(next_frame.duration_since(SystemTime::now()).unwrap_or(Duration::from_secs(1)));
        if let Err(err) = tx_state.send(StateActions::Render{render_start: next_frame}) {
            error!("{:?}", err);
            let _ = tx_tui.send(RenderActions::Exit);
            break;
        }
        next_frame += frametime;
    }
}
//-//////////////////////////////////////////////////////////////////
