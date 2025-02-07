use color_eyre::eyre::Context;
use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use std::path::PathBuf;
use tracing::metadata::Level;
use serde::Deserialize;
use serde::Serialize;
use directories::UserDirs;
use std::fs::write;
use std::fs::create_dir_all;

//-/////////////////////////////////////////////////////////////////
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConfLogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl ConfLogLevel {
    pub fn to_level(&self) -> Level {
        match self {
            ConfLogLevel::Error   => Level::ERROR,
            ConfLogLevel::Warning => Level::WARN,
            ConfLogLevel::Info    => Level::INFO,
            ConfLogLevel::Debug   => Level::DEBUG,
            ConfLogLevel::Trace   => Level::TRACE,
        }
    }
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub framerate: u16,
    pub media_dirs: Vec<PathBuf>,
    pub logging: ConfLog,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct ConfLog {
    pub enable_logging: bool,
    pub log_path: PathBuf,
    pub log_level: ConfLogLevel,
}

impl Config {
    pub fn write_default(conf_path: &PathBuf) -> Result<Config> {
        let media_dirs = match UserDirs::new().and_then(|d| d.audio_dir().map(|d| d.to_path_buf())) {
            Some(dir) => vec![dir],
            None      => vec![],
        };

        let config_file = Config{
            framerate: 30,
            media_dirs,
            logging: ConfLog{
                enable_logging: false,
                log_path: PathBuf::default(),
                log_level: ConfLogLevel::Info,
            }
        };
        let config_file_str = toml::to_string(&config_file).context("Creating default config file")?;

        // write default
        let conf_dir = conf_path.parent().context("Getting parrent folder of config file to make sure it exsists before trying to writie config")?;
        if !conf_dir.exists() {create_dir_all(conf_dir).context(format!("Create all parrent folders of config file: {}", conf_dir.to_str().unwrap()))?}
        write(&conf_path, &config_file_str).context(format!("Writing default config file at {}", conf_path.to_str().unwrap()))?;

        Ok(config_file)
    }
}
//-//////////////////////////////////////////////////////////////////
