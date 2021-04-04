use serde::{Deserialize, Serialize};
use serde_value::Value;

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum Response {
    GET(GetResponse),
    SET(SetResponse),
    DEL(DelResponse),
    KEYS(KeysResponse),
    PURGE(PurgeResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse {
    data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetResponse {
    status: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DelResponse {
    deleted: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeysResponse {
    keys: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurgeResponse {
    purged: Value,
}
