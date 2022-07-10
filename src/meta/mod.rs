use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::db::sgdb::{SGDBFetchResult, SGDBColumn, SGDBColumnType};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MetaViewType {
    Table, Grid
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaView {
    meta_type: MetaViewType,
    meta_columns: HashMap<String, MetaColumn>
}


impl MetaView {
    pub fn default_sgdb_result(res: &SGDBFetchResult) -> Self {
        let meta_columns = res.data.keys().map(|col| { (col.name().to_string(), MetaColumn::default_sgdb_column(col.r#type())) }).collect();

        Self { meta_type: MetaViewType::Table, meta_columns }
    }

    #[inline]
    pub fn meta_column(&self, col: &SGDBColumn) -> &MetaColumn {
        self.meta_columns.get(col.name()).unwrap()
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
