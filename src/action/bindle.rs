use colored::Colorize;

use crate::tool::{fs::AppDir, global_input::GlobalInput, kaleido, logger::error};

use super::helper::get_installed_packages;
use super::installer::{install as install_packages, InstallRequest, InstallerContext};
use super::uninstall::uninstall as uninstall_packages;

pub async fn list(name: Option<String>, app_dir: &AppDir) {
    match name {
        Some(n) => list_packages(n, app_dir).await,
        None => list_bindles(app_dir).await,
    }
}

pub async fn list_bindles(app_dir: &AppDir) {
    let kaleido = match kaleido::load_config(app_dir.get_home_dir()) {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to load metadata: {}", e);
            return;
        }
    };

    kaleido.bindles.iter().for_each(|b| {
        println!("  {}", b.name);
    })
}

pub async fn list_packages(name: String, app_dir: &AppDir) {
    let (requests, missing) = match get_packages(&name, app_dir) {
        Some(v) => v,
        None => return,
    };

    let installed_packages = match get_installed_packages(app_dir).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to get installed packages: {}", e);
            return;
        }
    };

    for request in requests {
        match installed_packages
            .iter()
            .find(|ip| ip.package.name == request.name)
        {
            Some(installed_package) => {
                println!(
                    "* {} - {}",
                    request.name.green(),
                    installed_package.version.green()
                );
            }
            None => {
                println!("  {}", request.name);
            }
        }
    }

    if !missing.is_empty() {
        println!();
        println!("cannot find the following packages:");
        for name in missing {
            println!("{}", name.red());
        }
    }
}

pub async fn install(
    name: String,
    app_dir: &AppDir,
    global_input: &mut GlobalInput<'_>,
    context: &InstallerContext,
) {
    let (requests, missing) = match get_packages(&name, app_dir) {
        Some(v) => v,
        None => return,
    };
    install_packages(requests, app_dir, global_input, context).await;

    if !missing.is_empty() {
        println!();
        println!("cannot find the following packages:");
        for name in missing {
            println!("{}", name.red());
        }
    }
}

pub async fn uninstall(name: String, app_dir: &AppDir) {
    let (requests, missing) = match get_packages(&name, app_dir) {
        Some(v) => v,
        None => return,
    };

    let package_names = requests.into_iter().map(|r| r.name).collect();
    uninstall_packages(package_names, app_dir).await;

    if !missing.is_empty() {
        println!();
        println!("cannot find the following packages:");
        for name in missing {
            println!("{}", name.red());
        }
    }
}

pub async fn update(
    name: String,
    app_dir: &AppDir,
    global_input: &mut GlobalInput<'_>,
    context: &InstallerContext,
) {
    install(name, app_dir, global_input, context).await;
}

fn get_packages(name: &str, app_dir: &AppDir) -> Option<(Vec<InstallRequest>, Vec<String>)> {
    let kaleido = match kaleido::load_config(app_dir.get_home_dir()) {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to load metadata: {}", e);
            return None;
        }
    };
    let bindle = match kaleido.bindles.iter().find(|b| b.name == name) {
        Some(b) => b,
        None => {
            error!("Cannot find bindle with name {}", name);
            return None;
        }
    };

    let mut requests = vec![];
    let mut missed_packages = vec![];
    bindle.packages.iter().for_each(|bindle_package| {
        if let Some(p) = kaleido
            .packages
            .iter()
            .find(|package| package.name == bindle_package.name)
        {
            requests.push(InstallRequest {
                name: p.name.clone(),
                version: None,
                alias: bindle_package.alias.clone(),
            })
        } else {
            missed_packages.push(bindle_package.name.clone());
        }
    });

    Some((requests, missed_packages))
}
