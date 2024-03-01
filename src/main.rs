use crate::{
    action::{
        config::{check_config, update as config_update},
        installer::{install, InstallRequest, InstallerContext},
        list::{list, list_all},
        search::search,
        uninstall::uninstall,
        upgrade::upgrade,
        bindle::{
            list as bindle_list,
            install as bindle_install,
            uninstall as bindle_uninstall,
            update as bindle_update,
        }
    },
    tool::{
        args,
        fs::AppDir,
        logger::{debug, init_logger},
    },
};

mod action;
mod tool;

#[tokio::main]
async fn main() {
    let app_dir = AppDir::new().unwrap();
    init_logger(app_dir.get_log_dir()).unwrap();
    let command: args::SysKaleidoCommand = argh::from_env();
    debug!("args: {:?}", command);
    match command.cmd {
        args::TopCommand::Search(cmd) => {
            check_config(app_dir.get_home_dir()).await;
            search(cmd.package, app_dir.get_home_dir());
        }
        args::TopCommand::Install(cmd) => {
            check_config(app_dir.get_home_dir()).await;
            let rust_abi = match cmd.rust_abi {
                Some(abi) => abi,
                None => default_abi(),
            };
            let context = InstallerContext { rust_abi, force: cmd.force };
            let requests: Vec<InstallRequest> = if cmd.packages.len() == 1 {
                vec![InstallRequest{name: cmd.packages[0].clone(), version: cmd.version, alias: cmd.alias}]
            } else {
                cmd.packages.into_iter().map(|p| InstallRequest{ name: p, version: None, alias: None}).collect()
            };

            install(requests, &app_dir, &context).await;
        }
        args::TopCommand::Bindle(cmd) => {
            check_config(app_dir.get_home_dir()).await;
            debug!("bindle command {:?}", cmd);

            match cmd.command {
                args::BindleSubCommand::Install(cmd) => {
                    let rust_abi = match cmd.rust_abi {
                        Some(abi) => abi,
                        None => default_abi(),
                    };
                    let context = InstallerContext { rust_abi, force: cmd.force };
                    bindle_install(cmd.name, &app_dir, &context).await;
                }
                args::BindleSubCommand::Uninstall(cmd) => {
                    bindle_uninstall(cmd.name, &app_dir).await;
                }
                args::BindleSubCommand::Update(cmd) => {
                    let rust_abi = match cmd.rust_abi {
                        Some(abi) => abi,
                        None => default_abi(),
                    };
                    let context = InstallerContext { rust_abi, force: cmd.force };
                    bindle_update(cmd.name, &app_dir, &context).await;
                }
                args::BindleSubCommand::List(cmd) => {
                    bindle_list(cmd.name, &app_dir).await;
                }
            }
        }
        args::TopCommand::Version(_) => {
            execute_version();
        }
        args::TopCommand::Update(cmd) => {
            check_config(app_dir.get_home_dir()).await;
            let rust_abi = match cmd.rust_abi {
                Some(abi) => abi,
                None => default_abi(),
            };
            let context = InstallerContext { rust_abi, force: cmd.force };
            let requests: Vec<InstallRequest> = if cmd.packages.len() == 1 {
                vec![InstallRequest{name: cmd.packages[0].clone(), version: cmd.version, alias: cmd.alias}]
            } else {
                cmd.packages.into_iter().map(|p| InstallRequest{ name: p, version: None, alias: None}).collect()
            };

            install(requests, &app_dir, &context).await;
        }
        args::TopCommand::Upgrade(_) => {
            upgrade();
            config_update(app_dir.get_home_dir()).await;
        }
        args::TopCommand::Config(cmd) => match cmd.command {
            args::ConfigSubCommand::Update(_) => {
                config_update(app_dir.get_home_dir()).await;
            }
        },
        args::TopCommand::List(cmd) => {
            if cmd.all {
                list_all(&app_dir).await;
            } else {
                list(&app_dir).await;
            }
        }
        args::TopCommand::Uninstall(cmd) => {
            uninstall(cmd.packages, &app_dir).await;
        }
    }
}

fn execute_version() {
    println!("sys-kaleido {}", env!("CARGO_PKG_VERSION"));
}

#[cfg(target_os = "windows")]
fn default_abi() -> String {
    "msvc".to_string()
}

#[cfg(not(target_os = "windows"))]
fn default_abi() -> String {
    "gnu".to_string()
}
