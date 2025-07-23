#![allow(dead_code)]
#![warn(unused_crate_dependencies)]

#[macro_use]
extern crate tracing;

//-////////////////////////////////////////////////////////////////////////////
mod init;
mod run;
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
    pub mod listener_updater;
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
        pub mod ui_time_util;
    }
    pub mod views {
        pub mod view_library;
        pub mod view_playback;
    }
    pub mod widgets {
        pub mod widget_playback_status;
    }
}
mod types {
    pub mod config;
    pub mod types_library_entry;
    pub mod types_msg_channels;
    pub mod types_tui;
}
//-////////////////////////////////////////////////////////////////////////////

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[static_init::dynamic]
static CONFIG: std::sync::OnceLock<crate::types::config::Config> = std::sync::OnceLock::new();

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    if init::init()? {
        run::run()?;
    }
    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
