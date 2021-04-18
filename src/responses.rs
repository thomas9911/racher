use crate::Db;
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
    JOIN(JoinResponse),
    SYNC(Db),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GetResponse {
    pub data: Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SetResponse {
    pub status: Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DelResponse {
    pub deleted: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct KeysResponse {
    pub keys: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PurgeResponse {
    pub purged: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct JoinResponse {
    pub code: String,
}
