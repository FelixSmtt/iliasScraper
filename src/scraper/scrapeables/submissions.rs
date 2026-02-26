use anyhow::Context;
use anyhow::{Error, Result};
use reqwest::Client;
use reqwest::Url;
use uuid::Uuid;

use crate::scraper::scrapable::{build_scrapeable, Scrapeable};
use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;
use crate::scraper::scrape_type::ScrapeType;
use crate::scraper::scrapeables::request::request_page;
use crate::utils::sanitize::sanitize_name;

#[derive(Debug)]
pub struct IliasSubmissions {
    pub parent: Option<Uuid>,
    pub order_index: usize,
    pub url: Url,
    pub name: String,
}

#[async_trait::async_trait]
impl Scrapeable for IliasSubmissions {
    async fn scrape(
        &self,
        client: &Client,
        scrape_options: &ScrapeOptions,
    ) -> Result<(ScrapeObject, Vec<Box<dyn Scrapeable + Send + Sync>>), Error> {
        let object_id = self
            .url
            .as_str()
            .split("/")
            .last()
            .context("Invalid submission url")?;
        let url = format!( "https://ilias.studium.kit.edu/ilias.php?baseClass=ilexercisehandlergui&cmd=showOverview&ref_id={}&mode=all", object_id);

        let text = request_page(url.as_str(), client).await?;
        let document = scraper::Html::parse_document(&text);

        let result = ScrapeObject::new(
            self.order_index,
            self.parent,
            ScrapeType::Submissions,
            url.parse().unwrap(),
            self.name.clone(),
        );

        if !document.errors.is_empty() && scrape_options.verbose {
            eprintln!(
                "Warning: {} parse errors while scraping {} (usually safe to ignore)",
                document.errors.len(),
                self.url
            );
        }

        // extract links
        let container_selector = scraper::Selector::parse(".il-std-item-container").unwrap();

        let links: Vec<Box<dyn Scrapeable + Send + Sync>> = document
            .select(&container_selector)
            .enumerate()
            .filter_map(|(idx, link)| {
                let href_container_selector =
                    scraper::Selector::parse(".il-item-title > a").ok()?;
                let href_container = link.select(&href_container_selector).next()?;

                let url = href_container.value().attr("href")?.to_string();

                let title = href_container
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string();

                let child_name = sanitize_name(title);
                Some(build_scrapeable(
                    idx,
                    result.id.into(),
                    url,
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
