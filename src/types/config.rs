use dirs::home_dir;
use std::path::PathBuf;
use tracing::metadata::Level;

//-//////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct Config {
    pub media_dirs: Vec<PathBuf>,
    pub log_path: Option<PathBuf>,
    pub log_level: Level,
}

impl Config {
    pub fn init() -> Config {
        Config{
            media_dirs: vec!["/mnt/musikk".parse().unwrap()],
            log_path: Some(home_dir().unwrap().join("music_player_debug.log")),
            log_level: Level::INFO,
        }
    }
}
//-//////////////////////////////////////////////////////////////////
