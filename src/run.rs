use crate::tasks::listener_input::start_input_listener;
use crate::tasks::listener_playback::start_playback_listener;
use crate::tasks::listener_render_delay::start_render_delay;
use crate::tasks::listener_scanner::start_fs_scanner_listener;
use crate::tasks::listener_state::start_state_listener;
use crate::tasks::listener_tui::reset_terminal;
use crate::tasks::listener_tui::start_tui_listener;
use crate::tasks::listener_tui::RenderActions;
use crate::tasks::listener_updater::start_updater;
use crate::types::types_msg_channels::MsgChannels;
use color_eyre::eyre::Context;
use color_eyre::Result;
use crossbeam_channel::bounded;
use std::thread;

//-////////////////////////////////////////////////////////////////////////////
macro_rules! spawn_thread {
    ($channels:expr, $name:expr, $func:expr) => {{
        let tx = $channels;
        thread::Builder::new()
            .name($name.to_string())
            .spawn(move || $func(tx))
            .context(format!("starting {} thread", $name))
    }}
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn run() -> Result<()>{
    // -- Create Channels -------------------------------------------
    info!("Creating channels...");
    let (tx_exit    , rx_exit    ) = bounded(0);
    let (tx_playback, rx_playback) = bounded(16);
    let (tx_state   , rx_state   ) = bounded(256);
    let (tx_update  , rx_update  ) = bounded(1);
    let (tx_delay   , rx_delay   ) = bounded(1);
    let (tx_tui     , rx_tui     ) = bounded(1);
    let (tx_tui_done, rx_tui_done) = bounded(0);

    let tx = || MsgChannels{
        exit    : tx_exit.clone(),
        playback: tx_playback.clone(),
        state   : tx_state.clone(),
        update  : tx_update.clone(),
        delay   : tx_delay.clone(),
        tui     : tx_tui.clone(),
    };

    // -- Create Threads --------------------------------------------
    info!("Creating threads...");

    spawn_thread!(tx(), "tui"            , move |tx| start_tui_listener(tx, tx_tui_done, rx_tui))?;
    spawn_thread!(tx(), "playback"       , move |tx| start_playback_listener(tx, rx_playback)   )?;
    spawn_thread!(tx(), "render-delay"   , move |tx| start_render_delay(tx, rx_delay)           )?;
    spawn_thread!(tx(), "render-interval", move |tx| start_updater(tx, rx_update)               )?;
    spawn_thread!(tx(), "state"          , move |tx| start_state_listener(tx, rx_state)         )?;
    spawn_thread!(tx(), "scanner"        , move |tx| start_fs_scanner_listener(tx)              )?;
    spawn_thread!(tx(), "input"          , move |tx| start_input_listener(tx)                   )?;

    // -- Wait for exit signal --------------------------------------
    info!("Ready for exit signal");
    match rx_exit.recv() {
        Err(err) => error!("Exit signal channel error: {:?}", err),
        Ok(msg) => {
            info!("Received exit signal. Running exit procedure...");

            // reset terminal
            let _ = tx_tui.send(RenderActions::Exit);
            let _ = rx_tui_done.recv();
            reset_terminal();

            // exit message
            match msg {
                Err(err) => {
                    info!("Exit with error");
                    error!("{:?}", err);
                    return Err(err);
                },
                Ok(msg) => {
                    if !msg.is_empty() {
                        info!("Exit msg: {}", msg);
                        println!("{}", msg);
                    }
                    info!("Exit done");
                },
            }
        },
    };

    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
