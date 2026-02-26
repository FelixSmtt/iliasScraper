use anyhow::{Context, Result};
use colored::{Color, Colorize};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf};

use crate::course::Course;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub path: PathBuf,
    pub courses: Vec<CourseConfig>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let dir = get_config_dir().context("Could not get config directory")?;
        let file =
            std::fs::File::open(dir.join("config.json")).context("Could not open config file.")?;
        let config: Config =
            serde_json::from_reader(file).context("Failed to parse config.json")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let file = std::fs::File::create(get_config_dir()?.join("config.json"))?;
        serde_json::to_writer_pretty(file, &self)?;

        Ok(())
    }

    pub fn add_course(&mut self, course: CourseConfig) -> Result<()> {
        if self.courses.iter().any(|c| c.id == course.id) {
            println!(
                "Course with ID {} already exists.",
                course.id.to_string().color(Color::Blue)
            );
            return Ok(());
        }

        self.courses.push(course);
        Ok(())
    }

    pub fn remove_course(&mut self, course_id: &u32) -> Result<()> {
        if let Some(pos) = self.courses.iter().position(|c| &c.id == course_id) {
            self.courses.remove(pos);
            Ok(())
        } else {
            anyhow::bail!("Course with ID {} not found.", course_id);
        }
    }

    pub fn get_course(&mut self, course_id: &u32) -> Result<&mut CourseConfig> {
        if let Some(course) = self.courses.iter_mut().find(|c| &c.id == course_id) {
            Ok(course)
        } else {
            anyhow::bail!("Course with ID {} not found.", course_id);
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Path: {}", self.path.display())?;
        writeln!(f, "Courses:")?;

        for course in &self.courses {
            writeln!(f, "  - {} ({})", course.name, course.id)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseConfig {
    pub name: String,
    pub id: u32,
}

impl CourseConfig {
    pub fn into_course(self) -> Course {
        Course::new(self.name, self.id)
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    let qualifier = "";
    let organization = "";
    let application = "ilias";

    // Get the config directory
    let proj_dirs = ProjectDirs::from(qualifier, organization, application)
        .context("Could not determine config directory")?;

    let mut config_dir = proj_dirs.config_dir().to_path_buf();
    // if on Windows remove /config
    if config_dir
        .file_name()
        .context("Could not get config directory file name")?
        == "config"
    {
        config_dir = config_dir
            .parent()
            .context("Could not get parent of config directory")?
            .to_path_buf()
    }

    Ok(config_dir)
}
