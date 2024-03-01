use log::error;
use serde::{self, Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

pub const KALEIDO_SYS_FILE: &str = "kaleido.toml";
pub const KALEIDO_CUSTOM_FILE: &str = "custom.toml";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Kaleido {
    pub packages: Vec<Package>,
    pub bindles: Vec<Bindle>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Bindle {
    pub name: String,
    pub packages: Vec<BindlePackage>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BindlePackage {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Package {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub replace: Option<String>,
    pub language: Option<String>,
    pub bin_name: String,
    pub bin_path: Option<String>,
    /// by default, sys-kaleido tries to find download fiel by 'arch', 'os', and maybe abi(for rust only)
    /// if this is specified, it will be also used when searching remote files.
    pub remote_file_prefix: Option<String>,
    pub github: Option<Github>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Github {
    pub org: String,
    pub repo: String,
}

type Result<T> = std::result::Result<T, ConfigError>;

impl Kaleido {
    pub fn load_config(home_dir: &Path) -> Result<Self> {
        let config_path = home_dir.join(KALEIDO_SYS_FILE);
        let sys = if !config_path.exists() || !config_path.is_file() {
            return Err(ConfigError::NoSuchFile());
        } else {
            let mut config_file = fs::File::open(config_path)?;
            let mut content = String::new();
            config_file.read_to_string(&mut content)?;
            let config: Kaleido = toml::from_str(&content)?;
            config
        };

        let custom_config_path = home_dir.join(KALEIDO_CUSTOM_FILE);
        if custom_config_path.exists() && custom_config_path.is_file() {
            let custom = {
                let mut config_file = fs::File::open(custom_config_path)?;
                let mut content = String::new();
                config_file.read_to_string(&mut content)?;
                let config: Kaleido = toml::from_str(&content)?;
                config
            };

            Ok(Kaleido {
                packages: [sys.packages, custom.packages].concat(),
                bindles: [sys.bindles, custom.bindles].concat(),
            })
        } else {
            Ok(sys)
        }
    }
}

pub fn load_config(home_dir: &Path) -> Result<Kaleido> {
    Kaleido::load_config(home_dir)
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("file system error: {0}")]
    Fs(#[from] std::io::Error),
    #[error("config file error: {0}")]
    ConfigRead(#[from] toml::de::Error),
    #[error("config file error: {0}")]
    ConfigConvert(#[from] toml::ser::Error),
    #[error("cannot find kaleido file")]
    NoSuchFile(),
}
