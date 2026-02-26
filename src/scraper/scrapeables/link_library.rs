use crate::scraper::scrapable::{build_scrapeable, Scrapeable};
use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;
use crate::scraper::scrape_type::ScrapeType;
use anyhow::{Error, Result};
use reqwest::Client;
use reqwest::Url;
use scraper::Element;
use uuid::Uuid;

use crate::scraper::scrapeables::request::request_page;
use crate::utils::sanitize::sanitize_name;

#[derive(Debug)]
pub struct IliasLinkLibrary {
    pub parent: Option<Uuid>,
    pub order_index: usize,
    pub url: Url,
    pub name: String,
}

#[async_trait::async_trait]
impl Scrapeable for IliasLinkLibrary {
    async fn scrape(
        &self,
        client: &Client,
        scrape_options: &ScrapeOptions,
    ) -> Result<(ScrapeObject, Vec<Box<dyn Scrapeable + Send + Sync>>), Error> {
        let text = request_page(self.url.as_str(), client).await?;
        let document = scraper::Html::parse_document(&text);

        // extract links
        let link_selector = scraper::Selector::parse("td.std").unwrap();

        let result = ScrapeObject::new(
            self.order_index,
            self.parent,
            ScrapeType::LinkLibrary,
            self.url.clone(),
            self.name.clone(),
        );

        // Convert Select iterator to Vec
        let links: Vec<Box<dyn Scrapeable + Send + Sync>> = document
            .select(&link_selector)
            .enumerate()
            .filter_map(|(idx, link)| {
                let child_element = link.first_element_child()?;
                let child_url = child_element.value().attr("href").map(|s| s.to_string())?;
                let child_name = sanitize_name(child_element.inner_html());

                Some(build_scrapeable(
                    idx,
                    result.id.into(),
                    child_url,
                    child_name,
                    scrape_options,
                ))
            })
            .collect();

        Ok((result, links))
    }

    fn get_url(&self) -> Url {
        self.url.clone()
    }
}
