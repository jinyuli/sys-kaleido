use colored::Colorize;

use super::helper::get_installed_packages;
use crate::tool::{fs::AppDir, logger::error, kaleido};

pub async fn list(app_dir: &AppDir) {
    let installed_packages = match get_installed_packages(app_dir).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to get installed packages: {}", e);
            return;
        }
    };

    for package in installed_packages {
        println!(
            "* {} - {}",
            package.package.name.green(),
            package.version.green()
        );
    }
}

pub async fn list_all(app_dir: &AppDir) {
    let kaleido = match kaleido::load_config(app_dir.get_home_dir()) {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to load metadata: {}", e);
            return;
        }
    };
    let installed_packages = match get_installed_packages(app_dir).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to get installed packages: {}", e);
            return;
        }
    };

    for package in kaleido.packages {
        match installed_packages.iter().find(|p| p.package.name == package.name) {
            Some(ip) => {
                print!(
                    "* {} ({})",
                    package.name.green(),
                    ip.version.green()
                );
            }
            None => {
                print!("  {}", package.name);
            }
        }
        if let Some(desc) = package.description {
            if !desc.is_empty() {
                print!(" - {}", desc);
            }
            print!(" [{}]", package.url);
        } else if !package.url.is_empty() {
            print!(" - [{}]", package.url);
        }
        println!()
    }
}
