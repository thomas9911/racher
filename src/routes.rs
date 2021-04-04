use crate::Db;

use std::collections::HashSet;
use std::iter::FromIterator;

use serde_json::json;
use serde_value::Value;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

use crate::MAX_FILE_SIZE;

fn ok_reponse() -> warp::reply::Json {
    warp::reply::json(&json!({"status": "ok"}))
}

fn data_response(value: &Value) -> warp::reply::Json {
    warp::reply::json(&json!({ "data": value }))
}

fn delete_response(value: bool) -> warp::reply::Json {
    warp::reply::json(&json!({ "deleted": value }))
}

pub fn getter(cache: Db) -> BoxedFilter<(impl Reply,)> {
    warp::path!("get" / String)
        .map(move |name| match cache.get(&name) {
            Some(x) => data_response(x.value()),
            None => data_response(&Value::Unit),
        })
        .boxed()
}

pub fn deleter(cache: Db) -> BoxedFilter<(impl Reply,)> {
    warp::path!("del" / String)
        .map(move |name| match cache.remove(&name) {
            Some(_) => delete_response(true),
            None => delete_response(false),
        })
        .boxed()
}

pub fn setter(cache: Db) -> BoxedFilter<(impl Reply,)> {
    warp::path!("set" / String)
        .and(warp::body::content_length_limit(MAX_FILE_SIZE))
        .and(warp::body::json())
        .map(move |name, simple_map: Value| {
            cache.insert(name, simple_map);
            ok_reponse()
        })
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
