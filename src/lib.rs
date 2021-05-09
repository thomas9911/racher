#![warn(rust_2018_idioms)]

pub mod arguments;
pub mod cli;
pub mod client;
pub mod config;
pub mod responses;
pub mod routes;
pub mod sync;
pub mod transport;
use config::RuntimeConfigArc;

use sync::Arc;

use dashmap::DashMap;
use serde_value::Value;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

pub type Db = Arc<DashMap<String, Value>>;

pub const MAX_FILE_SIZE: u64 = 1024 * 1024 * 32;

pub fn create_api(
    arc_cache: Db,
    cfg: RuntimeConfigArc,
    tx: transport::Sender,
) -> BoxedFilter<(impl Reply,)> {
    let cors = warp::cors().allow_any_origin().allow_methods(&[warp::http::Method::POST]).allow_header("content-type");

    let api = warp::post()
        .and(
            routes::setter(arc_cache.clone(), tx.clone())
                .or(routes::getter(arc_cache.clone()))
                .or(routes::keys(arc_cache.clone()))
                .or(routes::purge(arc_cache.clone()))
                .or(routes::ping())
                .or(routes::deleter(arc_cache.clone(), tx))
                .or(routes::internal(arc_cache.clone(), cfg.clone())),
        ).with(cors);

    // api.or(warp::options().map(warp::reply).with(cors))
    // let cors_stuff = warp::options().map(warp::reply).with(cors);
    // warp::any().and(api).or(cors_stuff)
    //     .boxed()

    cfg_if::cfg_if! {
        if #[cfg(feature = "dashboard")] {
            api.or(warp::get().and(routes::web())).boxed()
        } else {
            api.boxed()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup(arc_cache: Db) -> BoxedFilter<(impl Reply,)> {
        let (tx, _) = transport::channel(16);

        let config = config::RuntimeConfig::default().to_arc();
        create_api(arc_cache, config, tx)
    }

    #[tokio::test]
    async fn get_not_existing() {
        let map = DashMap::new();

        let cache = Arc::new(map);
        let filter = setup(cache);

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
        let filter = setup(cache);

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
        let filter = setup(cache.clone());

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
    async fn set_does_not_work_with_slash() {
        let map = DashMap::new();

        let cache = Arc::new(map);
        let filter = setup(cache.clone());

        let response = warp::test::request()
            .method("POST")
            .path("/set/testing/more/key/even")
            .json(&Value::I64(123))
            .reply(&filter)
            .await;

        assert_eq!(404, response.status());
    }

    #[tokio::test]
    async fn keys() {
        let map = DashMap::new();
        map.insert(String::from("testing"), Value::Bool(true));
        map.insert(String::from("another"), Value::Bool(true));

        let cache = Arc::new(map);
        let filter = setup(cache);

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

        let filter = setup(cache.clone());

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

        let filter = setup(cache.clone());

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

        let filter = setup(cache.clone());

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
}
