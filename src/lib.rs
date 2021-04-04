#![warn(rust_2018_idioms)]

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
