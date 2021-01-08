mod app_id;
mod client;
pub mod error;
pub mod filter;
mod host;
mod key;
pub mod model;
pub mod request;
pub mod response;

pub use app_id::{AppId, RefAppId};
pub use client::Client;
pub use error::{BoxError, Error, Result};
pub use key::ApiKey;

const HOST_FALLBACK_LIST: &[usize] = &[1, 2, 3];
