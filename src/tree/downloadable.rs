use anyhow::{Context, Error, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::future::Future;
use std::io::Write;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::scraper::scrape_options::ScrapeOptions;
use crate::tree::linkable::Linkable;
use crate::tree::tree_node::TreeNode;

#[async_trait::async_trait]
pub(crate) trait Downloadable<T: Downloadable<T>>: TreeNode<T> + Linkable<T> {
    fn download_tree<'a>(
        &'a self,
        client: &'a Client,
        path: PathBuf,
        scrape_options: &'a ScrapeOptions,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>>
    where
        Self: Sized + 'a,
    {
        Box::pin(async move {
            if self.is_container() {
                let deeper_path = path.join(self.get_name().clone());
                std::fs::create_dir_all(deeper_path.clone()).unwrap();

                for child in self.get_children() {
                    child
                        .download_tree(client, deeper_path.clone(), scrape_options)
                        .await;
                }
            } else if self.should_download(scrape_options) {
                self.download_node(client, path.clone())
                    .await
                    .unwrap_or_else(|err| println!("Error downloading file: {:?}", err));
            }
        })
    }

    async fn download_node(&self, client: &Client, path: PathBuf) -> Result<(), Error>;
    fn should_download(&self, scrape_options: &ScrapeOptions) -> bool;
}

pub(crate) fn save_link(link: Url, name: &str, path: PathBuf) -> Result<(), Error> {
    let file_loc = path.join(name.to_owned() + ".url");

    let mut file = std::fs::File::create(file_loc).context("Could not create shortcut file")?;
    file.write_all(format!("[InternetShortcut]\nURL={}", link).as_bytes())
        .context("Could not write to shortcut file")?;

    Ok(())
}

async fn request_file(client: &Client, url: Url) -> Result<reqwest::Response, Error> {
    let mut response = client
        .get(url.as_str())
        .send()
        .await
        .context("Error requesting file")?;

    let mut headers = response.headers().clone();

    while let Some(location) = headers.get("location") {
        let location = location.to_str().context("Location header is not UTF-8")?;

        response = client
            .get(location)
            .send()
            .await
            .context("Error requesting redirected file")?;
        headers = response.headers().clone();
    }

    Ok(response)
}

fn parse_file_name_from_response(
    file_path: PathBuf,
    response: &reqwest::Response,
) -> Option<PathBuf> {
    let content_disposition = response
        .headers()
        .get("content-disposition")?
        .to_str()
        .ok()?;
    let extension = content_disposition.split(".").last()?.split("\"").next()?;
    let extension = ".".to_owned() + extension;

    let mut new_file_name = file_path;
    if !new_file_name.to_str()?.ends_with(&extension) {
        new_file_name = PathBuf::from(new_file_name.to_str()?.to_owned() + &*extension);
    }

    Some(new_file_name)
}

pub(crate) async fn download_file(
    client: &Client,
    url: Url,
    filename: &str,
    path: PathBuf,
) -> Result<(), Error> {
    let file_loc = path.join(filename);

    let response = request_file(client, url).await?;

    let total_length = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

    let new_file_name = parse_file_name_from_response(file_loc, &response)
        .context("Could not parse file name from response.")?;

    let mut file = File::create(new_file_name).await?;

    let pb = match total_length {
        Some(len) => {
            let pb = ProgressBar::new(len);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
                    .unwrap(),
            );
            pb
        }
        None => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner} Downloading... {bytes}")
                    .unwrap(),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            pb
        }
    };

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error reading response stream")?;

        file.write_all(&chunk)
            .await
            .context("Error writing downloaded file")?;

        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Download complete");
    println!();

    Ok(())
}
