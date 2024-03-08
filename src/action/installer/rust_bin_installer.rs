use super::{
    execute::{InstallRequest, LATEST_VERSION},
    install::{find_assets, AppRelease, InstallError, Installer, InstallerContext},
    rust_src_installer::RustSrcInstaller,
};
use crate::tool::{
    fs::{decompress, get_file_type, make_link, remove_link, AppDir, FileType},
    global_input::GlobalInput,
    http::download_with_progress,
    kaleido::Package,
};
use colored::Colorize;
use log::{debug, error};
use std::fs::{copy, create_dir_all, remove_dir_all, remove_file};
use std::{env::consts::EXE_EXTENSION, io::Write};

pub struct RustBinInstaller<'a, 'b> {
    global_input: &'b mut GlobalInput<'a>,
}

impl<'a, 'b> RustBinInstaller<'a, 'b> {
    pub fn new(global_input: &'b mut GlobalInput<'a>) -> Self {
        RustBinInstaller { global_input }
    }

    async fn install_package(
        &self,
        release: &AppRelease,
        package: &Package,
        alias: &Option<String>,
        app_dir: &AppDir,
        force_install: bool,
    ) -> std::result::Result<(), InstallError> {
        let asset = &release.assets[0];
        let tmp_dir = app_dir.get_home_dir().join("tmp");
        let to_file = tmp_dir.join(&asset.name);
        if to_file.exists() && to_file.is_file() {
            remove_file(&to_file)?;
            // if let Err(e) = remove_file(&to_file) {
            //     error!("failed to delete file({:?}): {}", to_file, e);
            //     return;
            // }
        }

        let package_dir = app_dir
            .get_packages_dir()
            .join(&package.name)
            .join(&release.version);

        let mut package_bin_file = package_dir.join(&package.bin_name);
        package_bin_file.set_extension(EXE_EXTENSION);

        if package_dir.exists() && package_dir.is_dir() {
            if force_install {
                remove_dir_all(&package_dir)?;
                // if let Err(e) = remove_dir_all(&package_dir) {
                //     error!("failed to delete dir({:?}): {}", package_dir, e);
                //     return;
                // }
            } else if package_bin_file.exists() && package_bin_file.is_file() {
                println!(
                    "the package {} with version {} is already installed, skip it",
                    package.name, release.version
                );
                return Ok(());
            }
        }
        download_with_progress(to_file, asset.download_url.as_str()).await?;
        // if let Err(e) = download_with_progress(to_file, asset.download_url.as_str()).await {
        //     error!("failed to download file: {}", e);
        //     return;
        // }

        create_dir_all(&package_dir)?;
        // if let Err(e) = create_dir_all(&package_dir) {
        //     error!("failed to create dir({:?}): {}", package_dir, e);
        //     return;
        // }

        let to_file = tmp_dir.join(&asset.name);
        let exe_ext = format!(".{}", EXE_EXTENSION);
        debug!(
            "{}, {}, {}",
            asset.name,
            exe_ext,
            asset.name.ends_with(&exe_ext)
        );
        if !EXE_EXTENSION.is_empty() && asset.name.ends_with(&exe_ext) {
            copy(&to_file, &package_bin_file)?;
        } else {
            match get_file_type(&asset.name) {
                FileType::Unknown => {
                    // error!("unsupported file type: {}", asset.name);
                    return Err(InstallError::GeneralStr("unsupported file type"));
                }
                FileType::Compression => match decompress(&to_file, &tmp_dir) {
                    Ok(folder) => {
                        debug!("decompress output: {:?}", folder);
                        let mut bin_file = match folder {
                            Some(f) => tmp_dir.join(f),
                            None => tmp_dir.clone(),
                        };
                        if let Some(f) = &package.bin_path {
                            bin_file = bin_file.join(f);
                        }
                        bin_file = bin_file.join(&package.bin_name);
                        bin_file.set_extension(EXE_EXTENSION);
                        copy(&bin_file, &package_bin_file)?;
                    }
                    Err(e) => {
                        return Err(InstallError::ToolFs(e));
                    }
                },
                FileType::Plain => {
                    copy(&to_file, &package_bin_file)?;
                }
            }
        }

        let mut sys_bin_file = app_dir.get_bin_dir().join(&package.bin_name);
        sys_bin_file.set_extension(EXE_EXTENSION);
        if sys_bin_file.exists() && sys_bin_file.is_file() {
            remove_link(&sys_bin_file)?;
        }
        make_link(&sys_bin_file, &package_bin_file)?;

        if let Some(alias_str) = alias {
            let mut sys_bin_file = app_dir.get_alias_dir().join(alias_str);
            sys_bin_file.set_extension(EXE_EXTENSION);
            if sys_bin_file.exists() && sys_bin_file.is_file() {
                remove_link(&sys_bin_file)?;
            }
            make_link(&sys_bin_file, &package_bin_file)?;
        }

        let _ = remove_dir_all(tmp_dir);
        Ok(())
    }
}

impl<'a, 'b> Installer for RustBinInstaller<'a, 'b> {
    async fn install(
        &mut self,
        package: &Package,
        request: &InstallRequest,
        app_dir: &AppDir,
        context: &InstallerContext,
    ) {
        let final_version = match &request.version {
            Some(v) => v,
            None => LATEST_VERSION,
        };
        let app_release = match find_assets(
            package,
            final_version,
            &package.remote_file_prefix,
            &context.rust_abi,
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                error!("failed to get assets: {}", e);
                return;
            }
        };

        if !app_release.assets.is_empty() {
            if let Err(e) = self.install_package(&app_release, package, &request.alias, app_dir, context.force)
                .await {
                    println!("{}", format!("failed to install {}: {}", package.name, e).red());
                } else {
                        println!(
                            "the package {} with version {} has been installed",
                            package.name.green(),
                            app_release.version.green()
                        );
                }
        } else if app_release.source_url.is_some() {
            print!("cannot find appropriate package to install, would you like to install from source code? [y/n]");
            let _ = std::io::stdout().flush();
            let answer = match self.global_input.read_line() {
                Ok(a) => a,
                Err(e) => {
                    error!("failed to read from command: {}", e);
                    return;
                }
            };
            debug!("install from source code? {}", answer);
            if answer == "y" {
                let src_installer = RustSrcInstaller {};
                if let Err(e) = src_installer
                    .install_package(&app_release, package, &request.alias, app_dir)
                    .await {
                        println!("{}", format!("failed to install from source code {}: {}", package.name, e).red());
                    } else {
                        println!(
                            "the package {} with version {} has been installed",
                            package.name.green(),
                            app_release.version.green()
                        );
                    }
            }
        } else {
            println!("{}", "cannot find appropriate package to install".red());
        }
    }
}
