extern crate core;

use crate::auth::shibboleth::provider::ShibbolethAuthProvider;
use ilias::Ilias;

mod auth;
mod cli;
mod config;
mod course;
pub(crate) mod ilias;
mod scraper;
mod tree;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut ilias = Ilias::<ShibbolethAuthProvider>::new()?;

    ilias.handle_cli().await?;

    Ok(())
}
