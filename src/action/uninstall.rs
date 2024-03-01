use std::{
    fs::remove_dir_all,
    path::{Path, PathBuf},
};

use tokio::fs::{read_dir, read_link};

use crate::tool::{
    fs::{remove_link, AppDir},
    logger::{debug, error},
};

pub async fn uninstall(package_names: Vec<String>, app_dir: &AppDir) {
    let package_paths: Vec<PathBuf> = package_names
        .iter()
        .map(|n| app_dir.get_packages_dir().join(n))
        .filter(|p| p.exists() && p.is_dir())
        .collect();
    remove_linked_files(
        app_dir.get_bin_dir(),
        app_dir.get_packages_dir(),
        &package_paths,
    )
    .await;
    remove_linked_files(
        app_dir.get_alias_dir(),
        app_dir.get_packages_dir(),
        &package_paths,
    )
    .await;
    for package_path in package_paths {
        if let Err(e) = remove_dir_all(&package_path) {
            error!("failed to remove dir({:?}), error: {}", package_path, e);
        }
    }
}

async fn remove_linked_files(
    linked_files_in_dir: &Path,
    packages_dir: &Path,
    target_packages: &Vec<PathBuf>,
) {
    match read_dir(linked_files_in_dir).await {
        Ok(mut entries) => {
            loop {
                match entries.next_entry().await {
                    Ok(e) => {
                        if let Some(entry) = e {
                            let file_path = entry.path();
                            if file_path.is_symlink() {
                                match read_link(file_path).await {
                                    Ok(p) => {
                                        debug!("\tlink: {:?}", p);
                                        if p.starts_with(packages_dir) {
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
            }
        }
        Err(e) => {
            error!("failed to list directory: {}", e);
        }
    }
}
