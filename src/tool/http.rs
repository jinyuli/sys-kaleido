use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use log::{debug, error, info};
use reqwest::{
    self,
    header::{HeaderMap, USER_AGENT},
};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use thiserror::Error;
use tokio::task;

pub type Result<T> = std::result::Result<T, HttpError>;

pub async fn download_with_progress(to_file: PathBuf, download_url: &str) -> Result<()> {
    let url = download_url.to_string();
    let (sender, receiver) = channel();
    task::spawn(async move {
        let s = sender.clone();
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "sys-kaleido".parse().unwrap());
        match download(&url, &to_file, headers, s).await {
            Err(e) => {
                let _ = sender.send(ProgressInfo::Error(e.to_string()));
            }
            Ok(_) => {
                let _ = sender.send(ProgressInfo::Complete());
            }
        }
    });

    let mut pb: Option<ProgressBar> = None;
    while let Ok(p) = receiver.recv() {
        match p {
            ProgressInfo::Complete() => {
                info!("download file successfully");
                if let Some(p) = pb.as_ref() {
                    p.finish_with_message("download file successfully");
                }
                break;
            }
            ProgressInfo::Error(e) => {
                error!("failed to download file: {}", e);
                if let Some(p) = pb.as_ref() {
                    p.finish_with_message(format!("failed to download file: {}", e));
                }
                return Err(HttpError::General(e));
            }
            ProgressInfo::Progress { current, total: _ } => {
                // info!("process: {}/{:?}", current, total);
                if let Some(p) = pb.as_ref() {
                    p.set_position(current);
                }
            }
            ProgressInfo::Start(size) => {
                if size > 0 {
                    let _pb = ProgressBar::new(size);
                    _pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                        .unwrap()
                        .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                        .progress_chars("#>-"));
                    pb = Some(_pb);
                } else {
                    let _pb = ProgressBar::new_spinner();
                    _pb.set_style(
                        ProgressStyle::with_template("{spinner:.blue} {msg}")
                            .unwrap()
                            .tick_strings(&[
                                "▹▹▹▹▹",
                                "▸▹▹▹▹",
                                "▹▸▹▹▹",
                                "▹▹▸▹▹",
                                "▹▹▹▸▹",
                                "▹▹▹▹▸",
                                "▪▪▪▪▪",
                            ]),
                    );
                    _pb.set_message("Downloading...");
                    pb = Some(_pb);
                }
            }
        }
    }
    Ok(())
}

pub async fn download_file(url: &str, to_file: &Path) -> Result<()> {
    let client = get_client().await?;
    let body = client.get(url).send().await?.bytes().await?;
    let mut file = File::create(to_file)?;
    file.write_all(&body)?;
    Ok(())
}

async fn download(
    url: &str,
    to_file: &Path,
    headers: reqwest::header::HeaderMap,
    sender: Sender<ProgressInfo>,
) -> Result<()> {
    let client = get_client().await?;
    let res = client.get(url).headers(headers).send().await?;
    debug!("{:?}", res.headers());
    let content_len = res.content_length();
    let mut stream = res.bytes_stream();
    let mut file = File::create(to_file)?;
    let mut downloaded_bytes: u64 = 0;
    let total = content_len.unwrap_or(0);
    let _ = sender.send(ProgressInfo::Start(total));
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                file.write_all(&c)?;
                downloaded_bytes += c.len() as u64;
                let _ = sender.send(ProgressInfo::Progress {
                    current: downloaded_bytes,
                    total: content_len,
                });
            }
            Err(e) => {
                return Err(HttpError::General(e.to_string()));
            }
        }
    }
    Ok(())
}

async fn get_client() -> Result<reqwest::Client> {
    let client = reqwest::Client::builder().build()?;
    Ok(client)
}

#[derive(Debug, Clone)]
pub enum ProgressInfo {
    Start(u64),
    Progress { current: u64, total: Option<u64> },
    Complete(),
    Error(String),
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("file system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("{0}")]
    General(String),
}
