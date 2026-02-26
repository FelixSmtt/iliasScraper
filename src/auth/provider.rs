use std::{fmt::Debug, path::PathBuf, sync::Arc};

use anyhow::Result;
use reqwest::{redirect::Policy, Client};

#[async_trait::async_trait]
pub trait AuthProvider: Send + Sync + Debug {
    async fn authenticate(&self, policy: Policy) -> Result<Client>;
    async fn authed_client(&self, policy: Policy) -> Result<Client>;

    fn arc_clone(&self) -> Arc<dyn AuthProvider>;
}

pub trait AuthProviderFactory: AuthProvider + Sized {
    fn new(config_dir: PathBuf, session_file: PathBuf) -> Self;
}
