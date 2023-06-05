#![allow(clippy::result_large_err)]
#![warn(unused_crate_dependencies)]

#[macro_use]
extern crate tracing;

//-////////////////////////////////////////////////////////////////////////////
mod enums {
    pub mod enum_navigate;
}
mod functions {
    pub mod functions_hash;
}
mod state {
    pub mod state_app;
    pub mod state_config;
    pub mod state_interface;
    pub mod state_library;
}
mod tasks {
    pub mod listener_input;
    pub mod listener_playback;
    pub mod listener_scanner;
    pub mod listener_state;
    pub mod loop_tui;
}
mod traits {
    pub mod trait_listable;
}
mod ui {
    pub mod models {
        pub mod model_component_list_state;
    }
    pub mod views {
        pub mod view_library;
        pub mod view_playback;
    }
}
mod types {
    pub mod types_library_entry;
    pub mod types_msg_channels;
}
//-////////////////////////////////////////////////////////////////////////////

use std::fs::File;
use std::sync::mpsc::channel;
use std::thread;
use tracing_subscriber::prelude::*;
use crate::state::state_app::AppState;
use crate::tasks::listener_playback::start_playback_task;
use crate::tasks::listener_input::start_input_listener;
use crate::tasks::listener_scanner::ScannerActions;
use crate::tasks::listener_scanner::start_fs_scanner_listener;
use crate::tasks::listener_state::start_state_listener;
use crate::tasks::loop_tui::start_tui_listener;
use crate::types::types_msg_channels::MsgChannels;
use tracing::metadata::LevelFilter;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
fn main() {
    let state = AppState::init();
    // -- Init Logger -----------------------------------------------
    {
        let config = state.config.clone();
        if let Some(path) = &config.log_path {
            dbg!(path);
            let file = File::create(path).unwrap();
            let file_log = tracing_subscriber::fmt::layer()
                .with_writer(file)
                .with_filter(LevelFilter::from_level(config.log_level));
            tracing_subscriber::registry()
                .with(file_log)
                .init();
        }
    }

    // -- Create Channels -------------------------------------------
    info!("Creating channels...");
    let (tx_exit    , rx_exit    ) = channel();
    let (tx_scanner , rx_scanner ) = channel();
    let (tx_state   , rx_state   ) = channel();
    let (tx_playback, rx_playback) = channel();
    let (tx_tui     , rx_tui     ) = channel();

    let channels = || MsgChannels{
        tx_exit    : tx_exit.clone(),
        tx_scanner : tx_scanner.clone(),
        tx_state   : tx_state.clone(),
        tx_playback: tx_playback.clone(),
        tx_tui     : tx_tui.clone(),
    };

    // -- Create Threads --------------------------------------------
    info!("Crating threads...");
    {
        let _threads = [
            {
                let state = state.clone();
                let channels = channels();
                thread::spawn(move || start_state_listener(rx_state, channels, state))
            },
            {
                let channels = channels();
                thread::spawn(move || start_fs_scanner_listener(rx_scanner, channels))
            },
            {
                let channels = channels();
                thread::spawn(move || start_input_listener(channels))
            },
            {
                info!("Spawning playback thread");
                let channels = channels();
                thread::spawn(move || start_playback_task(rx_playback, channels))
            },
        ];
        let tui = {
            let channels = channels();
            thread::spawn(move || start_tui_listener(rx_tui, channels))
        };

        for dir in &state.config.media_dirs {
            tx_scanner.send(ScannerActions::ScanDir{dir: dir.clone()}).unwrap()
        }

        // -- Wait for exit signal --------------------------------------
        if rx_exit.recv().is_ok() {
            info!("Exit requested");
            if let Err(err) = tui.join() {
                error!("tui exit err: {:?}", err);
            }
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
