use crate::responses::JoinResponse;
use crate::Db;
use futures::future;
use reqwest::{Error, Request, Response};
use serde_json::json;
use serde_value::Value;
use std::error::Error as ErrorTrait;
use tower::util::BoxService;
use tower::Service;
use tower::ServiceExt;
use tracing::{debug, error};
use url::Url;

#[derive(Clone)]
struct Limit(usize);

impl tower::retry::Policy<Request, Response, Error> for Limit {
    type Future = future::Ready<Self>;
    fn retry(&self, _: &Request, result: Result<&Response, &Error>) -> Option<Self::Future> {
        if result.is_err() && self.0 > 0 {
            Some(future::ready(Limit(self.0 - 1)))
        } else {
            None
        }
    }

    fn clone_request(&self, req: &reqwest::Request) -> Option<reqwest::Request> {
        req.try_clone()
    }
}

#[derive(Debug)]
pub struct Client {
    pub client: reqwest::Client,
    pub service: BoxService<Request, Response, Error>,
}

impl Client {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Client {
            service: Self::build_client(client.clone()),
            client: client,
        }
    }

    pub async fn call(&mut self, req: Request) -> Result<Response, Error> {
        self.service.ready().await?.call(req).await
    }

    pub async fn internal_update(&mut self, mut send_to: Url, key: &str, value: &Value) -> () {
        debug!("update other host '{}' of key '{}'", send_to, key);

        if send_to.cannot_be_a_base() {
            error!("invalid url '{}'", send_to.as_str());
            return;
        }
        send_to
            .path_segments_mut()
            .expect("checked this before")
            .push("_internal")
            .push("update")
            .push(key);

        match self.client.post(send_to).json(value).build() {
            Ok(request) => match self.call(request).await {
                Ok(_) => (),
                Err(e) => error!(%e),
            },
            Err(e) => error!(%e),
        }
    }

    pub async fn join(
        &mut self,
        me: Url,
        mut join_with: Url,
    ) -> Result<JoinResponse, Box<dyn ErrorTrait>> {
        debug!("joining host '{}' with my address '{}'", join_with, me);

        join_with
            .path_segments_mut()
            .map_err(|_| String::from("invalid url"))?
            .push("_internal")
            .push("join");

        let value = json!({"host": me.as_str()});

        let request = self.client.post(join_with).json(&value).build()?;
        let response: JoinResponse = self.call(request).await?.json().await?;
        Ok(response)
    }

    pub async fn sync(
        &mut self,
        mut sync_with: Url,
        code: &str,
    ) -> Result<Db, Box<dyn ErrorTrait>> {
        debug!("syncing with host '{}'", sync_with);

        sync_with
            .path_segments_mut()
            .map_err(|_| String::from("invalid url"))?
            .push("_internal")
            .push("sync");

        let value = json!({ "code": code });

        let request = self.client.post(sync_with).json(&value).build()?;
        let response: Db = self.call(request).await?.json().await?;
        Ok(response)
    }

    pub fn build_client(client: reqwest::Client) -> BoxService<Request, Response, Error> {
        let svc = tower::ServiceBuilder::new()
            // .rate_limit(100, Duration::new(10, 0)) // 100 requests every 10 seconds
            .retry(Limit(50))
            .service(tower::service_fn(move |req| client.execute(req)));

        // let mut req = Request::new(Method::POST, Url::parse("http://httpbin.org/post")?);
        // *req.body_mut() = Some(Body::from("the exact body that is sent"));

        // let res = svc.ready_and().await?.call(req).await?;
        BoxService::new(svc)
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        let client = self.client.clone();
        Client {
            service: Self::build_client(client.clone()),
            client: client,
        }
    }
}
