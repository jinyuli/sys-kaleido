use super::install::{AppRelease, InstallError};
use crate::tool::{
    fs::{decompress, make_link, remove_link, AppDir},
    kaleido::Package,
    http::download_with_progress,
};
use log::{debug, error};
use std::env::consts::EXE_EXTENSION;
use std::fs::{copy, create_dir_all, remove_dir_all, remove_file};

pub struct RustSrcInstaller {}

impl RustSrcInstaller {
    pub async fn install_package(&self, app_release: &AppRelease, package: &Package, app_dir: &AppDir) -> std::result::Result<(), InstallError> {
        let source_url = app_release.source_url.as_ref().unwrap();
        let tmp_dir = app_dir.get_home_dir().join("tmp");
        let file_name = format!("{}.zip", app_release.version);
        let to_file = tmp_dir.join(&file_name);
        debug!("download {} to {:?}", source_url, to_file);
        if to_file.exists() && to_file.is_file() {
            remove_file(&to_file)?;
            // if let Err(e) = remove_file(&to_file) {
            //     error!("failed to delete old file({:?}): {}", to_file, e);
            //     return;
            // }
        }

        let src_path = tmp_dir.join(&package.name);
        if src_path.exists() && src_path.is_dir() {
            remove_dir_all(&src_path)?;
            // if let Err(e) = remove_dir_all(&src_path) {
            //     error!("failed to delete dir({:?}): {}", src_path, e);
            //     return;
            // }
        }
        create_dir_all(&src_path)?;
        // if let Err(e) = create_dir_all(&src_path) {
        //     error!("failed to create src dir({:?}): {}", src_path, e);
        //     return;
        // }

        download_with_progress(to_file, source_url).await?;
        // if let Err(e) = download_with_progress(to_file, source_url).await {
        //     error!("failed to download file: {}", e);
        //     return;
        // }


        let to_file = tmp_dir.join(&file_name);
        let top_folder = match decompress(&to_file, &src_path)? {
            Some(f) => f,
            None => String::new(),
        };
        // let top_folder = match decompress(&to_file, &src_path) {
        //     Ok(f) => match f {
        //         Some(folder) => folder,
        //         None => String::new(),
        //     },
        //     Err(e) => {
        //         error!("failed to decompress file: {}", e);
        //         return;
        //     }
        // };

        let src_folder = src_path.join(top_folder);
        let build_result = std::process::Command::new("cargo")
            .arg("build")
            .arg("--workspace")
            .arg("--release")
            .current_dir(&src_folder)
            .status();
        match build_result {
            Ok(status) => {
                if !status.success() {
                    error!("failed to build {} from source code", package.name);
                    return Err(InstallError::GeneralStr("failed to build from source code"));
                }
                let package_dir = app_dir
                    .get_packages_dir()
                    .join(&package.name)
                    .join(&app_release.version);
                if package_dir.exists() && package_dir.is_dir() {
                    remove_dir_all(&package_dir)?;
                    // if let Err(e) = remove_dir_all(&package_dir) {
                    //     error!("failed to delete dir({:?}): {}", package_dir, e);
                    //     return;
                    // }
                }
                create_dir_all(&package_dir)?;
                // if let Err(e) = create_dir_all(&package_dir) {
                //     error!("failed to create dir({:?}): {}", package_dir, e);
                //     return;
                // }

                let mut package_bin_file = package_dir.join(&package.bin_name);
                package_bin_file.set_extension(EXE_EXTENSION);
                let mut bin_file = src_folder
                    .join("target")
                    .join("release")
                    .join(&package.bin_name);
                bin_file.set_extension(EXE_EXTENSION);
                copy(&bin_file, &package_bin_file)?;
                // if let Err(e) = copy(&bin_file, &package_bin_file) {
                //     error!("failed to copy file from {:?} to {:?}: {}", bin_file, package_bin_file, e);
                //     return;
                // }
                let mut sys_bin_file = app_dir.get_bin_dir().join(&package.bin_name);
                sys_bin_file.set_extension(EXE_EXTENSION);
                if sys_bin_file.exists() && sys_bin_file.is_file() {
                    remove_link(&sys_bin_file)?;
                    // if let Err(e) = remove_link(&sys_bin_file) {
                    //     error!("failed to remove link file({:?}): {}", &sys_bin_file, e);
                    //     return;
                    // }
                }

                make_link(&sys_bin_file, &package_bin_file)?;
                // if let Err(e) = make_link(&sys_bin_file, &package_bin_file) {
                //     error!("failed to make link from {:?} to {:?} : {}", sys_bin_file, package_bin_file, e);
                //     return;
                // }

                let _ = remove_dir_all(tmp_dir);
            }
            Err(e) => {
                error!("failed to compile the source: {}", e);
            }
        }

        Ok(())
    }
}
