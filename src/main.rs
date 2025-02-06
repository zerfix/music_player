#![warn(unused_crate_dependencies)]

#[macro_use]
extern crate tracing;

//-////////////////////////////////////////////////////////////////////////////
mod enums {
    pub mod enum_input;
}
mod functions {
    pub mod functions_hash;
}
mod state {
    pub mod state_app;
    pub mod state_interface;
    pub mod state_library;
    pub mod state_playlist;
}
mod tasks {
    pub mod listener_input;
    pub mod listener_playback;
    pub mod listener_scanner;
    pub mod listener_state;
    pub mod listener_tui;
    pub mod loop_intervals;
}
mod traits {
    pub mod trait_listable;
}
mod ui {
    pub mod models {
        pub mod model_component_list_state;
    }
    pub mod utils {
        pub mod ui_text_util;
        pub mod ui_loading_icon_util;
    }
    pub mod views {
        pub mod view_library;
        pub mod view_playback;
    }
}
mod types {
    pub mod config;
    pub mod types_tui;
    pub mod types_library_entry;
    pub mod types_msg_channels;
}
//-////////////////////////////////////////////////////////////////////////////

use color_eyre::Report;
use crossbeam_channel::bounded;
use static_init::dynamic;
use std::fs::File;
use std::panic;
use std::thread;
use tracing::metadata::LevelFilter;
use tracing_subscriber::prelude::*;
use mimalloc::MiMalloc;
use crate::tasks::listener_input::start_input_listener;
use crate::tasks::listener_playback::start_playback_listener;
use crate::tasks::listener_scanner::start_fs_scanner_listener;
use crate::tasks::listener_state::start_state_listener;
use crate::tasks::listener_tui::start_tui_listener;
use crate::tasks::loop_intervals::start_intervals;
use crate::types::config::Config;
use crate::types::types_msg_channels::MsgChannels;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[global_allocator] static GLOBAL: MiMalloc = MiMalloc;
#[dynamic] static CONFIG: Config = Config::init();

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    // -- Init Logger -----------------------------------------------
    {
        if let Some(path) = &CONFIG.log_path {
            let file = File::create(path)?;
            let file_log = tracing_subscriber::fmt::layer()
                .with_writer(file)
                .with_filter(LevelFilter::from_level(CONFIG.log_level));
            tracing_subscriber::registry()
                .with(file_log)
                .init();
        }

        panic::set_hook(Box::new(|panic_info| {
            error!("Thread {}", panic_info.to_string().replacen(":\n", ": ", 1));
        }));
    }

    // -- Create Channels -------------------------------------------
    info!("Creating channels...");
    let (tx_exit    , rx_exit    ) = bounded(1);
    let (tx_playback, rx_playback) = bounded(16);
    let (tx_state   , rx_state   ) = bounded(256);
    let (tx_tui     , rx_tui     ) = bounded(1);

    let channels = || MsgChannels{
        tx_exit    : tx_exit.clone(),
        tx_playback: tx_playback.clone(),
        tx_state   : tx_state.clone(),
        tx_tui     : tx_tui.clone(),
    };

    // -- Create Threads --------------------------------------------
    info!("Crating threads...");
    {
        let _threads = [
            {
                let channels = channels();
                thread::spawn(move || start_tui_listener(rx_tui, channels))
            },
            {
                let channels = channels();
                thread::spawn(move || start_playback_listener(rx_playback, channels))
            },
            {
                let channels = channels();
                thread::spawn(move || start_state_listener(rx_state, channels))
            },
            {
                let channels = channels();
                thread::spawn(move || start_fs_scanner_listener(channels))
            },
            {
                let channels = channels();
                thread::spawn(move || start_input_listener(channels))
            },
            {
                let channels = channels();
                thread::spawn(move || start_intervals(channels))
            },
        ];

        // -- Wait for exit signal --------------------------------------
        match rx_exit.recv() {
            Ok(()) => info!("Exiting"),
            Err(err) => error!("Unreachable exit channel error: {:?}", err),
        }
    };

    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
