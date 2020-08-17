use std::fmt::{self, Debug};

pub mod app_id;
pub mod client;
mod host;
pub mod request;
pub mod response;

const HOST_FALLBACK_LIST: &[usize] = &[1, 2, 3];

/// Internal use type alias
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
// TODO: make an invariant that this _must_ be valid visible-ascii
pub struct ApiKey(pub String);

impl Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ApiKey").field(&"***").finish()
    }
}
