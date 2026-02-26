use anyhow::{Error, Result};
use reqwest::Client;
use url::Url;
use uuid::Uuid;

use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;
use crate::scraper::scrape_type::ScrapeType;

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait Scrapeable: Send + Sync + 'static {
    async fn scrape(
        &self,
        client: &Client,
        scrape_options: &ScrapeOptions,
    ) -> Result<(ScrapeObject, Vec<Box<dyn Scrapeable + Send + Sync>>), Error>;
    fn get_url(&self) -> Url;
}

pub struct TransientScrapeable {
    pub parent: Option<Uuid>,
    pub order_index: usize,
    pub item_type: ScrapeType,
    pub url: Url,
    pub name: String,
}

#[async_trait::async_trait]
impl Scrapeable for TransientScrapeable {
    async fn scrape(
        &self,
        _client: &Client,
        _scrape_options: &ScrapeOptions,
    ) -> Result<(ScrapeObject, Vec<Box<dyn Scrapeable + Send + Sync>>), Error> {
        Ok((
            ScrapeObject::new(
                self.order_index,
                self.parent,
                self.item_type.clone(),
                self.url.clone(),
                self.name.clone(),
            ),
            Vec::new(),
        ))
    }

    fn get_url(&self) -> Url {
        self.url.clone()
    }
}

fn fix_url(url: String) -> String {
    if url.starts_with("https") {
        return url;
    }

    format!("https://ilias.studium.kit.edu/{}", url)
}

pub fn build_scrapeable(
    index: usize,
    parent: Option<Uuid>,
    url: String,
    name: String,
    _scrape_options: &ScrapeOptions,
) -> Box<dyn Scrapeable + Send + Sync> {
    let url = Url::parse(&fix_url(url)).unwrap();
    let s_type = ScrapeType::from_url(&url);

    s_type.get_scrapable(index, parent, url, name)
}
