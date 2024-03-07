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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileType {
    Compression,
    Plain,
    Unknown,
}

pub fn get_file_type<T: AsRef<Path>>(file_name: T) -> FileType {
    match file_name.as_ref().extension() {
        Some(ext) => match ext.to_str() {
            Some(e) => match e {
                "" => FileType::Plain,
                "zip" => FileType::Compression,
                "7z" => FileType::Compression,
                "gz" => FileType::Compression,
                _ => FileType::Unknown,
            },
            None => FileType::Unknown,
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
        Some(v) => Some(v.path()?.into_owned().clone()),
        None => None,
    };

    match top_folder {
        Some(p) => {
            let mut components = p.components();
            let top = match components.next() {
                Some(c) => {
                    if c == Component::RootDir
                        || c == Component::CurDir
                        || c == Component::ParentDir
                    {
                        components.next()
                    } else {
                        Some(c)
                    }
                }
                None => None,
            };
            Ok(top.map(|t| t.as_os_str().to_str().unwrap().to_string()))
        }
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
    let mut top_path: Option<PathBuf> = Option::None;
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
        let child_path = if (*item.name()).ends_with('/') {
            Some(outpath.as_path())
        } else {
            outpath.parent()
        };
        match &top_path {
            Some(top) => {
                if let Some(child) = child_path {
                    top_path = Some(find_common_parent(top, child, to_path));
                }
            }
            None => {
                top_path = child_path.map(|f| f.to_path_buf());
            }
        }

        if (*item.name()).ends_with('/') {
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
    if top_folder.is_none() && top_path.is_some() {
        if let Some(p) = top_path {
            if let Some(name) = p.file_name() {
                if let Some(s) = name.to_str() {
                    top_folder = Some(s.to_string());
                }
            }
        }
    }
    Ok(top_folder)
}

fn find_common_parent<T: AsRef<Path>, S: AsRef<Path>>(t: T, s: S, root: &Path) -> PathBuf {
    let t_path = t.as_ref();
    let s_path = s.as_ref();
    if t_path.starts_with(s_path) {
        s_path.to_path_buf()
    } else if s_path.starts_with(t_path) {
        t_path.to_path_buf()
    } else {
        let mut top_parents = vec![];
        let mut parent = t_path;
        top_parents.push(parent);
        while let Some(p) = parent.parent() {
            if root.starts_with(p) {
                break;
            }
            top_parents.insert(0, p);
            parent = p;
        }

        let mut child_parents = vec![];
        parent = s_path;
        child_parents.push(parent);
        while let Some(p) = parent.parent() {
            if root.starts_with(p) {
                break;
            }
            child_parents.insert(0, p);
            parent = p;
        }
        let mut i = 0;
        while i < top_parents.len() && i < child_parents.len() {
            if top_parents[i] != child_parents[i] {
                break;
            }
            i += 1;
        }
        let common_parent = if i == 0 { root } else { top_parents[i - 1] };
        common_parent.to_path_buf()
    }
}

#[cfg(target_os = "windows")]
pub fn make_file_link<T: AsRef<Path>, S: AsRef<Path>>(link_file: T, origin_file: S) -> Result<()> {
    use log::error;
    use std::fs::remove_file;
    use std::os::windows::fs::symlink_file;

    let link_path = link_file.as_ref();
    let origin_path = origin_file.as_ref();
    debug!("make file link from {:?} to {:?}", link_path, origin_path);
    if let Err(err) = symlink_file(origin_path, link_path) {
        debug!("failed to make symlink: {}, try cmd", err);
        if link_path.exists() {
            if link_path.is_symlink() {
                remove_link(link_path)?
            } else {
                remove_file(link_path)?;
            }
        }
        let status = std::process::Command::new("cmd.exe")
            .arg("/c")
            .arg("mklink")
            .arg("/h")
            .arg(link_path)
            .arg(origin_path)
            .stdout(std::process::Stdio::null())
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
pub fn make_dir_link<T: AsRef<Path>, S: AsRef<Path>>(link_dir: T, origin_dir: S) -> Result<()> {
    use log::error;
    use std::fs::remove_dir;
    use std::os::windows::fs::symlink_dir;

    let link_path = link_dir.as_ref();
    let origin_path = origin_dir.as_ref();
    debug!("make dir link from {:?} to {:?}", link_path, origin_path);
    if let Err(err) = symlink_dir(origin_path, link_path) {
        debug!("failed to make symlink: {}, try cmd", err);
        if link_dir.as_ref().exists() {
            remove_dir(link_path)?;
        }
        let status = std::process::Command::new("cmd.exe")
            .arg("/c")
            .arg("mklink")
            .arg("/j")
            .arg(link_path)
            .arg(origin_path)
            .stdout(std::process::Stdio::null())
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
pub fn make_link<T: AsRef<Path>, S: AsRef<Path>>(link_dir: T, origin_dir: S) -> Result<()> {
    if origin_dir.as_ref().is_dir() {
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

#[cfg(test)]
mod test {
    use super::*;
    use std::{
        fs::{create_dir, File},
        io::Write,
    };
    use tempfile::tempdir;

    #[test]
    fn test_get_file_type() {
        assert_eq!(FileType::Plain, get_file_type("path/test"));
        assert_eq!(FileType::Compression, get_file_type("path/test.zip"));
        assert_eq!(FileType::Compression, get_file_type("path/test.7z"));
        assert_eq!(FileType::Compression, get_file_type("path/test.tar.gz"));
        assert_eq!(FileType::Unknown, get_file_type("path/test.x"));
    }

    #[test]
    fn test_make_link_file() -> std::result::Result<(), FSError> {
        let dir = tempdir()?;
        let file_path = dir.path().join("origin");
        let mut file = File::create(&file_path)?;
        writeln!(file, "test")?;
        drop(file);

        let link_path = dir.path().join("link");
        assert!(!link_path.exists());
        make_link(&link_path, &file_path)?;
        assert!(link_path.exists());
        if cfg!(target_os = "windows") {
            assert!(link_path.is_file());
        } else {
            assert!(link_path.is_symlink());
        }

        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_make_link_dir() -> std::result::Result<(), FSError> {
        let dir = tempdir()?;
        let file_path = dir.path().join("origin_dir");
        create_dir(&file_path)?;

        let link_path = dir.path().join("link_dir");
        assert!(!link_path.exists());
        make_link(&link_path, &file_path)?;
        assert!(link_path.exists());
        assert!(link_path.is_symlink());

        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_find_common_parent() {
        let root = Path::new("C:\\Users\\kaleido\\.sys-kaleido\\tmp");
        let s = "C:\\Users\\kaleido\\.sys-kaleido\\tmp\\a";
        let t = "C:\\Users\\kaleido\\.sys-kaleido\\tmp\\a\\b";
        assert_eq!(find_common_parent(t, s, root), Path::new(s));
        let s = "C:\\Users\\kaleido\\.sys-kaleido\\tmp\\a\\b";
        let t = "C:\\Users\\kaleido\\.sys-kaleido\\tmp\\a\\b";
        assert_eq!(find_common_parent(t, s, root), Path::new(s));
        let s = "C:\\Users\\kaleido\\.sys-kaleido\\tmp\\b\\c";
        let t = "C:\\Users\\kaleido\\.sys-kaleido\\tmp\\a\\b";
        assert_eq!(find_common_parent(t, s, root), root);
    }
}
