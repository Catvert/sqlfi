mod mysql;

use std::hash::Hash;

use anyhow::Result;

use async_trait::async_trait;
use chrono::Utc;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;

#[async_trait]
pub trait SGDBBuilder {
    async fn acquire_sgdb(&self) -> Result<Box<dyn SGDB>>;
}

pub struct Connection {
    kind: SGDBKind,
    uri: String,
}

impl Connection {
    pub fn new(kind: SGDBKind, uri: impl Into<String>) -> Self {
        let uri = uri.into();
        Connection { uri, kind }
    }

    pub fn into_connection_schema(self, schema: impl Into<String>) -> ConnectionSchema {
        ConnectionSchema::new(self, schema.into())
    }

    pub async fn databases(&self) -> Result<Vec<String>> {
        match self.kind {
            SGDBKind::Mysql => {
                mysql::MySQL::connect(&self.uri)
                    .await?
                    .list_databases()
                    .await
            }
            SGDBKind::Postgres => todo!(),
            SGDBKind::Sqlite => todo!(),
        }
    }
}

#[async_trait]
impl SGDBBuilder for Connection {
    async fn acquire_sgdb(&self) -> Result<Box<dyn SGDB>> {
        Ok(match self.kind {
            SGDBKind::Mysql => {
                let sgdb = mysql::MySQL::connect(&self.uri).await?;

                Box::new(sgdb) as Box<dyn SGDB>
            }
            SGDBKind::Postgres => todo!(),
            SGDBKind::Sqlite => todo!(),
        })
    }
}

pub struct ConnectionSchema {
    connection: Connection,
    schema: String,
}

impl ConnectionSchema {
    pub fn new(connection: Connection, schema: String) -> Self {
        ConnectionSchema { connection, schema }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }
}

#[async_trait]
impl SGDBBuilder for ConnectionSchema {
    async fn acquire_sgdb(&self) -> Result<Box<dyn SGDB>> {
        Ok(match self.connection.kind {
            SGDBKind::Mysql => {
                let sgdb =
                    mysql::MySQL::connect(&format!("{}/{}", self.connection.uri, self.schema))
                        .await?;

                Box::new(sgdb) as Box<dyn SGDB>
            }
            SGDBKind::Postgres => todo!(),
            SGDBKind::Sqlite => todo!(),
        })
    }
}

#[derive(Debug)]
pub struct SGDBTable {
    pub schema: String,
    pub table_name: String,
    pub full_path: String,

    pub table_type: String,
    pub engine: String,
    pub table_rows: u64,
    // pub create_time: chrono::NaiveDateTime,
}

#[async_trait]
pub trait SGDB: Send + Sync {
    async fn fetch_all(&self, query: &str, params: Option<Vec<String>>) -> Result<SGDBFetchResult>;

    async fn list_tables(&self, schema: &str) -> Result<Vec<SGDBTable>>;
}

#[derive(Default, Serialize, Deserialize, Clone, Copy)]
pub enum SGDBKind {
    #[default]
    Mysql,
    Postgres,
    Sqlite,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SGDBColumn {
    name: String,
    ordinal: usize,
    r#type: SGDBColumnType,
}

impl SGDBColumn {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn r#type(&self) -> SGDBColumnType {
        self.r#type
    }
}

impl Hash for SGDBColumn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.ordinal.hash(state);
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SGDBColumnType {
    Text,
    Boolean,
    Integer,
    UInteger,
    Double,
    Decimal,
    DateTime,
    Binary,
    Unknown,
}

#[derive(Debug)]
pub enum SGDBRowValue {
    Text(String),
    Boolean(bool),
    Integer(i64),
    UInteger(u64),
    Double(f64),
    Decimal(BigDecimal),
    DateTime(chrono::DateTime<Utc>),
    Binary(Vec<u8>),
    Null,
    Unknown { error: String },
}

#[derive(Debug)]
pub struct SGDBFetchResult {
    pub data: IndexMap<SGDBColumn, Vec<SGDBRowValue>>,
    pub num_rows: usize,
}
