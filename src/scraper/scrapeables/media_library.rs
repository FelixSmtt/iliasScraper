use anyhow::{Context, Error, Result};
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
pub struct IliasMediaLibrary {
    pub parent: Option<Uuid>,
    pub order_index: usize,
    pub url: Url,
    pub name: String,
}

#[async_trait::async_trait]
impl Scrapeable for IliasMediaLibrary {
    async fn scrape(
        &self,
        client: &Client,
        scrape_options: &ScrapeOptions,
    ) -> Result<(ScrapeObject, Vec<Box<dyn Scrapeable + Send + Sync>>), Error> {
        let result = ScrapeObject::new(
            self.order_index,
            self.parent,
            ScrapeType::MediaLibrary,
            self.url.clone(),
            self.name.clone(),
        );

        if !scrape_options.videos {
            return Ok((result, vec![]));
        }

        let text = request_page(self.url.as_str(), client).await?;

        let start_async_url_index = text
            .find("url: '/ilias.php")
            .context("Failed to find the start of the media library url in the page text")?;
        let end_async_url_index = text
            .find("&async=true'")
            .context("Failed to find the end of the media library url in the page text")?;

        let async_url = "https://ilias.studium.kit.edu".to_owned()
            + &text[(start_async_url_index + 6)..(end_async_url_index + 11)];

        //println!("Scraping media library with ACTUAL URL: {:#?}", async_url);
        let text = request_page(&async_url, client).await?;
        let document = scraper::Html::parse_document(&text);

        // table
        let table_selector = scraper::Selector::parse("table.table>tbody>tr").unwrap();
        //println!("Table item amount: {}", document.select(&table_selector).count());

        let links: Vec<Box<dyn Scrapeable + Send + Sync>> = document
            .select(&table_selector)
            .enumerate()
            .filter_map(|(idx, link)| {
                // get url
                let buttons_selector =
                    scraper::Selector::parse("div.btn-group-vertical > a").ok()?;
                let buttons = link.select(&buttons_selector).last()?;
                let url = String::from(buttons.value().attr("href")?);

                // make file name
                let title_selector = scraper::Selector::parse("td.std.small").ok()?;
                let title_name = link
                    .select(&title_selector)
                    .nth(2)?
                    .text()
                    .collect::<Vec<_>>()
                    .join("");
                let date = link
                    .select(&title_selector)
                    .nth(5)?
                    .text()
                    .collect::<Vec<_>>()
                    .join("");

                let title = format!(
                    "{} ({})",
                    sanitize_name(title_name).trim(),
                    sanitize_name(date.trim().to_owned()).trim()
                );
                Some(build_scrapeable(
                    idx,
                    result.id.into(),
                    url,
                    title,
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
