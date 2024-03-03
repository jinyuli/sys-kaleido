use crate::tool::{http::download_file, kaleido::KALEIDO_SYS_FILE};
use colored::Colorize;
use log::error;
use std::{
    fs::remove_file,
    io::{ErrorKind, Read},
    path::Path,
    time::{Duration, SystemTime},
};
use tokio::fs::{metadata, rename};

const CONFIG_URL: &str =
    "https://raw.githubusercontent.com/jinyuli/sys-kaleido/master/kaleido.toml";
const DAYS_7: u64 = 7 * 24 * 60 * 60;

pub async fn update(home_dir: &Path) {
    let tmp_config_path = home_dir.join(format!("{}.tmp", KALEIDO_SYS_FILE));
    match download_file(CONFIG_URL, &tmp_config_path).await {
        Ok(_) => {
            let config_path = home_dir.join(KALEIDO_SYS_FILE);
            if config_path.exists() && config_path.is_file() {
                if let Err(e) = remove_file(&config_path) {
                    error!("failed to delete old config file: {}", e);
                    return;
                }
            }
            if let Err(e) = rename(&tmp_config_path, &config_path).await {
                error!(
                    "failed to rename new config file(from {:?} to {:?}): {}",
                    tmp_config_path, config_path, e
                );
                return;
            }
            println!("{}", "updated configuration file successfully".green());
        }
        Err(e) => {
            error!("failed to download config file: {}", e);
        }
    }
}

pub async fn check_config(home_dir: &Path) {
    let config_path = home_dir.join(format!("{}.tmp", KALEIDO_SYS_FILE));
    if !config_path.exists() || !config_path.is_file() {
        update(home_dir).await;
    } else {
        match metadata(&config_path).await {
            Ok(m) => match m.modified() {
                Ok(t) => match SystemTime::now().duration_since(t) {
                    Ok(d) => {
                        if d > Duration::from_secs(DAYS_7) {
                            print!("It's been over 7 days since you update coniguration file, would you like to update it now? [y/n]");
                            let mut buffer = [0; 1];
                            loop {
                                match std::io::stdin().read_exact(&mut buffer) {
                                    Ok(_) => break,
                                    Err(e) if e.kind() != ErrorKind::Interrupted => continue,
                                    Err(e) => {
                                        error!("failed to read from command: {}", e);
                                        return;
                                    }
                                }
                            }
                            let answer = char::from(buffer[0]);
                            if answer == 'y' || answer == 'Y' {
                                update(home_dir).await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("failed to get file time: {}", e);
                    }
                },
                Err(e) => {
                    error!("failed to get file modified time: {}", e);
                }
            },
            Err(e) => {
                error!("failed to get file metadata: {}", e);
            }
        }
    }
}
