use std::{fs::remove_dir_all, path::PathBuf};

use tokio::fs::read_dir;

use crate::tool::{
    fs::{remove_link, AppDir},
    logger::error,
};

pub async fn uninstall(package_names: Vec<String>, app_dir: &AppDir) {
    let package_paths: Vec<PathBuf> = package_names
        .iter()
        .map(|n| app_dir.get_packages_dir().join(n))
        .filter(|p| p.exists() && p.is_dir())
        .collect();
    remove_all_links(&package_paths, app_dir).await;
    for package_path in package_paths {
        if let Err(e) = remove_dir_all(&package_path) {
            error!("failed to remove dir({:?}), error: {}", package_path, e);
        }
    }
}

#[cfg(target_os = "windows")]
async fn remove_all_links(target_packages: &Vec<PathBuf>, app_dir: &AppDir) {
    use super::helper::get_hardlinks;

    for dir in [app_dir.get_bin_dir(), app_dir.get_alias_dir()] {
        match read_dir(dir).await {
            Ok(mut entries) => loop {
                match entries.next_entry().await {
                    Ok(e) => {
                        if let Some(entry) = e {
                            let file_path = entry.path();
                            if let Ok(links) = get_hardlinks(&file_path, dir) {
                                let mut is_desired = false;
                                'out_search: for link in &links {
                                    if link.starts_with(app_dir.get_packages_dir()) {
                                        for package_path in target_packages {
                                            if link.starts_with(package_path) {
                                                is_desired = true;
                                                break 'out_search;
                                            }
                                        }
                                    }
                                }
                                if is_desired {
                                    for link in &links {
                                        if link.starts_with(app_dir.get_bin_dir())
                                            || link.starts_with(app_dir.get_alias_dir())
                                        {
                                            if let Err(e) = remove_link(link) {
                                                error!(
                                                    "failed to remove link({:?}), error: {}",
                                                    link, e
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("failed to list directory: {}", e);
                        break;
                    }
                }
            },
            Err(e) => {
                error!("failed to list directory: {}", e);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
async fn remove_all_links(target_packages: &Vec<PathBuf>, app_dir: &AppDir) {
    use crate::tool::logger::debug;
    use tokio::fs::read_link;
    for dir in [app_dir.get_bin_dir(), app_dir.get_alias_dir()] {
        match read_dir(dir).await {
            Ok(mut entries) => loop {
                match entries.next_entry().await {
                    Ok(e) => {
                        if let Some(entry) = e {
                            let file_path = entry.path();
                            if file_path.is_symlink() {
                                match read_link(file_path).await {
                                    Ok(p) => {
                                        debug!("\tlink: {:?}", p);
                                        if p.starts_with(app_dir.get_packages_dir()) {
                                            for package_path in target_packages {
                                                if p.starts_with(package_path) {
                                                    if let Err(e) = remove_link(&p) {
                                                        error!("failed to remove link({:?}), error: {}", p, e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("failed to list directory: {}", e);
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("failed to list directory: {}", e);
                        break;
                    }
                }
            },
            Err(e) => {
                error!("failed to list directory: {}", e);
            }
        }
    }
}
