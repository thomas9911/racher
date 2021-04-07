#![warn(rust_2018_idioms)]

pub mod cli;
pub mod config;
pub mod responses;
pub mod routes;

use std::sync::Arc;

use dashmap::DashMap;
use serde_value::Value;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

pub type Db = Arc<DashMap<String, Value>>;

pub const MAX_FILE_SIZE: u64 = 1024 * 1024 * 32;

pub fn create_api(arc_cache: Db) -> BoxedFilter<(impl Reply,)> {
    warp::post()
        .and(
            routes::setter(arc_cache.clone())
                .or(routes::getter(arc_cache.clone()))
                .or(routes::keys(arc_cache.clone()))
                .or(routes::purge(arc_cache.clone()))
                .or(routes::ping())
                .or(routes::deleter(arc_cache.clone())),
        )
        .boxed()
}

#[tokio::test]
async fn get_not_existing() {
    let map = DashMap::new();

    let cache = Arc::new(map);
    let filter = create_api(cache);

    let value = warp::test::request()
        .method("POST")
        .path("/get/not-existing")
        .reply(&filter)
        .await
        .into_body();

    let value: responses::GetResponse = serde_json::from_slice(&value).unwrap();
    let expected = responses::GetResponse { data: Value::Unit };
    assert_eq!(value, expected);
}

#[tokio::test]
async fn get_existing() {
    let map = DashMap::new();
    map.insert(String::from("testing"), Value::Bool(true));

    let cache = Arc::new(map);
    let filter = create_api(cache);

    let value = warp::test::request()
        .method("POST")
        .path("/get/testing")
        .reply(&filter)
        .await
        .into_body();

    let value: responses::GetResponse = serde_json::from_slice(&value).unwrap();
    let expected = responses::GetResponse {
        data: Value::Bool(true),
    };
    assert_eq!(value, expected);
}

#[tokio::test]
async fn set() {
    let map = DashMap::new();

    let cache = Arc::new(map);
    let filter = create_api(cache.clone());

    let value = warp::test::request()
        .method("POST")
        .path("/set/testing")
        .json(&Value::I64(123))
        .reply(&filter)
        .await
        .into_body();

    let value: responses::SetResponse = serde_json::from_slice(&value).unwrap();
    assert_eq!(
        value,
        responses::SetResponse {
            status: Value::String("ok".into())
        }
    );

    let value = warp::test::request()
        .method("POST")
        .path("/get/testing")
        .reply(&filter)
        .await
        .into_body();

    let value: responses::GetResponse = serde_json::from_slice(&value).unwrap();
    let expected = responses::GetResponse {
        data: Value::U64(123),
    };
    assert_eq!(value, expected);

    // has been set in the cache
    assert_eq!(&Value::U64(123), cache.get("testing").unwrap().value());
}

#[tokio::test]
async fn keys() {
    let map = DashMap::new();
    map.insert(String::from("testing"), Value::Bool(true));
    map.insert(String::from("another"), Value::Bool(true));

    let cache = Arc::new(map);
    let filter = create_api(cache.clone());

    let value = warp::test::request()
        .method("POST")
        .path("/keys")
        .reply(&filter)
        .await
        .into_body();

    let value: responses::KeysResponse = serde_json::from_slice(&value).unwrap();
    let mut keys = value.keys;
    keys.sort();

    assert_eq!(keys, vec!["another", "testing"]);
}

#[tokio::test]
async fn delete_existing() {
    let map = DashMap::new();
    map.insert(String::from("testing"), Value::Bool(true));
    map.insert(String::from("another"), Value::Bool(true));

    let cache = Arc::new(map);

    assert!(cache.contains_key("testing"));
    assert!(cache.contains_key("another"));

    let filter = create_api(cache.clone());

    let value = warp::test::request()
        .method("POST")
        .path("/del/testing")
        .reply(&filter)
        .await
        .into_body();

    let value: responses::DelResponse = serde_json::from_slice(&value).unwrap();
    let expected = responses::DelResponse { deleted: true };
    assert_eq!(expected, value);

    assert!(!cache.contains_key("testing"));
    assert!(cache.contains_key("another"));
}

#[tokio::test]
async fn delete_not_existing() {
    let map = DashMap::new();
    map.insert(String::from("testing"), Value::Bool(true));
    map.insert(String::from("another"), Value::Bool(true));

    let cache = Arc::new(map);

    assert!(cache.contains_key("testing"));
    assert!(cache.contains_key("another"));

    let filter = create_api(cache.clone());

    let value = warp::test::request()
        .method("POST")
        .path("/del/something")
        .reply(&filter)
        .await
        .into_body();

    let value: responses::DelResponse = serde_json::from_slice(&value).unwrap();
    let expected = responses::DelResponse { deleted: false };
    assert_eq!(expected, value);

    assert!(cache.contains_key("testing"));
    assert!(cache.contains_key("another"));
}

#[tokio::test]
async fn purge() {
    let map = DashMap::new();
    map.insert(String::from("testing"), Value::Bool(true));
    map.insert(String::from("another"), Value::Bool(true));

    let cache = Arc::new(map);

    assert!(cache.contains_key("testing"));
    assert!(cache.contains_key("another"));

    let filter = create_api(cache.clone());

    let value = warp::test::request()
        .method("POST")
        .path("/purge")
        .reply(&filter)
        .await
        .into_body();

    let value: responses::PurgeResponse = serde_json::from_slice(&value).unwrap();
    let expected = responses::PurgeResponse { purged: true };
    assert_eq!(expected, value);

    assert!(!cache.contains_key("testing"));
    assert!(!cache.contains_key("another"));
}
