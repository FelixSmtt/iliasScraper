use std::sync::Arc;

use crate::auth::provider::AuthProvider;

#[derive(Clone)]
pub struct ScrapeOptions {
    pub videos: bool,
    pub course_id: Option<String>,
    pub verbose: bool,

    pub auth: Arc<dyn AuthProvider>,
}
