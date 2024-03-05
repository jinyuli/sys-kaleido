use super::helper::get_installed_packages;
use crate::tool::{fs::AppDir, kaleido};
use colored::Colorize;
use log::error;

pub async fn search(keyword: String, app_dir: &AppDir) {
    let kaleido = match kaleido::load_config(app_dir.get_home_dir()) {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to load metadata: {}", e);
            return;
        }
    };

    let installed_packages = match get_installed_packages(app_dir).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to get installed packages: {}", e);
            return;
        }
    };

    for package in kaleido.packages {
        if package.name.contains(&keyword) || package.replace.is_some_and(|r| r.contains(&keyword))
        {
            match installed_packages
                .iter()
                .find(|p| p.package.name == package.name)
            {
                Some(ip) => {
                    print!("* {} ({})", package.name.green(), ip.version.green());
                }
                None => {
                    print!("  {}", package.name);
                }
            }
            println!();
        }
    }
}
