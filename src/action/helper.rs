use std::{
    collections::HashMap,
    io::{self, BufRead, Cursor, Error, Result},
    path::{Path, PathBuf},
    str,
};

use tokio::fs::read_dir;

use crate::tool::{
    fs::AppDir,
    kaleido::{self, Package},
    logger::{debug, error},
};

#[derive(Debug, Clone)]
pub struct InstalledPackage {
    pub package: Package,
    pub version: String,
}

pub async fn get_installed_packages(app_dir: &AppDir) -> Result<Vec<InstalledPackage>> {
    let kaleido = match kaleido::load_config(app_dir.get_home_dir()) {
        Ok(k) => k,
        Err(e) => {
            // error!("Failed to load metadata: {}", e);
            return Err(Error::other(e));
        }
    };

    let package_map: HashMap<String, &Package> = kaleido
        .packages
        .iter()
        .map(|p| (p.name.clone(), p))
        .collect();
    let mut packages: Vec<InstalledPackage> = vec![];

    match read_dir(app_dir.get_bin_dir()).await {
        Ok(mut entries) => {
            loop {
                match entries.next_entry().await {
                    Ok(e) => {
                        if let Some(entry) = e {
                            let file_path = entry.path();
                            match get_installed_package(
                                &file_path,
                                app_dir.get_packages_dir(),
                                &package_map,
                            )
                            .await
                            {
                                Ok(o) => {
                                    if let Some(ip) = o {
                                        packages.push(ip);
                                    }
                                }
                                Err(e) => {
                                    // ignore the error for a single file
                                    debug!("failed to list directory: {}", e);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("failed to get a directory: {}", e);
                        break;
                    }
                }
            }
            Ok(packages)
        }
        Err(e) => {
            // error!("failed to list directory: {}", e);
            Err(Error::other(e))
        }
    }
}

fn get_file_name(p: &Path) -> Option<String> {
    if let Some(f) = p.file_name() {
        if let Some(s) = f.to_str() {
            return Some(s.to_string());
        }
    }
    None
}

#[cfg(target_os = "windows")]
async fn get_installed_package(
    file: &Path,
    dir: &Path,
    package_map: &HashMap<String, &Package>,
) -> std::io::Result<Option<InstalledPackage>> {
    if file.exists() {
        let hardlinkes = get_hardlinks(file, dir)?;
        if hardlinkes.len() < 2 {
            return Err(std::io::Error::other(format!(
                "the given file {:?} is not a hardlink",
                file
            )));
        }
        'outloop: for p in hardlinkes {
            if p.starts_with(dir) {
                let ancestors = p.ancestors();
                let mut child = p.as_path();
                for a in ancestors {
                    if a == dir {
                        if let Some(file_name) = get_file_name(child) {
                            if let Some(package) = package_map.get(&file_name) {
                                if let Some(v) = get_file_name(
                                    p.parent().expect("linked file point to nowhere?"),
                                ) {
                                    return Ok(Some(InstalledPackage {
                                        package: (*package).clone(),
                                        version: v,
                                    }));
                                }
                            }
                        }
                        break 'outloop;
                    } else {
                        child = a;
                    }
                }
            }
        }
    }
    Ok(None)
}

#[cfg(target_os = "windows")]
pub fn get_hardlinks(file: &Path, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    debug!("get hardlinks: {:?}", file);
    let r = std::process::Command::new("cmd.exe")
        .arg("/c")
        .arg("fsutil")
        .arg("hardlink")
        .arg("list")
        .arg(file)
        .output();
    match r {
        Ok(output) => {
            if !output.status.success() {
                error!(
                    "get_hardlinks failed, status: {:?}, out: {:?}, err: {:?}",
                    output.status.code(),
                    str::from_utf8(&output.stdout),
                    str::from_utf8(&output.stderr)
                );
                Err(io::Error::other(""))
            } else {
                let mut lines = vec![];
                let mut cursor = Cursor::new(output.stdout);
                let mut line = String::new();
                loop {
                    match cursor.read_line(&mut line) {
                        Ok(l) => {
                            if l == 0 {
                                break;
                            }

                            let clone = line.clone();
                            lines.push(clone);
                        }
                        Err(e) => {
                            error!("error when reading lines from fsutil: {}", e);
                        }
                    }
                    line.clear();
                }
                let prefix = dir.components().next();
                if let Some(p) = prefix {
                    if let std::path::Component::Prefix(_drive) = p {
                        let paths: Vec<_> = lines
                            .iter()
                            .map(|l| {
                                let trimed = l.trim_matches(&['\r', '\n']);
                                let path = Path::new(trimed);
                                if path.is_relative() {
                                    let mut pb = PathBuf::new();
                                    pb.push(p);
                                    pb.push(trimed);
                                    pb
                                } else {
                                    path.to_path_buf()
                                }
                            })
                            .collect();
                        return Ok(paths);
                    }
                }
                let paths: Vec<_> = lines.iter().map(|l| Path::new(&l).to_path_buf()).collect();
                Ok(paths)
            }
        }
        Err(e) => {
            error!("get_hardlinks error: {}", e);
            Err(io::Error::other(e))
        }
    }
}

#[cfg(not(target_os = "windows"))]
async fn get_installed_package(
    file: &Path,
    dir: &Path,
    package_map: &HashMap<String, &Package>,
) -> std::io::Result<Option<InstalledPackage>> {
    use tokio::fs::read_link;
    if file.exists() && file.is_symlink() {
        let p = read_link(file).await?;
        debug!("\tlink: {:?}", p);
        if p.starts_with(dir) {
            let ancestors = p.ancestors();
            let mut child = p.as_path();
            for a in ancestors {
                debug!("\t\tancestor: {:?}", a);
                if a == dir {
                    if let Some(file_name) = get_file_name(child) {
                        if let Some(package) = package_map.get(&file_name) {
                            if let Some(v) =
                                get_file_name(p.parent().expect("linked file point to nowhere?"))
                            {
                                return Ok(Some(InstalledPackage {
                                    package: (*package).clone(),
                                    version: v,
                                }));
                            }
                        }
                    }
                    break;
                } else {
                    child = a;
                }
            }
        }
    }
    Ok(None)
}
