use std::error::Error as StdError;

use clap::Command;
use clap::CommandFactory;
use inquire::Autocomplete;

use crate::cli::ilias_cli::Cli;

#[derive(Clone)]
#[allow(dead_code)]
pub struct ClapAutocomplete {
    root: Command,
}

#[allow(dead_code)]
impl ClapAutocomplete {
    pub fn new() -> Self {
        Self {
            root: Cli::command(),
        }
    }
}

fn find_command<'a>(mut cmd: &'a Command, tokens: &[&str]) -> &'a Command {
    for token in tokens {
        if let Some(sub) = cmd.find_subcommand(token) {
            cmd = sub;
        } else {
            break;
        }
    }
    cmd
}

impl Autocomplete for ClapAutocomplete {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> Result<Vec<std::string::String>, Box<dyn StdError + Send + Sync + 'static>> {
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let last = tokens.last().copied().unwrap_or("");

        let cmd = find_command(&self.root, &tokens[..tokens.len().saturating_sub(1)]);

        let mut suggestions = Vec::new();

        // subcommands
        for sub in cmd.get_subcommands() {
            let name = sub.get_name();
            if name.starts_with(last) {
                suggestions.push(name.to_string());
            }
        }

        // flags
        for arg in cmd.get_arguments() {
            if let Some(long) = arg.get_long() {
                let flag = format!("--{long}");
                if flag.starts_with(last) {
                    suggestions.push(flag);
                }
            }

            if let Some(short) = arg.get_short() {
                let flag = format!("-{short}");
                if flag.starts_with(last) {
                    suggestions.push(flag);
                }
            }

            // value_enum support
            if arg.is_positional() {
                let values = arg.get_possible_values();

                for v in values {
                    if v.get_name().starts_with(last) {
                        suggestions.push(v.get_name().to_string());
                    }
                }
            }
        }

        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        input: &str,
        suggestion: Option<String>,
    ) -> Result<std::option::Option<std::string::String>, Box<dyn StdError + Send + Sync + 'static>>
    {
        let mut parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        if !parts.is_empty() {
            parts.pop();
        }
        parts.push(suggestion.unwrap_or_default());

        Ok(Some(parts.join(" ")))
    }
}
