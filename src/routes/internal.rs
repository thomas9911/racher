use crate::config::RuntimeConfigArc;
use crate::routes::utils::move_object;
use crate::Db;

use warp::http::StatusCode;

use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_value::Value;
use std::convert::Infallible;
// use std::net::SocketAddr;
use url::Url;
use warp::filters::BoxedFilter;
use warp::reply;
use warp::{Filter, Reply};

use crate::MAX_FILE_SIZE;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct JoinRequest {
    host: Url,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SyncRequest {
    code: String,
}

async fn inner_join(
    req: JoinRequest,
    cfg: RuntimeConfigArc,
) -> Result<impl warp::Reply, Infallible> {
    let mut guard = cfg.write().await;
    if req.host.cannot_be_a_base() {
        return Ok(reply::with_status(
            reply::json(&json!({ "error": "invalid host" })),
            StatusCode::BAD_REQUEST,
        ));
    }

    guard.neighbours.insert(req.host);
    let code = guard.base_code.clone();
    Ok(reply::with_status(
        reply::json(&json!({ "code": code })),
        StatusCode::OK,
    ))
}

async fn inner_sync(
    req: SyncRequest,
    cache: Db,
    cfg: RuntimeConfigArc,
) -> Result<impl warp::Reply, Infallible> {
    let guard = cfg.read().await;
    if guard.base_code == req.code {
        return Ok(reply::with_status(reply::json(&cache), StatusCode::OK));
    }

    Ok(reply::with_status(
        reply::json(&json!({"error": "invalid code"})),
        StatusCode::BAD_REQUEST,
    ))
}

async fn inner_update(
    name: String,
    simple_map: Value,
    cache: Db,
) -> Result<impl warp::Reply, Infallible> {
    cache.insert(name, simple_map);
    Ok::<_, Infallible>(super::ok_reponse())
}

pub fn join(cfg: RuntimeConfigArc) -> BoxedFilter<(impl Reply,)> {
    warp::path!("join")
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        // .map(move |req: JoinRequest| {
        //     let mut guard = block_on(cfg.write());
        //     guard.neighbours.insert(req.host);
        //     let code = guard.base_code.clone();
        //     reply::json(&json!({ "code": code }))
        // })
        .and(move_object(cfg))
        .and_then(inner_join)
        .boxed()
}

pub fn sync(cache: Db, cfg: RuntimeConfigArc) -> BoxedFilter<(impl Reply,)> {
    warp::path!("sync")
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        // .map(move |req: SyncRequest| {
        //     let guard = block_on(cfg.read());
        //     if guard.base_code == req.code {
        //         let json = reply::json(&cache);
        //         reply::with_status(json, StatusCode::OK)
        //     } else {
        //         let json = reply::json(&json!({"error": "invalid code"}));
        //         reply::with_status(json, StatusCode::BAD_REQUEST)
        //     }
        // })
        .and(move_object(cache))
        .and(move_object(cfg))
        .and_then(inner_sync)
        .with(warp::compression::gzip())
        .boxed()
}

pub fn update(cache: Db) -> BoxedFilter<(impl Reply,)> {
    warp::path!("update" / String)
        .and(warp::body::content_length_limit(MAX_FILE_SIZE))
        .and(warp::body::json())
        .and(move_object(cache))
        .and_then(inner_update)
        .boxed()
}
