use super::{
    install::{Installer, InstallerContext},
    rust_bin_installer::RustBinInstaller,
};
use crate::tool::{fs::AppDir, global_input::GlobalInput, kaleido};
use log::{error, warn};
use std::fs::{create_dir, remove_dir_all};

pub const LATEST_VERSION: &str = "LATEST";

#[derive(Clone, Debug)]
pub struct InstallRequest {
    pub name: String,
    pub alias: Option<String>,
    pub version: Option<String>,
}

pub async fn install<'a, 'b>(
    packages: Vec<InstallRequest>,
    app_dir: &AppDir,
    global_input: &'b mut GlobalInput<'a>,
    context: &InstallerContext,
) where
    'a: 'b,
{
    for request in packages {
        install_one(global_input, &request, app_dir, context).await;
    }
}

pub async fn install_one<'a, 'b>(
    global_input: &'b mut GlobalInput<'a>,
    request: &InstallRequest,
    app_dir: &AppDir,
    context: &InstallerContext,
) where
    'a: 'b,
{
    println!(
        "=================install package {}=================",
        request.name
    );
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
                RustBinInstaller::new(global_input)
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
