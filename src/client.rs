use crate::{
    app_id::{AppId, RefAppId},
    filter::Filterable,
    host::Host,
    model::task::{TaskId, TaskStatus},
    request::{BatchWriteRequests, PartialUpdateQuery, SearchQuery, SetSettings},
    response::{
        BatchWriteResponse, ObjectDeleteResponse, ObjectUpdateResponse, SearchResponse,
        SettingsUpdateResponse, TaskStatusResponse,
    },
    ApiKey, Error, Result, HOST_FALLBACK_LIST,
};
use rand::seq::SliceRandom;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use serde::de::DeserializeOwned;
use std::{fmt, future::Future, time::Duration};

// todo: make the ApiKey a `RefApiKey`
fn reqwest_client(app_id: &RefAppId, api_key: &ApiKey) -> reqwest::Result<reqwest::Client> {
    let mut headers = HeaderMap::new();

    headers.append(
        "X-Algolia-Application-Id",
        HeaderValue::from_str(app_id.as_str()).expect("app_id wasn't valid as a header?"),
    );

    let mut api_key_header =
        HeaderValue::from_str(&api_key.0).expect("api_key wasn't valid as a header?");

    api_key_header.set_sensitive(true);

    headers.append("X-Algolia-API-Key", api_key_header);

    reqwest::ClientBuilder::new()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .user_agent("ALGOLIA-RS")
        .build()
}

#[derive(Copy, Clone)]
enum IndexRouteKind {
    Query,
    Settings,
    Batch,
}

impl fmt::Display for IndexRouteKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query => f.write_str("query"),
            Self::Settings => f.write_str("settings"),
            Self::Batch => f.write_str("batch"),
        }
    }
}

struct IndexRoute<'a> {
    index_name: &'a str,
    kind: Option<IndexRouteKind>,
}

impl fmt::Display for IndexRoute<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "indexes/{}", self.index_name)?;

        if let Some(kind) = self.kind {
            write!(f, "/{}", kind)?;
        }

        Ok(())
    }
}

struct ObjectRoute<'a> {
    index_name: &'a str,
    object_id: &'a str,
    partial: bool,
}

impl fmt::Display for ObjectRoute<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "indexes/{}/{}", self.index_name, self.object_id)?;

        if self.partial {
            f.write_str("/partial")?;
        }

        Ok(())
    }
}

struct TaskRoute<'a> {
    index_name: &'a str,
    task_id: TaskId,
}

impl fmt::Display for TaskRoute<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "indexes/{}/task/{}", self.index_name, self.task_id.0)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    client: reqwest::Client,
    application_id: AppId,
    api_key: ApiKey,
}

async fn decode<T: DeserializeOwned>(resp: reqwest::Response) -> Result<Option<T>, Error> {
    resp.json()
        .await
        .map(Some)
        .map_err(|it| Error::DecodeError(Box::new(it)))
}

macro_rules! unwrap_ret {
    ($e:expr) => {
        match $e {
            Ok(Some(x)) => x,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        }
    };
}

async fn check_response(
    resp: reqwest::Result<reqwest::Response>,
    index: Option<&str>,
) -> Result<Option<reqwest::Response>, Error> {
    let resp = match resp {
        Ok(resp) => resp,
        Err(e) if e.is_timeout() => return Ok(None),
        Err(e) => return Err(Error::RequestError(Box::new(e))),
    };

    // presumably we should try again if the server messed up?
    if resp.status().is_server_error() {
        return Ok(None);
    }

    if let Some(index) = index {
        if resp.status() == StatusCode::NOT_FOUND {
            return Err(Error::IndexNotFound(index.to_owned()));
        }
    }

    if resp.status() == StatusCode::BAD_REQUEST {
        return Err(Error::bad_request(resp).await);
    }

    if resp.status().is_client_error() {
        return Err(Error::unexpected(resp).await);
    }

    Ok(Some(resp))
}

impl Client {
    pub fn new(application_id: AppId, api_key: ApiKey) -> Result<Self> {
        let client = reqwest_client(&application_id, &api_key)
            .map_err(|it| Error::Configuration(Box::new(it)))?;

        Ok(Self {
            client,
            application_id,
            api_key,
        })
    }

    async fn retry_with<
        T: fmt::Display,
        O,
        Fut: Future<Output = Result<Option<O>>>,
        Fn: FnMut(String) -> Fut,
    >(
        &self,
        route: T,
        mut f: Fn,
    ) -> Result<O> {
        let mut fallback_order = HOST_FALLBACK_LIST.to_vec();
        fallback_order.shuffle(&mut rand::thread_rng());

        for backup_number in std::iter::once(0).chain(fallback_order.iter().copied()) {
            match f(format!(
                "https://{}/1/{}",
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

        Err(Error::Timeout)
    }

    pub async fn batch(&self, index: &str, req: &BatchWriteRequests) -> Result<BatchWriteResponse> {
        self.retry_with(
            IndexRoute {
                index_name: index,
                kind: Some(IndexRouteKind::Batch),
            },
            |url| async move {
                let resp = unwrap_ret!(
                    check_response(self.client.post(&url).json(req).send().await, None).await
                );

                decode(resp).await
            },
        )
        .await
    }

    pub async fn set_settings(
        &self,
        index: &str,
        req: &SetSettings,
    ) -> Result<SettingsUpdateResponse> {
        self.retry_with(
            IndexRoute {
                index_name: index,
                kind: Some(IndexRouteKind::Settings),
            },
            |url| async move {
                let resp = unwrap_ret!(
                    check_response(self.client.put(&url).json(req).send().await, None).await
                );

                decode(resp).await
            },
        )
        .await
    }

    pub async fn task_status(&self, index: &str, task_id: TaskId) -> Result<TaskStatus> {
        self.retry_with(
            TaskRoute {
                index_name: index,
                task_id,
            },
            |url| async move {
                let resp =
                    unwrap_ret!(check_response(self.client.get(&url).send().await, None).await);

                decode::<TaskStatusResponse>(resp)
                    .await
                    .map(|it| it.map(|it| it.status))
            },
        )
        .await
    }

    #[inline(always)]
    pub async fn search<T: DeserializeOwned, Q: Filterable>(
        &self,
        index: &str,
        request: SearchQuery<'_, Q>,
    ) -> Result<SearchResponse<T>> {
        let request = serde_urlencoded::to_string(request).expect("request should be serializable");
        let request = &*request;

        self.search_inner(index, request).await
    }

    // Wrapped by `search`. But removes of the generic arguments
    // to avoid more instantiations of this function than needed.
    async fn search_inner<T: DeserializeOwned>(
        &self,
        index: &str,
        request: &str,
    ) -> Result<SearchResponse<T>> {
        #[derive(serde::Serialize)]
        struct Request<'a> {
            params: &'a str,
        }

        self.retry_with(
            IndexRoute {
                index_name: index,
                kind: Some(IndexRouteKind::Query),
            },
            |url| async move {
                let mut req = self.client.post(&url);

                req = req.json(&Request { params: request });

                let resp = unwrap_ret!(check_response(req.send().await, Some(index)).await);

                decode(resp).await
            },
        )
        .await
    }

    /// Add or replace an object with a given object ID.
    /// If the object does not exist, it will be created. If it already exists, it will be replaced.
    pub async fn add_or_update_object<T: serde::Serialize>(
        &self,
        index: &str,
        object_id: &str,
        body: &T,
    ) -> Result<ObjectUpdateResponse> {
        self.retry_with(
            ObjectRoute {
                index_name: index,
                object_id,
                partial: false,
            },
            |url| async move {
                let resp = unwrap_ret!(
                    check_response(self.client.put(&url).json(body).send().await, None).await
                );

                decode(resp).await
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
    ) -> Result<ObjectUpdateResponse> {
        self.retry_with(
            ObjectRoute {
                index_name: index,
                object_id,
                partial: true,
            },
            |url| async move {
                let resp = unwrap_ret!(
                    check_response(
                        self.client.post(&url).query(query).json(body).send().await,
                        None
                    )
                    .await
                );

                decode(resp).await
            },
        )
        .await
    }

    /// Delete an existing object from an index.
    pub async fn delete_object(
        &self,
        index: &str,
        object_id: &str,
    ) -> Result<ObjectDeleteResponse> {
        self.retry_with(
            ObjectRoute {
                index_name: index,
                object_id,
                partial: false,
            },
            |url| async move {
                let resp =
                    unwrap_ret!(check_response(self.client.delete(&url).send().await, None).await);

                decode(resp).await
            },
        )
        .await
    }
}
