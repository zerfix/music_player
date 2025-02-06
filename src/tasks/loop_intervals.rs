use color_eyre::Result;
use crossbeam_channel::Sender;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_msg_channels::MsgChannels;

//-//////////////////////////////////////////////////////////////////
pub fn start_intervals(tx: MsgChannels) {
    let tx_tui = tx.tx_tui;

    loop {
        if let Err(err) = sleep_and_render(&tx_tui) {
            error!("{:?}", err);
            let _ = tx_tui.send(RenderActions::Exit).unwrap();
            break;
        }
    }
}

fn sleep_and_render(tx_tui: &Sender<RenderActions>) -> Result<()> {
    sleep(Duration::from_secs_f64(1.0 / 99.982));
    tx_tui.send(RenderActions::RenderRequest{render_start: Instant::now()})?;
    Ok(())
}
//-//////////////////////////////////////////////////////////////////
