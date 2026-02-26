use url::Url;
use uuid::Uuid;

use crate::scraper::scrapable::{Scrapeable, TransientScrapeable};

use crate::scraper::scrape_type::ScrapeType;
use crate::scraper::scrapeables::folder::IliasFolder;
use crate::scraper::scrapeables::link::IliasLink;
use crate::scraper::scrapeables::link_library::IliasLinkLibrary;
use crate::scraper::scrapeables::media_library::IliasMediaLibrary;
use crate::scraper::scrapeables::submission::IliasSubmission;
use crate::scraper::scrapeables::submissions::IliasSubmissions;
use crate::scraper::scrapeables::video::IliasVideo;

impl ScrapeType {
    pub fn get_scrapable(
        &self,
        order_index: usize,
        parent: Option<Uuid>,
        url: Url,
        name: String,
    ) -> Box<dyn Scrapeable> {
        match self {
            ScrapeType::Folder => Box::new(IliasFolder {
                order_index,
                parent,
                url,
                name,
            }),
            ScrapeType::Forum => Box::new(TransientScrapeable {
                order_index,
                parent,
                item_type: ScrapeType::Forum,
                url,
                name,
            }),
            ScrapeType::MediaLibrary => Box::new(IliasMediaLibrary {
                order_index,
                parent,
                url,
                name,
            }),
            ScrapeType::Link => Box::new(IliasLink {
                order_index,
                parent,
                url,
                name,
            }),
            ScrapeType::LinkLibrary => Box::new(IliasLinkLibrary {
                order_index,
                parent,
                url,
                name,
            }),
            ScrapeType::Submissions => Box::new(IliasSubmissions {
                order_index,
                parent,
                url,
                name,
            }),
            ScrapeType::Submission => Box::new(IliasSubmission {
                order_index,
                parent,
                url,
                name,
            }),
            ScrapeType::File => Box::new(TransientScrapeable {
                order_index,
                parent,
                item_type: ScrapeType::File,
                url,
                name,
            }),
            ScrapeType::Video => Box::new(IliasVideo {
                order_index,
                parent,
                url,
                name,
            }),
            ScrapeType::Calender => Box::new(TransientScrapeable {
                order_index,
                parent,
                item_type: ScrapeType::Calender,
                url,
                name,
            }),
            ScrapeType::Ignore => Box::new(TransientScrapeable {
                order_index,
                parent,
                item_type: ScrapeType::Ignore,
                url,
                name,
            }),
        }
    }
}

pub fn build_root_node(index: usize, name: &str, url: Url) -> IliasFolder {
    IliasFolder {
        parent: None,
        order_index: index,
        url,
        name: name.to_string(),
    }
}
