use crate::types::config::Config;
use crate::CONFIG;
use backtrace::Backtrace;
use color_backtrace::BacktracePrinter;
use color_eyre::eyre::Context;
use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use color_eyre::Section;
use directories::ProjectDirs;
use std::fs::read_to_string;
use std::fs::File;
use std::panic;
use std::thread;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn init() -> Result<()> {
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
            let thread = thread::current();
            let name   = thread.name().unwrap_or("<name not found>");

            let backtrace = BacktracePrinter::new().format_trace_to_string(&Backtrace::new())
                .unwrap_or("<Backtrace failed>".to_string());

            let location = panic_info.location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "unknown location".to_string());

            let message = panic_info.payload()
                .downcast_ref::<&str>()
                .unwrap_or(&"Unknown panic");

            error!("Thread '{}' panicked at {}: {}\n{}", name, location, message, backtrace);
        }));
    }

    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
