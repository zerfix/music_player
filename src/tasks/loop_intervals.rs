use color_eyre::Result;
use crossbeam_channel::Sender;
use std::thread::sleep;
use std::time::Duration;
use std::time::SystemTime;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_msg_channels::MsgChannels;

//-//////////////////////////////////////////////////////////////////
pub fn start_intervals(tx: MsgChannels) {
    let tx_tui   = tx.tx_tui;

    let frametime = Duration::from_secs_f64(1.0 / (99.982));
    let mut now = SystemTime::now();
    let mut next_frame = now.clone();

    loop {
        now = SystemTime::now();
        while next_frame < now {
            next_frame += frametime;
        }
        if let Err(err) = sleep_and_render(&tx_tui, now, next_frame) {
            error!("{:?}", err);
            let _ = tx_tui.send(RenderActions::Exit);
            break;
        }
    }
}

fn sleep_and_render(tx_tui: &Sender<RenderActions>, now: SystemTime, next: SystemTime) -> Result<()> {
    sleep(next.duration_since(now)?);
    tx_tui.send(RenderActions::RenderRequest{render_target: next, render_start: SystemTime::now()})?;
    Ok(())
}
//-//////////////////////////////////////////////////////////////////
