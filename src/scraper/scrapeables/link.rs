use anyhow::{Context, Error, Result};
use reqwest::redirect::Policy;
use reqwest::Client;
use reqwest::Url;
use uuid::Uuid;

use crate::scraper::scrapable::Scrapeable;
use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;
use crate::scraper::scrape_type::ScrapeType;

#[derive(Debug)]
pub struct IliasLink {
    pub parent: Option<Uuid>,
    pub order_index: usize,
    pub url: Url,
    pub name: String,
}

#[async_trait::async_trait]
impl Scrapeable for IliasLink {
    async fn scrape(
        &self,
        _client: &Client,
        scrape_options: &ScrapeOptions,
    ) -> Result<(ScrapeObject, Vec<Box<dyn Scrapeable + Send + Sync>>), Error> {
        // println!("Scraping link {}", url);

        let custom_client = scrape_options.auth.authed_client(Policy::none()).await?;
        let r = custom_client
            .get(self.url.clone())
            .send()
            .await
            .context("Could not send request")?;
        let url = r
            .headers()
            .get("location")
            .context("No location header found in redirect response")?
            .to_str()
            .context("Failed to convert location header to string")?;

        //println!("Redirected to {}", url);

        let new_url = Url::parse(url).context("Could not parse redirect url")?;

        Ok((
            ScrapeObject::new(
                self.order_index,
                self.parent,
                ScrapeType::Link,
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
