use egui::{Key, Modifiers};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    db::sgdb::{SGDBColumnType, SGDBRowValue},
    ui::components::icons,
};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaQuery {
    icon: String,
    pub name: String,
    pub hotkey: MetaQueryHotKey,
    query_type: MetaQueryType,
    pub query: String,
    actions: Vec<MetaAction>,
    pub params: IndexMap<String, MetaParam>,
}

impl MetaQuery {
    pub fn has_setup(&self) -> bool {
        !self.params.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaQueryType {
    Global, Row { inject_columns: Vec<String> }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaQueryHotKey {
    pub modifiers: egui::Modifiers,
    pub key: egui::Key
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaAction {
    DoNothing,
    ShowQuery {
        tab: u8,
        meta_columns: Vec<MetaColumn>
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
    pub id: String,
    pub r#type: MetaParamType,
    pub default: MetaParamValue
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaParamType {
    Text, Boolean, Number, Decimal
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MetaParamValue {
    Text(String), Boolean(bool), Number(i64), Decimal(f64)
}

impl MetaQuery {
    pub fn from_normal_query(
        name: impl Into<String>,
        query: impl Into<String>,
        res: &FetchResult,
    ) -> Self {
        let columns = res.res.keys().cloned().collect();
        Self {
            icon: icons::ICON_TABLE.to_string(),
            name: name.into(),
            query: query.into(),
            query_type: MetaQueryType::Global,
            params: IndexMap::new(),
            hotkey: MetaQueryHotKey { modifiers: Modifiers::CTRL, key: Key::T},
            actions: vec![MetaAction::ShowQuery { tab: 0, meta_columns: columns }],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum ImageType {
    Url, File
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum MetaColumnType {
    Text { color: Option<(u8, u8, u8)> },
    CheckBox,
    Number { variant: MetaColNumber },
    DateTime { format: String },
    Image(ImageType),
    Binary,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum MetaColNumber {
    Simple,
    Money,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct MetaColumn {
    pub name: String,
    pub raw_name: String,
    pub r#type: MetaColumnType
}


impl MetaColumn {
    pub fn default_sgdb_column(raw_name: impl Into<String>, col_type: SGDBColumnType) -> Self {

        let r#type = match col_type {
            SGDBColumnType::Text => MetaColumnType::Text { color: None },
            SGDBColumnType::Boolean => MetaColumnType::CheckBox,
            SGDBColumnType::Integer => MetaColumnType::Number {
                variant: MetaColNumber::Simple,
            },
            SGDBColumnType::UInteger => MetaColumnType::Number {
                variant: MetaColNumber::Simple,
            },
            SGDBColumnType::Double => MetaColumnType::Number {
                variant: MetaColNumber::Simple,
            },
            SGDBColumnType::Decimal => MetaColumnType::Number {
                variant: MetaColNumber::Simple,
            },
            SGDBColumnType::DateTime => MetaColumnType::DateTime {
                format: "%d/%m/%Y %H:%M:%S".to_string(),
            },
            SGDBColumnType::Binary => MetaColumnType::Binary,
            SGDBColumnType::Unknown => MetaColumnType::Unknown,
        };

        let raw_name = raw_name.into();

        Self {
            name: raw_name.clone(),
            raw_name,
            r#type
        }
    }
}

pub struct FetchResult {
    pub num_rows: usize,
    pub res: IndexMap<MetaColumn, Vec<SGDBRowValue>>
}
