use color_eyre::Result;
use crossbeam_channel::Sender;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_msg_channels::MsgChannels;
use crate::ui::utils::ui_loading_icon_util::LOADING_ICONS_LEN;
use crate::ui::utils::ui_loading_icon_util::LOADING_SPEED_UP;
use crate::ui::utils::ui_loading_icon_util::LOADING_SPEED_DOWN;

const FRAMES: u16 = 100;

//-//////////////////////////////////////////////////////////////////
pub fn start_intervals(tx: MsgChannels) {
    let tx_tui = tx.tx_tui;
    let mut interval = 0u8;
    let mut acc      = 0u16;

    loop {
        // loading icon progresses on every interval
        // loading takes one round per second * speed_up / speed_down
        acc += 1;
        if acc > (FRAMES * LOADING_SPEED_DOWN) / ((FRAMES / LOADING_ICONS_LEN as u16) * LOADING_SPEED_UP) {
            acc = 0;
            interval += 1;
        }
        if interval >= LOADING_ICONS_LEN {interval = 0}

        if let Err(err) = sleep_and_render(&tx_tui, interval) {
            error!("{:?}", err);
            let _ = tx_tui.send(RenderActions::Exit).unwrap();
            break;
        }
    }
}

fn sleep_and_render(tx_tui: &Sender<RenderActions>, interval: u8) -> Result<()> {
    sleep(Duration::from_secs_f64(1.0 / 99.982));
    tx_tui.send(RenderActions::RenderRequest{render_start: Instant::now(), interval})?;
    Ok(())
}
//-//////////////////////////////////////////////////////////////////
