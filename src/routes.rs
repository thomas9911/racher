use crate::config::RuntimeConfigArc;
use crate::transport;
use crate::transport::Message;
use crate::Db;

use serde_json::json;
use serde_value::Value;
use std::collections::HashSet;
use std::convert::Infallible;
use std::iter::FromIterator;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

pub mod internal;
pub mod utils;

use crate::MAX_FILE_SIZE;

#[cfg(feature = "dashboard")]
static DASHBOARD: &'static str = include_str!("../web/dist/index.html");

pub(crate) fn ok_reponse() -> warp::reply::Json {
    warp::reply::json(&json!({"status": "ok"}))
}

fn data_response(value: &Value) -> warp::reply::Json {
    warp::reply::json(&json!({ "data": value }))
}

fn delete_response(value: bool) -> warp::reply::Json {
    warp::reply::json(&json!({ "deleted": value }))
}

async fn inner_setter(
    name: String,
    simple_map: Value,
    cache: Db,
    tx: transport::Sender,
) -> Result<impl warp::Reply, Infallible> {
    cache.insert(name.clone(), simple_map.clone());
    // ignore the error, this will only return if no-one is listening.
    tx.send(Message::Created(name, simple_map)).ok();
    Ok::<_, Infallible>(ok_reponse())
}

pub fn getter(cache: Db) -> BoxedFilter<(impl Reply,)> {
    warp::path!("get" / String)
        .map(move |name| match cache.get(&name) {
            Some(x) => data_response(x.value()),
            None => data_response(&Value::Unit),
        })
        .boxed()
}

pub fn deleter(cache: Db, tx: transport::Sender) -> BoxedFilter<(impl Reply,)> {
    warp::path!("del" / String)
        .map(move |name| {
            let deleted = match cache.remove(&name) {
                Some(_) => true,
                None => false,
            };
            tx.send(Message::Deleted(name)).ok();
            delete_response(deleted)
        })
        .boxed()
}

pub fn setter(cache: Db, tx: transport::Sender) -> BoxedFilter<(impl Reply,)> {
    warp::path!("set" / String)
        .and(warp::body::content_length_limit(MAX_FILE_SIZE))
        .and(warp::body::json())
        .and(utils::move_object(cache))
        .and(utils::move_object(tx))
        .and_then(inner_setter)
        .boxed()
}

pub fn purge(cache: Db) -> BoxedFilter<(impl Reply,)> {
    warp::path!("purge")
        .map(move || {
            cache.clear();
            warp::reply::json(&json!({ "purged": true }))
        })
        .boxed()
}

pub fn ping() -> BoxedFilter<(impl Reply,)> {
    warp::path!("ping")
        .map(|| warp::reply::json(&json!({ "pong": true })))
        .boxed()
}

pub fn keys(cache: Db) -> BoxedFilter<(impl Reply,)> {
    warp::path!("keys")
        .map(move || {
            let keys: HashSet<String> =
                HashSet::from_iter(cache.iter().map(|item| item.key().clone()));
            warp::reply::json(&json!({ "keys": keys }))
        })
        .boxed()
}

pub fn internal(cache: Db, cfg: RuntimeConfigArc) -> BoxedFilter<(impl Reply,)> {
    warp::path("_internal")
        .and(
            internal::join(cfg.clone())
                .or(internal::sync(cache.clone(), cfg.clone()))
                .or(internal::update(cache.clone()))
                .or(internal::fanout(cfg.clone()))
                .or(internal::config(cfg.clone())),
        )
        .boxed()
}

#[cfg(feature = "dashboard")]
pub fn web() -> BoxedFilter<(impl Reply,)> {
    warp::path!("dashboard" / ..)
        .map(|| {
            warp::reply::html(DASHBOARD)
        })
        .boxed()
}