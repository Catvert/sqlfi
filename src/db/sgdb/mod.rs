mod mysql;

use std::hash::Hash;

use anyhow::Result;

use async_trait::async_trait;
use chrono::Utc;
use indexmap::IndexMap;
use sqlx::types::BigDecimal;

pub async fn connect(kind: SGDBKind, uri: &str) -> Result<Box<dyn SGDB>> {
    Ok(Box::new(match kind {
        SGDBKind::MySQL => mysql::MySQL::connect(uri).await?,
        SGDBKind::Postgres => todo!(),
        SGDBKind::Sqlite => todo!(),
    }))
}

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
pub trait SGDB {
    async fn fetch_all(&self, query: &str) -> Result<SGDBFetchResult>;
    async fn tables(&self, schema: &str) -> Result<Vec<SGDBTable>>;
}

pub enum SGDBKind {
    MySQL,
    Postgres,
    Sqlite,
}

#[derive(PartialEq, Eq)]
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

pub struct SGDBFetchResult {
    pub data: IndexMap<SGDBColumn, Vec<SGDBRowValue>>,
    pub num_rows: usize,
}
