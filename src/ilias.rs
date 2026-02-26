use anyhow::{Context, Ok, Result};
use reqwest::redirect::Policy;
use reqwest::Client;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::auth::provider::{AuthProvider, AuthProviderFactory};
use crate::config::{get_config_dir, Config};
use crate::course::Course;
use crate::scraper::scrape_object::ScrapeObject;
use crate::scraper::scrape_options::ScrapeOptions;
use crate::scraper::scraper::scrape_courses;
use crate::tree::local_tree_node::LocalTreeNode;
use crate::tree::tree_connector::{connect_trees, TreeConnectorNode};

#[derive(Clone, Debug)]
pub struct Ilias<AuthProviderType: AuthProvider + AuthProviderFactory> {
    pub courses: Vec<Course>,

    client: Option<Client>,
    auth_provider: AuthProviderType,

    base_path: PathBuf,
}

impl<AuthProviderType: AuthProvider + AuthProviderFactory> Ilias<AuthProviderType> {
    pub fn new() -> Result<Ilias<AuthProviderType>> {
        let config_dir = get_config_dir().expect("Could not get config directory");

        // Ensure the directory exists
        create_dir_all(&config_dir).expect("Could not create config directory");

        let config = Config::load().context("Failed to load configuration")?;

        let courses = config
            .courses
            .into_iter()
            .map(|raw_course| raw_course.into_course())
            .collect();

        let auth_provider =
            AuthProviderType::new(config_dir.clone(), config_dir.join("session.json"));

        Ok(Ilias {
            courses,
            client: None,
            auth_provider,
            base_path: config.path,
        })
    }

    async fn get_client(&mut self) -> Result<Client> {
        if let Some(client) = &self.client {
            Ok(client.clone())
        } else {
            let client = self
                .get_auth_provider()
                .authenticate(Policy::default())
                .await?;

            self.client = Some(client.clone());

            Ok(client)
        }
    }

    pub fn get_config(&self) -> Result<Config> {
        Config::load().context("Failed to load configuration")
    }

    pub fn get_auth_provider(&self) -> &AuthProviderType {
        &self.auth_provider
    }

    fn get_filtered_courses(&self, options: &ScrapeOptions) -> Vec<&Course> {
        let mut courses = self.courses.iter().collect();

        if let Some(course_id) = &options.course_id {
            courses = self
                .courses
                .iter()
                .filter(|&course| course.id.to_string().eq(course_id))
                .collect();
        }

        courses
    }

    pub async fn sync(&mut self, options: &ScrapeOptions) -> Result<()> {
        let client = self.get_client().await?;

        let courses = scrape_courses(&client, self.get_filtered_courses(options), options).await;
        for (course, root) in courses {
            course
                .sync(self.base_path.clone(), &client, root, options)
                .await;
        }

        Ok(())
    }

    pub fn local_tree(&self) -> Result<TreeConnectorNode<LocalTreeNode>> {
        Ok(connect_trees(
            self.courses
                .iter()
                .map(|course| course.tree_local(self.base_path.clone()))
                .collect(),
            String::from("local"),
        ))
    }

    pub async fn remote_tree(
        &mut self,
        options: &ScrapeOptions,
    ) -> Result<TreeConnectorNode<ScrapeObject>> {
        let client = self.get_client().await?;

        Ok(connect_trees(
            scrape_courses(&client, self.get_filtered_courses(options), options)
                .await
                .into_iter()
                .map(|(_, root)| root)
                .collect::<Vec<ScrapeObject>>(),
            String::from("ilias"),
        ))
    }
}
