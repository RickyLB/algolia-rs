/// Internal use type alias
pub type BoxError = Box<dyn std::error::Error + 'static + Send + Sync>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Error occurred while trying to setup the Client
    #[error("error initializing client: {0}")]
    Configuration(#[source] BoxError),

    #[error("request timed out")]
    Timeout,

    #[error("index `{0}` not found")]
    IndexNotFound(String),

    #[error("decode error: {0}")]
    DecodeError(#[source] BoxError),

    /// Error occurred with a request
    #[error("request error: {0}")]
    RequestError(#[source] BoxError),
}

impl Error {
    pub(crate) async fn bad_request(resp: reqwest::Response) -> Self {
        match resp.json::<BadRequestError>().await {
            Ok(e) => Self::RequestError(Box::new(e)),
            Err(e) => Self::RequestError(Box::new(e)),
        }
    }
}

#[derive(serde::Deserialize, thiserror::Error, Debug)]
#[error("bad request: {message}")]
pub struct BadRequestError {
    message: String,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
