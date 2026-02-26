use anyhow::{Error, Result};
use colored::{Color, Colorize};
use reqwest::Client;
use std::path::PathBuf;
use url::Url;

use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;
use crate::scraper::scrape_type::ScrapeType;
use crate::tree::downloadable::{download_file, save_link, Downloadable};
use crate::tree::linkable::Linkable;
use crate::tree::printable::Printable;
use crate::tree::tree_comparer::{compare_trees, ComparableTreeNode};
use crate::tree::tree_node::TreeNode;

impl TreeNode<ScrapeObject> for ScrapeObject {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_color(&self, indent: usize) -> colored::Color {
        if indent == 0 {
            Color::Red
        } else {
            self.item_type.get_color()
        }
    }

    fn is_container(&self) -> bool {
        self.item_type == ScrapeType::Folder
            || self.item_type == ScrapeType::LinkLibrary
            || self.item_type == ScrapeType::MediaLibrary
            || self.item_type == ScrapeType::Submissions
            || self.item_type == ScrapeType::Submission
    }

    fn get_children(&self) -> &Vec<Self> {
        self.children.as_ref()
    }
    fn update_children(&mut self, children: Vec<Self>) {
        self.children = children;
    }
}

impl ComparableTreeNode<ScrapeObject> for ScrapeObject {
    fn compare_as_remote<T: TreeNode<T>>(&self, other: &T) -> Self {
        compare_trees(other, self)
    }
    fn compare_as_local<T: TreeNode<T> + Clone>(&self, other: &T) -> T {
        compare_trees(self, other)
    }
}

impl Linkable<ScrapeObject> for ScrapeObject {
    fn get_url(&self) -> Option<Url> {
        Option::from(self.url.clone())
    }
}

impl Printable<ScrapeObject> for ScrapeObject {
    fn should_print(&self) -> bool {
        !self.item_type.eq(&ScrapeType::Ignore)
    }
}

#[async_trait::async_trait]
impl Downloadable<ScrapeObject> for ScrapeObject {
    async fn download_node(&self, client: &Client, path: PathBuf) -> Result<(), Error> {
        if let Some(url) = self.get_url() {
            if self.item_type.eq(&ScrapeType::Link) {
                save_link(url, self.get_name(), path)?;
            } else {
                println!(
                    ">> Downloading {} to {:?}",
                    self.get_name().color(self.get_color(1)),
                    path
                );

                download_file(client, url, self.get_name(), path).await?
            }
        }

        Ok(())
    }

    fn should_download(&self, scrape_options: &ScrapeOptions) -> bool {
        self.item_type.eq(&ScrapeType::File)
            || self.item_type.eq(&ScrapeType::Link)
            || scrape_options.videos && self.item_type.eq(&ScrapeType::Video)
    }
}
