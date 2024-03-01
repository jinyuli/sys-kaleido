use super::logger::debug;
use directories::UserDirs;
use flate2::read::GzDecoder;
use sevenz_rust::default_entry_extract_fn;
use std::fs;
use std::io;
use std::path::Component;
use std::path::{Path, PathBuf};
use tar::Archive;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, FSError>;

#[derive(Clone, Debug)]
pub struct AppDir {
    home_dir: PathBuf,
    log_dir: PathBuf,
    bin_dir: PathBuf,
    packages_dir: PathBuf,
    alias_dir: PathBuf,
}

impl AppDir {
    pub fn new() -> Result<Self> {
        if let Some(user_dir) = UserDirs::new() {
            let user_home = user_dir.home_dir();
            let home = user_home.join(".sys-kaleido");
            if !home.exists() || !home.is_dir() {
                fs::create_dir(&home)?;
            }

            let packages_dir = home.join("packages");
            if !packages_dir.exists() || !packages_dir.is_dir() {
                fs::create_dir(&packages_dir)?;
            }

            let alias_dir = home.join("alias");
            if !alias_dir.exists() || !alias_dir.is_dir() {
                fs::create_dir(&alias_dir)?;
            }

            let bin_dir = home.join("bin");
            if !bin_dir.exists() || !bin_dir.is_dir() {
                fs::create_dir(&bin_dir)?;
            }

            let log_dir = home.join("log");
            if !log_dir.exists() || !log_dir.is_dir() {
                fs::create_dir(&log_dir)?;
            }


            Ok(AppDir {
                home_dir: home,
                log_dir,
                packages_dir,
                alias_dir,
                bin_dir,
            })
        } else {
            Err(FSError::AppDirError())
        }
    }

    pub fn get_home_dir(&self) -> &Path {
        self.home_dir.as_path()
    }

    pub fn get_log_dir(&self) -> &Path {
        self.log_dir.as_path()
    }

    pub fn get_packages_dir(&self) -> &Path {
        self.packages_dir.as_path()
    }

    pub fn get_alias_dir(&self) -> &Path {
        self.alias_dir.as_path()
    }

    pub fn get_bin_dir(&self) -> &Path {
        self.bin_dir.as_path()
    }
}

#[derive(Clone, Debug)]
pub enum FileType {
    Compression,
    Plain,
    Unknown,
}

pub fn get_file_type<T: AsRef<Path>>(file_name: T) -> FileType {
    match file_name.as_ref().extension() {
        Some(ext) => {
            match ext.to_str() {
                Some(e) => {
                    match e {
                        "" => FileType::Plain,
                        "zip" => FileType::Compression,
                        "7z" => FileType::Compression,
                        "gz" => FileType::Compression,
                        _ => FileType::Unknown,
                    }
                },
                None => FileType::Unknown,
            }
        },
        None => FileType::Plain,
    }
}

pub fn decompress(file_path: &Path, to_path: &Path) -> Result<Option<String>> {
    debug!("decompress from {:?} to {:?}", file_path, to_path);
    let folder = match file_path.extension() {
        Some(ext) => {
            if ext == "zip" {
                debug!("use unzip");
                unzip(file_path, to_path)?
            } else if ext == "7z" {
                debug!("use sevenz");
                sevenz(file_path, to_path)?
            } else {
                debug!("use deflate");
                deflate(file_path, to_path)?
            }
        }
        None => return Err(FSError::UnsupportedFile("")),
    };

    let result = match folder {
        Some(f) => {
            if f.ends_with('/') {
                Some(f[0..(f.len() - 1)].to_string())
            } else {
                Some(f)
            }
        }
        None => None,
    };

    Ok(result)
}

fn deflate(file_path: &Path, to_path: &Path) -> Result<Option<String>> {
    let tar_gz = fs::File::open(file_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(to_path)?;

    let tar_gz = fs::File::open(file_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    let top_folder: Option<PathBuf> = match archive.entries()?.filter_map(|e| e.ok()).next() {
        Some(v) => {
            Some(v.path()?.into_owned().clone())
        },
        None => None,
    };

    match top_folder {
        Some(p) => {
            let mut components = p.components();
            let top = match components.next() {
                Some(c) => {
                    if c == Component::RootDir || c == Component::CurDir || c == Component::ParentDir {
                        components.next()
                    } else {
                        Some(c)
                    }
                },
                None => None,
            };
            Ok(top.map(|t| t.as_os_str().to_str().unwrap().to_string()))
        },
        None => Ok(None),
    }
}

fn sevenz(file_path: &Path, to_path: &Path) -> Result<Option<String>> {
    let mut top_folder: Option<PathBuf> = Option::None;
    sevenz_rust::decompress_file_with_extract_fn(file_path, to_path, |entry, reader, dest| {
        if entry.is_directory()
            && (top_folder.is_none() || top_folder.as_ref().is_some_and(|f| f.starts_with(dest)))
        {
            top_folder = Some(dest.clone());
        }
        default_entry_extract_fn(entry, reader, dest)
    })?;
    match top_folder {
        Some(folder) => match folder.file_name() {
            Some(s) => match s.to_str() {
                Some(v) => Ok(Some(v.to_string())),
                None => Ok(None),
            },
            None => Ok(None),
        },
        None => Ok(None),
    }
}

fn unzip(file_path: &Path, to_path: &Path) -> Result<Option<String>> {
    let mut top_folder = Option::None;
    let file = fs::File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut item = archive.by_index(i)?;
        let outpath = match item.enclosed_name() {
            Some(p) => to_path.join(p),
            None => continue,
        };
        {
            let comment = item.comment();
            if !comment.is_empty() {
                println!("file {i} comment: {comment}");
            }
        }
        if (*item.name()).ends_with('/') {
            if top_folder.is_none()
                || top_folder
                    .as_ref()
                    .is_some_and(|f: &String| f.len() > item.name().len())
            {
                top_folder = Some(item.name().to_string());
            }
            fs::create_dir_all(&outpath)?
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut item, &mut outfile)?;
        }
    }
    Ok(top_folder)
}

#[cfg(target_os = "windows")]
pub fn make_file_link(link_file: &Path, origin_file: &Path) -> Result<()> {
    use log::error;
    use std::fs::remove_file;
    use std::os::windows::fs::symlink_file;

    debug!("make file link from {:?} to {:?}", link_file, origin_file);
    if let Err(err) = symlink_file(origin_file, link_file) {
        error!("failed to make symlink: {}, try cmd", err);
        if link_file.exists() {
            if link_file.is_symlink() {
                remove_link(link_file)?
            } else {
                remove_file(link_file)?;
            }
        }
        let status = std::process::Command::new("cmd.exe")
            .arg("/c")
            .arg("mklink")
            .arg("/h")
            .arg(link_file)
            .arg(origin_file)
            .status()?;
        if !status.success() {
            error!("cmd mklink failed, status: {:?}", status.code());
            Err(FSError::LinkError())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub fn make_dir_link(link_dir: &Path, origin_dir: &Path) -> Result<()> {
    use log::error;
    use std::fs::remove_dir;
    use std::os::windows::fs::symlink_dir;

    debug!("make dir link from {:?} to {:?}", link_dir, origin_dir);
    if let Err(err) = symlink_dir(origin_dir, link_dir) {
        error!("failed to make symlink: {}, try cmd", err);
        if link_dir.exists() {
            remove_dir(link_dir)?;
        }
        let status = std::process::Command::new("cmd.exe")
            .arg("/c")
            .arg("mklink")
            .arg("/j")
            .arg(link_dir)
            .arg(origin_dir)
            .status()?;
        if !status.success() {
            error!("cmd mklink failed, status: {:?}", status.code());
            Err(FSError::LinkError())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub fn make_link(link_dir: &Path, origin_dir: &Path) -> Result<()> {
    if origin_dir.is_dir() {
        make_dir_link(link_dir, origin_dir)
    } else {
        make_file_link(link_dir, origin_dir)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn make_link(from_dir: &Path, to_dir: &Path) -> Result<()> {
    use std::os::unix::fs::symlink;
    symlink(to_dir, from_dir)?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn remove_link(dir: &Path) -> Result<()> {
    use std::fs::{remove_dir, remove_file};
    if dir.is_dir() {
        remove_dir(dir)?;
    } else {
        remove_file(dir)?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn remove_link(dir: &Path) -> Result<()> {
    use std::fs::remove_file;
    remove_file(dir)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum FSError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Cannot find user home")]
    AppDirError(),
    #[error("{0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("{0}")]
    SevenZ(#[from] sevenz_rust::Error),
    #[error("make symbol link error")]
    LinkError(),
    #[error("file type '{0}' is not supported")]
    UnsupportedFile(&'static str),
}

