use indexmap::IndexMap;
use serde::{Serialize, Deserialize};

use crate::{db::sgdb::{SGDBKind, ConnectionSchema, Connection}, meta::MetaQuery};

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub name: String,
    pub kind: SGDBKind,
    pub uri: String,
    pub schema: String,

    pub meta_queries: IndexMap<String, MetaQuery>
}

impl ConnectionConfig {
    pub fn new(name: impl Into<String>, kind: SGDBKind, uri: impl Into<String>, schema: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind,
            uri: uri.into(),
            schema: schema.into(),
            meta_queries: IndexMap::new()
        }
    }
}

impl Into<ConnectionSchema> for ConnectionConfig {
    fn into(self) -> ConnectionSchema {
        ConnectionSchema::new(Connection::new(self.kind, self.uri), self.schema)
    }
}

#[derive(Serialize, Deserialize)]
pub struct MainConfig {
    connections: Vec<ConnectionConfig>
}
