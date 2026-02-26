use crate::scraper::scrape_type::ScrapeType;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ScrapeObject {
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub order_index: usize,

    pub item_type: ScrapeType,
    pub url: Url,
    pub name: String,
    pub children: Vec<ScrapeObject>,
}

impl ScrapeObject {
    pub(crate) fn new(
        order_index: usize,
        parent: Option<Uuid>,
        item_type: ScrapeType,
        url: Url,
        name: String,
    ) -> ScrapeObject {
        ScrapeObject {
            id: Uuid::new_v4(),
            parent,
            item_type,
            url,
            name,
            children: Vec::new(),
            order_index,
        }
    }
}
