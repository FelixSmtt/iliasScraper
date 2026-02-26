use anyhow::{Context, Result};
use clap::error::ErrorKind;
use clap::{Parser, Subcommand, ValueEnum};
use colored::{Color, Colorize};
use inquire::Text;

use crate::auth::provider::{AuthProvider, AuthProviderFactory};
use crate::config::{get_config_dir, CourseConfig};
use crate::ilias::Ilias;
use crate::scraper::scrape_options::ScrapeOptions;
use crate::tree::printable::Printable;

#[derive(Parser)]
#[command(name = "ilias")]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum TreeSource {
    Ilias,
    Local,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fetch a course tree
    Tree {
        /// Source to scrape from
        #[arg(value_enum)]
        source: TreeSource,

        /// Identifier course if only one course should be scraped (e.g. 12345678)
        #[arg(long)]
        course: Option<String>,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,

        /// Include videos
        #[arg(long)]
        videos: bool,
    },

    /// Sync courses to local storage
    Sync {
        /// Identifier course if only one course should be synced (e.g. 12345678)
        #[arg(long)]
        course: Option<String>,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,

        /// Include videos
        #[arg(long)]
        videos: bool,
    },

    /// Start interactive command line interface
    Cli,

    /// Show or edit configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show configuration
    Show,

    /// Manage configured courses
    Course {
        #[command(subcommand)]
        command: ConfigCoursesCommands,
    },

    /// Show path to configuration file
    Path,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCoursesCommands {
    /// List all courses
    Ls,

    Add {
        /// Course name
        name: String,

        /// Course ID
        id: u32,
    },

    Remove {
        /// Course ID
        id: u32,
    },

    Rename {
        /// Course ID
        id: u32,

        /// New name
        name: String,
    },

    UpdateId {
        /// Old course ID
        old_id: u32,

        /// New course ID
        new_id: u32,
    },
}

#[derive(Debug)]
enum InteractiveCommand {
    Exit,
    ClapCommand(Commands),
}

impl<AuthProviderType: AuthProvider + AuthProviderFactory> Ilias<AuthProviderType> {
    fn process_clap_error(e: clap::Error) -> String {
        // Post-process unknown command errors
        let mut msg = e.to_string();
        msg = msg.replace("ilias ", "");
        msg = msg.replace("'--help'", "help");
        msg = msg.replace("subcommand", "command");

        msg
    }

    fn parse_interactive_command(input: &str) -> anyhow::Result<InteractiveCommand> {
        let input = input.trim();

        // interactive-only commands
        match input {
            "exit" | "quit" => return Ok(InteractiveCommand::Exit),
            "" => anyhow::bail!(
                "error:: empty command\n\nUsage: [COMMAND]\n\nFor more information, try help."
            ),
            _ => {}
        }

        // clap-based commands
        let argv = std::iter::once("ilias-scraper")
            .chain(input.split_whitespace())
            .collect::<Vec<_>>();

        let cli = Cli::try_parse_from(argv).map_err(|e| {
            match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    // Just return the help text as output, no "Error parsing command"
                    anyhow::anyhow!("{}", Self::process_clap_error(e))
                }
                _ => {
                    anyhow::anyhow!("{}", Self::process_clap_error(e))
                }
            }
        })?;

        match cli.command {
            Some(Commands::Cli) => {
                anyhow::bail!("`cli` is not allowed inside interactive mode")
            }
            Some(cmd) => Ok(InteractiveCommand::ClapCommand(cmd)),
            None => anyhow::bail!("No command provided"),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let input = Text::new("")
                // .with_autocomplete(ClapAutocomplete::new())
                .prompt()
                .context("Failed to read input")?;

            let command = match Self::parse_interactive_command(&input) {
                Ok(cmd) => cmd,
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            };

            match command {
                InteractiveCommand::Exit => break,
                InteractiveCommand::ClapCommand(cmd) => {
                    self.execute_command(&cmd).await?;
                }
            }
        }

        Ok(())
    }

    async fn execute_command(&mut self, command: &Commands) -> Result<()> {
        let auth_provider: &AuthProviderType = self.get_auth_provider();

        match command {
            Commands::Tree {
                source,
                course,
                verbose,
                videos,
            } => {
                let scrape_options = ScrapeOptions {
                    videos: *videos,
                    course_id: course.clone(),
                    verbose: *verbose,
                    auth: (*auth_provider).arc_clone(),
                };

                match source {
                    TreeSource::Ilias => {
                        self.remote_tree(&scrape_options).await?.print();
                    }
                    TreeSource::Local => {
                        self.local_tree()?.print();
                    }
                }
            }
            Commands::Sync {
                course,
                verbose,
                videos,
            } => {
                let scrape_options = ScrapeOptions {
                    videos: *videos,
                    course_id: course.clone(),
                    verbose: *verbose,
                    auth: (*auth_provider).arc_clone(),
                };

                self.sync(&scrape_options).await?;
            }
            Commands::Config { command } => match command {
                ConfigCommands::Path => {
                    let config_path = get_config_dir()?.join("config.json");
                    println!("Configuration file path: {}", config_path.display());
                }
                ConfigCommands::Show => {
                    let config = self.get_config()?;
                    println!("Current configuration:\n{}", config);
                }
                ConfigCommands::Course { command } => match command {
                    ConfigCoursesCommands::Ls => {
                        println!("Configured courses:");
                        for course in &self.courses {
                            println!(
                                "{} - {}",
                                course.name.color(Color::Green),
                                course.id.to_string().color(Color::Blue)
                            );
                        }
                    }
                    ConfigCoursesCommands::Add { name, id } => {
                        let mut config = self.get_config()?;

                        let course = CourseConfig {
                            name: name.clone(),
                            id: *id,
                        };
                        config.add_course(course)?;

                        config.save()?;
                        println!(
                            "Added course: {} ({})",
                            name.color(Color::Green),
                            id.to_string().color(Color::Blue)
                        );
                    }
                    ConfigCoursesCommands::Remove { id } => {
                        let mut config = self.get_config()?;

                        config.remove_course(id)?;

                        config.save()?;
                        println!(
                            "Removed course with ID: {}",
                            id.to_string().color(Color::Blue)
                        );
                    }
                    ConfigCoursesCommands::Rename { id, name } => {
                        let mut config = self.get_config()?;

                        let course = config.get_course(id)?;

                        course.name = name.clone();

                        config.save()?;
                        println!(
                            "Renamed course ID {} to {}",
                            id.to_string().color(Color::Blue),
                            name.color(Color::Green)
                        );
                    }
                    ConfigCoursesCommands::UpdateId { old_id, new_id } => {
                        let mut config = self.get_config()?;

                        let course = config.get_course(old_id)?;

                        course.id = *new_id;

                        config.save()?;
                        println!(
                            "Updated course ID from {} to {}",
                            old_id.to_string().color(Color::Blue),
                            new_id.to_string().color(Color::Blue)
                        );
                    }
                },
            },
            _ => {
                println!("Unsupported command in non-interactive mode.");
            }
        }

        Ok(())
    }

    pub async fn handle_cli(&mut self) -> Result<()> {
        let cli = Cli::parse();

        match &cli.command {
            Some(Commands::Cli) => {
                self.execute_command(&Commands::Cli).await?;
            }
            None => {
                self.run().await?;
            }
            Some(command) => {
                self.execute_command(command).await?;
            }
        }

        Ok(())
    }
}
