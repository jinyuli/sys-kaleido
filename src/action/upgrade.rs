use crate::tool::logger::error;

pub fn upgrade() {
    let release_update = match self_update::backends::github::Update::configure()
        .repo_owner("jinyuli")
        .repo_name("sys-kaleido")
        .bin_name("sys-kaleido")
        .show_download_progress(true)
        .current_version(self_update::cargo_crate_version!())
        .build()
    {
        Ok(u) => u,
        Err(e) => {
            error!("failed to check update: {}", e);
            return;
        }
    };
    match release_update.update() {
        Ok(s) => match s {
            self_update::Status::UpToDate(v) => {
                println!("sys-kaleido has been updated to {}.", v);
            }
            self_update::Status::Updated(v) => {
                println!("sys-kaleido {} is the latest version.", v);
            }
        },
        Err(e) => {
            error!("failed to check update: {}", e);
        }
    };
}