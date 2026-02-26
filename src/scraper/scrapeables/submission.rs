use anyhow::{Error, Result};
use reqwest::Client;
use url::Url;
use uuid::Uuid;

use crate::scraper::scrapable::{build_scrapeable, Scrapeable};
use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;
use crate::scraper::scrape_type::ScrapeType;
use crate::scraper::scrapeables::request::request_page;
use crate::utils::sanitize::{remove_extension, sanitize_name};

#[derive(Debug)]
pub struct IliasSubmission {
    pub parent: Option<Uuid>,
    pub order_index: usize,
    pub url: Url,
    pub name: String,
}

#[async_trait::async_trait]
impl Scrapeable for IliasSubmission {
    async fn scrape(
        &self,
        client: &Client,
        scrape_options: &ScrapeOptions,
    ) -> Result<(ScrapeObject, Vec<Box<dyn Scrapeable + Send + Sync>>), Error> {
        let url = self
            .url
            .to_string()
            .replace("ilExSubmissionFileGUI", "ilAssignmentPresentationGUI");

        let text = request_page(url.as_str(), client).await?;
        let document = scraper::Html::parse_document(&text);

        let result = ScrapeObject::new(
            self.order_index,
            self.parent,
            ScrapeType::Submission,
            self.url.clone(),
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
        let container_selector = scraper::Selector::parse(
            ".panel-primary > .panel-body > .panel-sub:nth-child(2)  .row",
        )
        .unwrap();

        let links: Vec<Box<dyn Scrapeable + Send + Sync>> = document
            .select(&container_selector)
            .enumerate()
            .filter_map(|(idx, link)| {
                let href_container_selector = scraper::Selector::parse("a").ok()?;
                let href_container = link.select(&href_container_selector).next()?;

                let url = href_container.value().attr("href")?.to_string();

                let label_container_selector =
                    scraper::Selector::parse("div.control-label > p").ok()?;
                let label_container = link.select(&label_container_selector).next()?;

                let title = label_container
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string();

                let child_name = sanitize_name(title);
                // Submission files can contain the extension, remove this
                let child_name = remove_extension(&child_name);

                Some(build_scrapeable(
                    idx,
                    result.id.into(),
                    url,
                    child_name.to_string(),
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
