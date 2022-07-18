use anyhow::Result;
use directories::ProjectDirs;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    db::sgdb::{Connection, SGDBKind},
    meta::MetaQuery,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub name: String,
    pub kind: SGDBKind,
    pub uri: String,
    pub schema: String,
    pub meta_queries: IndexMap<String, MetaQuery>,
}

impl ConnectionConfig {
    pub fn new(
        name: impl Into<String>,
        kind: SGDBKind,
        uri: impl Into<String>,
        schema: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            uri: uri.into(),
            schema: schema.into(),
            meta_queries: IndexMap::new(),
        }
    }
}

impl Into<Connection> for ConnectionConfig {
    fn into(self) -> Connection {
        Connection::new(self.kind, self.uri, self.schema)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SqlifeConfig {
    pub connections: Vec<ConnectionConfig>,
}

impl SqlifeConfig {
    fn config_file_path() -> PathBuf {
        Self::project_dir().config_dir().join("config.json")
    }

    fn project_dir() -> ProjectDirs {
        ProjectDirs::from("com", "sqlife", "sqlife").unwrap()
    }

    pub fn load() -> Result<Self> {
        Self::load_custom_path(Self::config_file_path())
    }

    pub fn load_custom_path(path: impl Into<PathBuf>) -> Result<Self> {
        let content = std::fs::read_to_string(path.into())?;
        serde_json::de::from_str(&content).map_err(|err| anyhow::anyhow!(err))
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::ser::to_string_pretty(self)?;

        let config_path = Self::config_file_path();
        fs::create_dir_all(config_path.parent().unwrap())?;

        let mut w = File::create(config_path)?;

        w.write(content.as_bytes())?;

        Ok(())
    }
}
