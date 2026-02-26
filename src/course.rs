use crate::scraper::scrapable::Scrapeable;
use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;
use colored::Colorize;
use reqwest::Client;
use std::path::PathBuf;
use url::Url;

use crate::tree::local_tree_handler::build_tree;

use crate::scraper::scrapeables::builder::build_root_node;
use crate::tree::downloadable::Downloadable;
use crate::tree::local_tree_node::LocalTreeNode;
use crate::tree::printable::Printable;
use crate::tree::tree_comparer::ComparableTreeNode;

#[derive(Debug, Clone)]
pub(crate) struct Course {
    pub(crate) name: String,
    pub(crate) id: u32,
}

impl Course {
    pub(crate) fn new(name: String, id: u32) -> Course {
        Course { name, id }
    }

    pub fn get_root_url(&self) -> Url {
        format!("https://ilias.studium.kit.edu/goto.php/fold/{}", self.id)
            .parse()
            .unwrap()
    }

    pub fn build_remote_root(&self, index: usize) -> Box<dyn Scrapeable> {
        Box::new(build_root_node(
            index,
            self.name.as_ref(),
            self.get_root_url(),
        ))
    }

    pub(crate) fn tree_local(&self, base_path: PathBuf) -> LocalTreeNode {
        build_tree(self, base_path)
    }

    pub(crate) async fn sync(
        &self,
        base_path: PathBuf,
        client: &Client,
        remote_tree_root: ScrapeObject,
        scrape_options: &ScrapeOptions,
    ) {
        let local = self.tree_local(base_path.clone());

        let result = remote_tree_root.compare_as_remote(&local);

        result.print();

        println!(
            ">> Downloading course: {}",
            self.name.color(colored::Color::Blue)
        );
        result
            .download_tree(client, base_path, scrape_options)
            .await;
    }
}
