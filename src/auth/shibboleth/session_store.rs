use anyhow::{Context, Ok, Result};
use reqwest::cookie::Jar;
use reqwest::redirect::Policy;
use reqwest::Url;
use reqwest::{Client, Response};
use serde_json::to_string;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn session_store(r: Response, session_file: &PathBuf) -> Result<()> {
    let cookies: HashMap<String, String> = r
        .headers()
        .get_all("Set-Cookie")
        .iter()
        .filter_map(|value| {
            let parts: Vec<&str> = value.to_str().ok()?.split(';').collect();
            let name_value: Vec<&str> = parts[0].split('=').collect();
            if name_value.len() == 2 {
                Some((
                    name_value[0].trim().to_string(),
                    name_value[1].trim().to_string(),
                ))
            } else {
                None
            }
        })
        .collect();

    let mut file = File::create(session_file).context("Could not create session file")?;
    file.write_all(
        to_string(&cookies)
            .context("Could not serialize cookies")?
            .as_bytes(),
    )
    .context("Could not write to session file")?;

    Ok(())
}

fn session_load_json(session_file: &PathBuf) -> Result<HashMap<String, String>> {
    let file = File::open(session_file).context("Could not open session file")?;

    let cookies: HashMap<String, String> =
        serde_json::from_reader(file).context("Could not read session file")?;

    Ok(cookies)
}

pub fn load_session(
    redirect: Policy,
    print_cookies: bool,
    session_file: &PathBuf,
) -> Result<Client> {
    let url = Url::parse("https://ilias.studium.kit.edu").unwrap();
    let cookies = session_load_json(session_file)?;
    let mut cookie_str: String = String::new();

    for (key, value) in cookies {
        cookie_str.push_str(&format!("{}={};", key, value));
    }

    let jar = Jar::default();
    if print_cookies {
        println!("Cookies loaded: {}", cookie_str);
    }
    jar.add_cookie_str(&cookie_str, &url);

    let client = Client::builder()
        .redirect(redirect)
        .cookie_store(true)
        .cookie_provider(jar.into())
        .build()
        .unwrap();

    Ok(client)
}

pub fn session_available(session_file: &PathBuf) -> bool {
    let file = File::open(session_file);
    file.is_ok()
}
