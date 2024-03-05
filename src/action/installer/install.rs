use super::execute::{InstallRequest, LATEST_VERSION};
use crate::tool::{
    fs::AppDir,
    kaleido::{Github, Package},
};
use log::{debug, error};
use octocrab::models::repos::Release;
use reqwest::{self};
use std::env::consts::{ARCH, OS};
use std::{collections::HashSet, vec};
use thiserror::Error;

type Result<T> = std::result::Result<T, InstallError>;

pub struct InstallerContext {
    pub rust_abi: String,
    /// if true, the installed version will be deleted, then install the package again, even it's the same version.
    /// otherwise, the package will be skipped if the given version has been installed.
    pub force: bool,
}

pub trait Installer {
    /// install a package, this method should not panic.
    /// return true if success, return false otherwise.
    async fn install(
        &mut self,
        package: &Package,
        request: &InstallRequest,
        app_dir: &AppDir,
        context: &InstallerContext,
    );
}

pub async fn find_assets(
    package: &Package,
    version: &str,
    remote_file_prefix: &Option<String>,
    rust_abi: &str,
) -> Result<AppRelease> {
    debug!(
        "try to find assets: {}, {}, {}",
        package.name, version, rust_abi
    );
    match package.github.as_ref() {
        Some(gt) => {
            let release = match find_github_release(gt, version).await {
                Ok(o) => match o {
                    Some(r) => r,
                    None => {
                        return Err(InstallError::GeneralStr("cannot find appropriate release"))
                    }
                },
                Err(e) => {
                    return Err(e);
                }
            };

            let assets: Vec<_> = release
                .assets
                .iter()
                .filter(|a| {
                    let segments: HashSet<_> = a.name.split(&['-', '.']).collect();
                    segments.contains(to_github_arch(ARCH))
                        && segments.contains(to_github_os(OS))
                        && segments.contains(rust_abi)
                        && (remote_file_prefix.is_none()
                            || remote_file_prefix
                                .as_ref()
                                .is_some_and(|r| segments.contains(r.as_str())))
                })
                .map(|a| AppAsset {
                    name: a.name.clone(),
                    download_url: a.browser_download_url.to_string(),
                })
                .collect();

            Ok(AppRelease {
                version: release.tag_name.clone(),
                source_url: release.zipball_url.map(|u| u.to_string()),
                assets,
            })
        }
        None => Err(InstallError::GeneralStr(
            "currently only support to install from github.com",
        )),
    }
}

#[derive(Clone, Debug)]
pub struct AppRelease {
    pub version: String,
    pub source_url: Option<String>,
    pub assets: Vec<AppAsset>,
}

#[derive(Clone, Debug)]
pub struct AppAsset {
    /// file name
    pub name: String,
    /// download url
    pub download_url: String,
}

async fn find_github_release(gt: &Github, version: &str) -> Result<Option<Release>> {
    let client = octocrab::instance();
    let mut page: u32 = 0;
    // assume that tag name is the version, or `v` + version
    let mut versions = vec![version.to_string()];
    if version != LATEST_VERSION && !version.starts_with('v') {
        versions.push(format!("v{}", version));
    }
    loop {
        match client
            .repos(&gt.org, &gt.repo)
            .releases()
            .list()
            .page(page)
            .per_page(10)
            .send()
            .await
        {
            Ok(s) => {
                if s.items.is_empty() {
                    return Ok(None);
                }
                // filter by draft and prerelease
                match s.items.into_iter().find(|r| {
                    if version == LATEST_VERSION {
                        !(r.draft || r.prerelease)
                    } else {
                        // TODO maybe the tag_name format can be configured in kaleido.toml
                        versions.contains(&r.tag_name)
                    }
                }) {
                    Some(r) => return Ok(Some(r)),
                    None => {
                        page += 1;
                        debug!("load next page releases: {}", page);
                    }
                };
            }
            Err(e) => {
                return Err(InstallError::General(format!(
                    "failed to load releases: {}",
                    e
                )));
            }
        }
    }
}

fn to_github_os(os: &str) -> &str {
    match os {
        "macos" => "darwin",
        _ => os,
    }
}

fn to_github_arch(arch: &str) -> &str {
    arch
}

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("file system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("{0}")]
    General(String),
    #[error("{0}")]
    GeneralStr(&'static str),
}
