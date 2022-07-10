use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{db::sgdb::{SGDBFetchResult, SGDBColumn, SGDBColumnType}, ui::components::icons};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MetaQueryType {
    Table, Grid { columns: u8 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaQuery {
    icon: String,
    name: String,
    query: String,
    meta_type: MetaQueryType,
    meta_columns: HashMap<String, MetaColumn>
}


impl MetaQuery {
    pub fn new(name: impl Into<String>, query: impl Into<String>, res: &SGDBFetchResult) -> Self {
        let meta_columns = res.data.keys().map(|col| { (col.name().to_string(), MetaColumn::default_sgdb_column(col.r#type())) }).collect();

        Self { icon: icons::ICON_TABLE.to_string(), name: name.into(), query: query.into(), meta_type: MetaQueryType::Table, meta_columns }
    }

    #[inline]
    pub fn meta_column(&self, col: &SGDBColumn) -> Option<&MetaColumn> {
        self.meta_columns.get(col.name())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaColumn {
    Text {
        color: Option<(u8, u8, u8)>,
    },
    CheckBox,
    Number { variant: MetaColNumber },
    DateTime { format: String },
    Binary,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaColNumber {
    Simple, Money
}

impl MetaColumn {
    pub fn default_sgdb_column(col_type: SGDBColumnType) -> Self {
        match col_type {
            SGDBColumnType::Text => MetaColumn::Text { color: None },
            SGDBColumnType::Boolean => MetaColumn::CheckBox,
            SGDBColumnType::Integer => MetaColumn::Number { variant: MetaColNumber::Simple },
            SGDBColumnType::UInteger => MetaColumn::Number { variant: MetaColNumber::Simple },
            SGDBColumnType::Double => MetaColumn::Number { variant: MetaColNumber::Simple },
            SGDBColumnType::Decimal => MetaColumn::Number { variant: MetaColNumber::Simple },
            SGDBColumnType::DateTime => MetaColumn::DateTime { format: "%d/%m/%Y %H:%M:%S".to_string() },
            SGDBColumnType::Binary => MetaColumn::Binary,
            SGDBColumnType::Unknown => MetaColumn::Unknown,
        }
    }
}
