use std::fmt::{self, Debug};

mod app_id;
mod client;
mod host;
pub mod request;
pub mod response;
pub mod model;

pub use app_id::{AppId, RefAppId};
pub use client::Client;

const HOST_FALLBACK_LIST: &[usize] = &[1, 2, 3];

/// Internal use type alias
pub type BoxError = Box<dyn std::error::Error + 'static + Send + Sync>;

#[derive(Clone)]
// TODO: make an invariant that this _must_ be valid visible-ascii
pub struct ApiKey(pub String);

impl Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ApiKey").field(&"***").finish()
    }
}
