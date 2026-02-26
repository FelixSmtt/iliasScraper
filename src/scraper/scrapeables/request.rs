use anyhow::{Context, Error};
use reqwest::{Client, Response};

pub async fn request_page(url: &str, client: &Client) -> Result<String, Error> {
    let response: Response = client
        .get(url)
        .send()
        .await
        .context("Could not send request")?;
    let text = response
        .text()
        .await
        .context("Could not read response text")?
        .to_string();

    Ok(text)
}
