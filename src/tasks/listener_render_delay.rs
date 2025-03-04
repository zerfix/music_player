use crate::tasks::listener_state::StateActions;
use crate::types::types_msg_channels::MsgChannels;
use crate::CONFIG;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use crossbeam_channel::Receiver;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

//-//////////////////////////////////////////////////////////////////
pub fn start_render_delay(tx: MsgChannels, rx: Receiver<Instant>) {
    if let Err(err) = render_delay(rx, &tx) {
        error!("Render delay error: {}", err);
        tx.exit.send(Err(err)).unwrap();
    }
}

fn render_delay(rx: Receiver<Instant>, tx: &MsgChannels) -> Result<()> {
    let framerate = CONFIG.get().ok_or(eyre!("Config not initialized!"))?.framerate;
    let frame_time = Duration::from_secs_f64(1.0 / framerate as f64);
    loop {
        let last_render = rx.recv()?;
        let sleep_duration = frame_time.saturating_sub(last_render.elapsed());
        sleep(sleep_duration);
        tx.state.send((Instant::now(), StateActions::Render()))?;
    }
}
//-//////////////////////////////////////////////////////////////////
