use std::fmt::{self, Debug};

mod app_id;
mod client;
pub mod error;
pub mod filter;
mod host;
pub mod model;
pub mod request;
pub mod response;

pub use app_id::{AppId, RefAppId};
pub use client::Client;
pub use error::{BoxError, Error, Result};

const HOST_FALLBACK_LIST: &[usize] = &[1, 2, 3];

#[derive(Clone)]
// TODO: make an invariant that this _must_ be valid visible-ascii
pub struct ApiKey(pub String);

impl Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ApiKey").field(&"***").finish()
    }
}
