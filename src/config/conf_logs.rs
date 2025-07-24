use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use tracing::metadata::Level;

//-////////////////////////////////////////////////////////////////////////////
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
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct ConfLog {
    pub enable_logging: bool,
    pub log_path: PathBuf,
    pub log_level: ConfLogLevel,
    pub log_libraries: bool,
}

impl ConfLog {
    pub fn init() -> ConfLog {
        ConfLog{
            enable_logging: false,
            log_path: PathBuf::default(),
            log_level: ConfLogLevel::Info,
            log_libraries: false,
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
