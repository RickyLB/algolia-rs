use crate::{
    app_id::{AppId, RefAppId},
    request::PartialUpdateQuery,
    response::{ObjectDeleteResponse, ObjectUpdateResponse},
    ApiKey, BoxError, HOST_FALLBACK_LIST, host::Host,
};
use rand::seq::SliceRandom;
use reqwest::header::{HeaderMap, HeaderValue};
use std::{fmt, future::Future, time::Duration};

// todo: make the ApiKey a `RefApiKey`
fn reqwest_client(app_id: &RefAppId, api_key: &ApiKey) -> reqwest::Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.append(
        "X-Algolia-Application-Id",
        HeaderValue::from_str(app_id.as_str()).expect("app_id wasn't valid as a header?"),
    );
    headers.append(
        "X-Algolia-API-Key",
        HeaderValue::from_str(&api_key.0).expect("api_key wasn't valid as a header?"),
    );

    reqwest::ClientBuilder::new()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .user_agent("ALGOLIA-RS")
        .build()
}

struct ObjectRoute<'a> {
    index_name: &'a str,
    object_id: &'a str,
    partial: bool,
}

impl fmt::Display for ObjectRoute<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "1/indexes/{}/{}", self.index_name, self.object_id)?;

        if self.partial {
            f.write_str("/partial")?;
        }

        Ok(())
    }
}

pub struct Client {
    client: reqwest::Client,
    application_id: AppId,
    api_key: ApiKey,
}

impl Client {
    pub fn new(application_id: AppId, api_key: ApiKey) -> Result<Self, BoxError> {
        let client = reqwest_client(&application_id, &api_key)?;

        Ok(Self {
            client,
            application_id,
            api_key,
        })
    }

    async fn retry_with<
        T: fmt::Display,
        O,
        Fut: Future<Output = Result<Option<O>, BoxError>>,
        Fn: FnMut(String) -> Fut,
    >(
        &self,
        route: T,
        mut f: Fn,
    ) -> Result<O, BoxError> {
        let mut fallback_order = HOST_FALLBACK_LIST.to_vec();
        fallback_order.shuffle(&mut rand::thread_rng());

        for backup_number in std::iter::once(0).chain(fallback_order.iter().copied()) {
            match f(format!(
                "https://{}/{}",
                Host::with_backup(&self.application_id, Some(backup_number)),
                &route,
            ))
            .await
            {
                Ok(None) => continue,
                Ok(Some(res)) => return Ok(res),
                Err(e) => return Err(e),
            }
        }

        todo!("what happens when we run out of timeout checks")
    }

    /// Add or replace an object with a given object ID.
    /// If the object does not exist, it will be created. If it already exists, it will be replaced.
    pub async fn add_or_update_object<T: serde::Serialize>(
        &self,
        index: &str,
        object_id: &str,
        body: &T,
    ) -> Result<ObjectUpdateResponse, BoxError> {
        self.retry_with(
            ObjectRoute {
                index_name: index,
                object_id,
                partial: false,
            },
            |url| async move {
                let resp = match self.client.put(&url).json(body).send().await {
                    Ok(resp) => resp,
                    Err(e) if e.is_timeout() => return Ok(None),
                    Err(e) => return Err(e.into()),
                };

                // presumably we should try again if the server messed up?
                if resp.status().is_server_error() {
                    return Ok(None);
                }

                if resp.status().is_client_error() {
                    todo!("What error for `400` for this route?")
                }

                Ok(Some(resp.json().await?))
            },
        )
        .await
    }

    /// Partially update an object.
    ///
    /// This creates a brand new record if it doesn’t already exist (and the createIfNotExists parameter isn’t set to false).
    ///
    /// You can pass any first-level attribute you want to add or replace within the record, but you can’t individually update nested attributes.
    /// If you specify a nested attribute, the engine treats it as a replacement of its first-level ancestor.
    pub async fn partially_update_object<T: serde::Serialize>(
        &self,
        index: &str,
        object_id: &str,
        body: &T,
        query: &PartialUpdateQuery,
    ) -> Result<ObjectUpdateResponse, BoxError> {
        self.retry_with(
            ObjectRoute {
                index_name: index,
                object_id,
                partial: true,
            },
            |url| async move {
                let resp = match self.client.post(&url).query(query).json(body).send().await {
                    Ok(resp) => resp,
                    Err(e) if e.is_timeout() => return Ok(None),
                    Err(e) => return Err(e.into()),
                };

                // presumably we should try again if the server messed up?
                if resp.status().is_server_error() {
                    return Ok(None);
                }

                if resp.status().is_client_error() {
                    todo!("What error for `400` for this route?")
                }

                // todo: figure out what happens when the update is ignored (due to not existing & `create_if_not_exists` being false)
                Ok(Some(resp.json().await?))
            },
        )
        .await
    }

    /// Delete an existing object from an index.
    pub async fn delete_object(
        &self,
        index: &str,
        object_id: &str,
    ) -> Result<ObjectDeleteResponse, BoxError> {
        self.retry_with(
            ObjectRoute {
                index_name: index,
                object_id,
                partial: false,
            },
            |url| async move {
                let resp = match self.client.delete(&url).send().await {
                    Ok(resp) => resp,
                    Err(e) if e.is_timeout() => return Ok(None),
                    Err(e) => return Err(e.into()),
                };

                // presumably we should try again if the server messed up?
                if resp.status().is_server_error() {
                    return Ok(None);
                }

                if resp.status().is_client_error() {
                    todo!("What error for `400` for this route?")
                }

                Ok(Some(resp.json().await?))
            },
        )
        .await
    }
}
