use log::error;
use crate::tool::kaleido;
use std::path::Path;

pub fn search(keyword: String, dir: &Path) {
    let kaleido = match kaleido::load_config(dir) {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to load metadata: {}", e);
            return;
        }
    };
    for package in kaleido.packages {
        if package.name.contains(&keyword) || package.replace.is_some_and(|r| r.contains(&keyword))
        {
            println!("{}", package.name);
        }
    }
}
