use color_eyre::eyre::Context;
use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use directories::BaseDirs;
use directories::UserDirs;
use serde::Deserialize;
use serde::Serialize;
use std::fs::create_dir_all;
use std::fs::write;
use std::path::PathBuf;
use tracing::metadata::Level;

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
    pub log_libraries: bool,
}

impl Config {
    pub fn write_default(conf_path: &PathBuf) -> Result<Config> {
        let media_dirs = match UserDirs::new().and_then(|d| d.audio_dir().map(|d| d.to_path_buf())) {
            Some(dir) => vec![dir],
            None      => vec![],
        };

        let config_file = Config{
            framerate: 60,
            media_dirs,
            logging: ConfLog{
                enable_logging: false,
                log_path: PathBuf::default(),
                log_level: ConfLogLevel::Info,
                log_libraries: false,
            },
        };
        let config_file_str = toml::to_string(&config_file).context("Creating default config file")?;

        // write default
        let conf_dir = conf_path.parent()
            .context("Getting parent folder of config file to make sure it exists before trying to write config")?;
        if !conf_dir.exists() {
            create_dir_all(conf_dir)
                .context(format!("Creating all parent folders of config file: {}", conf_dir.to_string_lossy()))?
        }
        write(conf_path, &config_file_str)
            .context(format!("Writing default config file at {}", conf_path.to_string_lossy()))?;

        Ok(config_file)
    }

    pub fn fix_home_dir_paths(mut self) -> Result<Config> {
        let base_dirs = BaseDirs::new();

        self.logging.log_path   = expand_home_dir(&base_dirs, self.logging.log_path)?;
        self.media_dirs = self.media_dirs.into_iter()
            .map(|path| expand_home_dir(&base_dirs, path))
            .collect::<Result<Vec<PathBuf>>>()?;

        Ok(self)
    }
}
//-//////////////////////////////////////////////////////////////////
fn expand_home_dir(base_dirs: &Option<BaseDirs>, path: PathBuf) -> Result<PathBuf> {
    match (path.starts_with("~"), base_dirs) {
        (false, _) => Ok(path),
        (true, None) => Err(eyre!("Could not find home dir to replace ~ in path {}", path.to_string_lossy())),
        (true, Some(base_dirs)) => {
            let home_dir = base_dirs.home_dir();
            let mut buf = home_dir.to_path_buf();
            buf.push(path.strip_prefix("~/").context(format!("Replacing ~ with {} path", home_dir.to_string_lossy()))?);
            Ok(buf)
        },
    }
}
//-//////////////////////////////////////////////////////////////////
