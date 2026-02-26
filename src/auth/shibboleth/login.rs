use anyhow::{Context, Ok, Result};
use reqwest::header::HeaderMap;
use reqwest::redirect::Policy;
use reqwest::Client;
use std::path::PathBuf;

use crate::auth::shibboleth::credentials::Credentials;
use crate::auth::shibboleth::session_store::{load_session, session_available, session_store};

use indicatif::{ProgressBar, ProgressStyle};

async fn verify(client: &Client) -> Result<()> {
    let r = client
        .get("https://ilias.studium.kit.edu/ilias.php?baseClass=ildashboardgui&cmdNode=9l:wa&cmdClass=ilPersonalProfileGUI")
        .send()
        .await
        .context("Failed to send verification request")?;

    let location = r
        .headers()
        .get("Location")
        .context("Failed to get Location header")?
        .to_str()
        .context("Failed to convert Location header to string")?;

    if location.contains("/login.php") {
        anyhow::bail!("Session invalid");
    }

    Ok(())
}

pub async fn login(
    credentials: Credentials,
    session_file: &PathBuf,
    policy: Policy,
) -> Result<Client> {
    if session_available(session_file) {
        println!("Session available, loading...");

        let possible_client = load_session(Policy::none(), false, session_file)?;

        if verify(&possible_client).await.is_ok() {
            println!("Session loaded successfully!");

            return load_session(policy, true, session_file);
        } else {
            println!("Session invalid, logging in...");
        }
    }

    let client = Client::builder()
        .cookie_store(true)
        .redirect(Policy::none())
        .build()
        .context("Could not build login client")?;

    let pb = ProgressBar::new(6);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap(),
    );

    // Step 1: initial login page
    pb.set_message("Requesting login page");

    let response = client
        .get("https://ilias.studium.kit.edu/shib_login.php")
        .send()
        .await
        .context("Failed to send initial login page request")?;

    pb.inc(1);

    // Step 2: SAML redirect
    pb.set_message("Following SAML redirect");

    let url = response
        .headers()
        .get("Location")
        .context("Failed to get SAML redirect Location header")?
        .to_str()
        .context("Failed to convert SAML redirect Location header to string")?;

    let r = client
        .get(url)
        .send()
        .await
        .context("Failed to follow SAML redirect")?;

    pb.inc(1);

    // Step 3: IDP login page
    pb.set_message("Loading identity provider");

    let idp_redirect_url = r
        .headers()
        .get("Location")
        .context("Failed to get identity provider redirect Location header")?
        .to_str()
        .context("Failed to convert identity provider redirect Location header to string")?;

    let url = format!("https://idp.scc.kit.edu{}", idp_redirect_url);
    let r = client
        .get(&url)
        .send()
        .await
        .context("Failed to load identity provider login page")?;
    let text = r
        .text()
        .await
        .context("Failed to read identity provider login page text")?;

    pb.inc(1);

    // Step 4: submit credentials
    pb.set_message("Submitting credentials");

    let csrf_token =
        extract_secret(&text, "csrf_token", "\" />").context("Could not extract CSRF token")?;
    let payload = format!(
        "csrf_token={}&j_username={}&j_password={}&_eventId_proceed=",
        csrf_token, credentials.username, credentials.password
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    let r = client
        .post(&url)
        .headers(headers)
        .body(payload)
        .send()
        .await?;
    let text = r.text().await?.clone();

    pb.inc(1);

    // Step 5: confirm SAML response
    pb.set_message("Confirming SAML response");
    let saml_response = extract_secret(&text, "SAMLResponse", "\"/>")
        .context("Could not extract SAML secret (Probably wrong username/password)")?;
    let payload = format!(
        "RelayState=https%3A%2F%2Filias.studium.kit.edu%2Fshib_login.php&SAMLResponse={}",
        urlencoding::encode(&saml_response)
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    client
        .post("https://ilias.studium.kit.edu/Shibboleth.sso/SAML2/POST")
        .headers(headers)
        .body(payload)
        .send()
        .await
        .context("Failed to confirm SAML response")?;

    pb.inc(1);

    // Step 6: finalize login
    pb.set_message("Finalizing login");

    let r = client
        .get("https://ilias.studium.kit.edu/shib_login.php")
        .send()
        .await
        .context("Failed to finalize login")?;

    pb.inc(1);

    pb.finish_with_message("Login successful ✔");

    session_store(r, session_file)?;
    load_session(policy, true, session_file)
}

fn extract_secret(text: &str, name: &str, end_str: &str) -> Result<String> {
    // get start index of saml response
    let saml_index_search_str = format!("<input type=\"hidden\" name=\"{}\" value=", name);
    let saml_index_search = text
        .find(&saml_index_search_str)
        .context("Failed to find SAML secret start index")?;
    let saml_index = saml_index_search + saml_index_search_str.len() + 1;

    // get end index of saml response
    let saml_index_end = text
        .get(saml_index..)
        .context("Failed to get SAML secret substring")?
        .find(end_str)
        .context("Failed to find SAML secret end index")?;

    let secret = text
        .get(saml_index..saml_index + saml_index_end)
        .context("Failed to extract SAML secret")?
        .to_string();

    Ok(secret)
}
