#![allow(dead_code)]
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
    pub mod listener_render_delay;
    pub mod listener_scanner;
    pub mod listener_state;
    pub mod listener_tui;
}
mod traits {
    pub mod trait_listable;
}
mod ui {
    pub mod models {
        pub mod model_component_list_state;
    }
    pub mod utils {
        pub mod ui_loading_icon_util;
        pub mod ui_text_util;
    }
    pub mod views {
        pub mod view_library;
        pub mod view_playback;
    }
}
mod types {
    pub mod config;
    pub mod types_library_entry;
    pub mod types_msg_channels;
    pub mod types_tui;
}
//-////////////////////////////////////////////////////////////////////////////

use crate::tasks::listener_input::start_input_listener;
use crate::tasks::listener_playback::start_playback_listener;
use crate::tasks::listener_render_delay::start_render_delay;
use crate::tasks::listener_scanner::start_fs_scanner_listener;
use crate::tasks::listener_state::start_state_listener;
use crate::tasks::listener_tui::start_tui_listener;
use crate::tasks::listener_tui::RenderActions;
use crate::types::config::Config;
use crate::types::types_msg_channels::MsgChannels;
use color_eyre::eyre::Context;
use color_eyre::eyre::ContextCompat;
use color_eyre::Report;
use color_eyre::Section;
use crossbeam_channel::bounded;
use directories::ProjectDirs;
use mimalloc::MiMalloc;
use static_init::dynamic;
use std::fs::read_to_string;
use std::fs::File;
use std::panic;
use std::sync::OnceLock;
use std::thread;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[global_allocator] static GLOBAL: MiMalloc = MiMalloc;
#[dynamic] static CONFIG: OnceLock<Config> = OnceLock::new();

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    // -- Init ------------------------------------------------------
    {
        let config_path = ProjectDirs::from("", "", "music_player")
            .context("Getting project paths")?
            .config_dir()
            .join("config.toml");

        // -- Config ----------------------------
        {
            if !config_path.exists() {
                let config = Config::write_default(&config_path)?;
                println!("Config written to: {}", config_path.to_string_lossy());
                match config.media_dirs.first() {
                    Some(dir) => println!(
                        "Modify config file to specify your music folder paths or re-run to use default {} dir",
                        dir.to_string_lossy(),
                    ),
                    None => println!(
                        "Please add your music directory to the config file at {}",
                        config_path.to_string_lossy()
                    ),
                }
                return Ok(());
            }

            let config_raw = read_to_string(&config_path)
                .context(format!("Reading config file at {}", config_path.to_string_lossy()))?;
            let config_parsed: Config = toml::from_str(&config_raw)
                .context(format!("Parsing config file at {}", config_path.to_string_lossy()))?;
            let config = config_parsed.fix_home_dir_paths()
                .context("Replacing ~ with full path")?;

            if config.media_dirs.is_empty() {
                println!("Please add your music directory to the config file at {}", config_path.to_string_lossy());
                return Ok(());
            }

            CONFIG.set(config).unwrap();
        }

        // -- Logging ----------------------------
        {
            let config = &CONFIG.get().unwrap().logging;
            let log_level = config.log_level.to_level();
            if config.enable_logging {
                let file = File::create(&config.log_path)
                    .context(format!("Trying to create log file at {}", config.log_path.to_string_lossy()))
                    .note(format!(
                        "Double check the `log_path` value in your configuration file at {}",
                        config_path.to_string_lossy(),
                    ))?;
                let file_log = tracing_subscriber::fmt::layer()
                    .with_writer(file)
                    .with_filter(match config.log_libraries {
                        true  => EnvFilter::new(log_level.as_str()),
                        false => EnvFilter::new(format!("{}={}", env!("CARGO_PKG_NAME"), log_level)),
                    });
                tracing_subscriber::registry().with(file_log).init();
            }

            // log hook on thread panic
            panic::set_hook(Box::new(|panic_info| {
                error!("Thread {}", panic_info.to_string().replacen(":\n", ": ", 1));
            }));
        }
    }

    // -- Create Channels -------------------------------------------
    info!("Creating channels...");
    let (tx_exit    , rx_exit    ) = bounded(1);
    let (tx_playback, rx_playback) = bounded(16);
    let (tx_state   , rx_state   ) = bounded(256);
    let (tx_delay   , rx_delay   ) = bounded(1);
    let (tx_tui     , rx_tui     ) = bounded(1);
    let (tx_tui_done, rx_tui_done) = bounded(0);

    let channels = || MsgChannels{
        exit    : tx_exit.clone(),
        playback: tx_playback.clone(),
        state   : tx_state.clone(),
        delay   : tx_delay.clone(),
        tui     : tx_tui.clone(),
    };

    // -- Create Threads --------------------------------------------
    info!("Creating threads...");
    {
        let _threads = [
            {
                let channels = channels();
                thread::spawn(move || start_tui_listener(rx_tui, channels, tx_tui_done))
            },
            {
                let channels = channels();
                thread::spawn(move || start_playback_listener(rx_playback, channels))
            },
            {
                let channels = channels();
                thread::spawn(move || start_render_delay(rx_delay, channels))
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
        ];

        // -- Wait for exit signal --------------------------------------
        match rx_exit.recv() {
            Err(err) => error!("Exit channel error: {:?}", err),
            Ok(msg) => {
                info!("Exiting");
                let _ = tx_tui.send(RenderActions::Exit);
                let _ = rx_tui_done.recv();

                match msg {
                    Err(err) => return Err(err),
                    Ok(msg) => {
                        if !msg.is_empty() {
                            info!("Exit msg: {}", msg);
                            println!("{}", msg);
                        }
                    },
                }
            },
        }
    };

    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
