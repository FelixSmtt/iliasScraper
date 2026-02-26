use anyhow::{Context, Error, Result};
use reqwest::Client;
use reqwest::Url;
use uuid::Uuid;

use crate::scraper::scrapable::Scrapeable;
use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;
use crate::scraper::scrape_type::ScrapeType;
use crate::scraper::scrapeables::request::request_page;

#[derive(Debug)]
pub struct IliasVideo {
    pub parent: Option<Uuid>,
    pub order_index: usize,
    pub url: Url,
    pub name: String,
}

#[async_trait::async_trait]
impl Scrapeable for IliasVideo {
    async fn scrape(
        &self,
        client: &Client,
        scrape_options: &ScrapeOptions,
    ) -> Result<(ScrapeObject, Vec<Box<dyn Scrapeable + Send + Sync>>), Error> {
        //println!("Scraping folder with URL: {:#?}", url.to_string());
        let text = request_page(self.url.as_str(), client).await?;
        let document = scraper::Html::parse_document(&text);

        if !document.errors.is_empty() && scrape_options.verbose {
            eprintln!(
                "Warning: {} parse errors while scraping {} (usually safe to ignore)",
                document.errors.len(),
                self.url
            );
        }

        let start_video_url_index = text
            .find("[{\"src\":\"")
            .context("Failed to find the start of the video URL in the page text")?;
        let video_url_index = text
            .find("\",\"mimetype\":\"")
            .context("Failed to find the end of the video URL in the page text")?;

        let video_url = &text[(start_video_url_index + 9)..video_url_index];

        // println!("Video URL: {}", video_url);

        let new_url: Url = video_url.parse().context("Failed to parse the video URL")?;

        Ok((
            ScrapeObject::new(
                self.order_index,
                self.parent,
                ScrapeType::Video,
                new_url,
                self.name.clone(),
            ),
            vec![],
        ))
    }

    fn get_url(&self) -> Url {
        self.url.clone()
    }
}
