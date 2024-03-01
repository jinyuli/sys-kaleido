use super::{
    install::{Installer, InstallerContext},
    rust_bin_installer::RustBinInstaller,
};
use crate::tool::{fs::AppDir, kaleido};
use log::{error, warn};
use std::{
    fs::{create_dir, remove_dir_all},
    io::Stdin,
};

pub const LATEST_VERSION: &str = "LATEST";

#[derive(Clone, Debug)]
pub struct InstallRequest {
    pub name: String,
    pub alias: Option<String>,
    pub version: Option<String>,
}

pub async fn install(packages: Vec<InstallRequest>, app_dir: &AppDir, context: &InstallerContext) {

    let mut stdin = std::io::stdin();
    for request in packages {
        install_one(&mut stdin, &request, app_dir, context).await;
    }
}

pub async fn install_one(stdin: &mut Stdin, request: &InstallRequest, app_dir: &AppDir, context: &InstallerContext) {
    println!("=================install package {}=================", request.name);
    let home_dir = app_dir.get_home_dir();
    let kaleido = match kaleido::load_config(home_dir) {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to load metadata: {}", e);
            return;
        }
    };

    let package = match kaleido.packages.iter().find(|p| p.name == request.name) {
        Some(p) => p,
        None => {
            error!("cannot find package with name {}", request.name);
            return;
        }
    };

    let tmp_dir = home_dir.join("tmp");
    if tmp_dir.exists() && tmp_dir.is_dir() {
        if let Err(e) = remove_dir_all(&tmp_dir) {
            error!("failed to remove dir: {}", e);
            return;
        }
    }
    if let Err(e) = create_dir(&tmp_dir) {
        error!("failed to create dir: {}", e);
        return;
    }

    match &package.language {
        Some(lang) => match lang.to_lowercase().as_str() {
            "rust" => {
                RustBinInstaller::new(stdin)
                    .install(package, request, app_dir, context)
                    .await;
            }
            _ => {
                warn!("unsupported language: {}", lang);
            }
        },
        None => {}
    }
}
