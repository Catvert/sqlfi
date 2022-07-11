use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    db::sgdb::{SGDBColumnType, SGDBFetchResult},
    ui::components::icons,
};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaQuery {
    icon: String,
    name: String,
    query_type: MetaQueryType,
    query: String,
    params: IndexMap<String, MetaParam>,
    actions: Vec<MetaAction>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaQueryType {
    Global, Row { inject_columns: Vec<String> }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaQueryHotKey {
    ctrl: bool,
    alt: bool,
    shift: bool,
    key: egui::Key
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaAction {
    DoNothing,
    ShowQuery {
        tab: u8,
        meta_columns: HashMap<String, MetaColumn>
    },
    Command {
        command: String
    },
    CommandPipeMetaQuery {
        meta_query_id: String,
        command: String,
        response_type: CommandPipeMetaQueryResponseType,
    },
    CallMetaQuery { meta_query_id: String }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CommandPipeMetaQueryResponseType {
    JSON, CSV
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaParam {
    id: String,
    r#type: MetaParamType,
    default: MetaParamTypeDefault
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaParamType {
    Text, Boolean, Number, Decimal
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaParamTypeDefault {
    Text(String), Boolean(bool), Number(i64), Decimal(f64)
}

impl MetaQuery {
    pub fn from_normal_query(
        name: impl Into<String>,
        query: impl Into<String>,
        res: &SGDBFetchResult,
    ) -> Self {
        let columns = res
            .data
            .keys()
            .map(|col| {
                (
                    col.name().to_string(),
                    MetaColumn::default_sgdb_column(col.r#type()),
                )
            })
            .collect();

        Self {
            icon: icons::ICON_TABLE.to_string(),
            name: name.into(),
            query: query.into(),
            query_type: MetaQueryType::Global,
            params: IndexMap::new(),
            actions: vec![MetaAction::ShowQuery { tab: 0, meta_columns: columns }],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaColumn {
    Text { color: Option<(u8, u8, u8)> },
    CheckBox,
    Number { variant: MetaColNumber },
    DateTime { format: String },
    Binary,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaColNumber {
    Simple,
    Money,
}

impl MetaColumn {
    pub fn default_sgdb_column(col_type: SGDBColumnType) -> Self {
        match col_type {
            SGDBColumnType::Text => MetaColumn::Text { color: None },
            SGDBColumnType::Boolean => MetaColumn::CheckBox,
            SGDBColumnType::Integer => MetaColumn::Number {
                variant: MetaColNumber::Simple,
            },
            SGDBColumnType::UInteger => MetaColumn::Number {
                variant: MetaColNumber::Simple,
            },
            SGDBColumnType::Double => MetaColumn::Number {
                variant: MetaColNumber::Simple,
            },
            SGDBColumnType::Decimal => MetaColumn::Number {
                variant: MetaColNumber::Simple,
            },
            SGDBColumnType::DateTime => MetaColumn::DateTime {
                format: "%d/%m/%Y %H:%M:%S".to_string(),
            },
            SGDBColumnType::Binary => MetaColumn::Binary,
            SGDBColumnType::Unknown => MetaColumn::Unknown,
        }
    }
}
