use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use reqwest::{redirect::Policy, Client};

use crate::auth::{
    provider::{AuthProvider, AuthProviderFactory},
    shibboleth::{credentials::Credentials, login::login, session_store::load_session},
};

#[derive(Clone, Debug)]
pub struct ShibbolethAuthProvider {
    credentials: Credentials,
    session_file: PathBuf,
}

#[async_trait::async_trait]
impl AuthProvider for ShibbolethAuthProvider {
    async fn authenticate(&self, policy: Policy) -> Result<Client> {
        login(self.credentials.clone(), &self.session_file, policy)
            .await
            .context("Failed to authenticate with Shibboleth")
    }

    async fn authed_client(&self, policy: Policy) -> Result<Client> {
        load_session(policy, false, &self.session_file)
    }

    fn arc_clone(&self) -> Arc<dyn AuthProvider> {
        Arc::new(Self {
            credentials: self.credentials.clone(),
            session_file: self.session_file.clone(),
        })
    }
}

impl AuthProviderFactory for ShibbolethAuthProvider {
    fn new(config_dir: PathBuf, session_file: PathBuf) -> Self {
        let credentials = Credentials::new(config_dir.join("credentials.json"));

        ShibbolethAuthProvider {
            credentials,
            session_file,
        }
    }
}
