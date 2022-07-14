use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;

use indexmap::IndexMap;
use log::info;
use sqlx::{
    mysql::{MySqlColumn, MySqlRow, MySqlValueRef},
    Column, Decode, MySql, MySqlPool, Row, Type, TypeInfo, ValueRef,
};

use super::{SGDBColumn, SGDBColumnType, SGDBFetchResult, SGDBRowValue, SGDBTable, SGDB};

pub struct MySQL {
    pool: MySqlPool,
}

impl MySQL {
    pub async fn connect(uri: &str) -> Result<Self> {
        let pool = MySqlPool::connect(uri).await?;
        Ok(MySQL { pool })
    }

    pub async fn list_databases(&self) -> Result<Vec<String>> {
        let res = sqlx::query("SHOW DATABASES")
            .map(|r: MySqlRow| r.get::<String, _>(0))
            .fetch_all(&self.pool)
            .await?;

        Ok(res)
    }
}

fn decode<'r, T>(value: MySqlValueRef<'r>) -> Result<T>
where
    T: Decode<'r, MySql> + Type<MySql>,
{
    if !value.is_null() {
        let ty = value.type_info();

        if !ty.is_null() && !T::compatible(&ty) {
            bail!("Invalid column type encountered")
        }
    }

    Ok(T::decode(value).map_err(|source| anyhow!("Invalid column type encountered"))?)
}

fn map_column_value(col: &SGDBColumn, row: &MySqlRow) -> Result<SGDBRowValue> {
    let value = row.try_get_raw(col.ordinal())?;

    Ok(if value.is_null() {
        SGDBRowValue::Null
    } else {
        match col.r#type {
            SGDBColumnType::Text => SGDBRowValue::Text(decode(value)?),
            SGDBColumnType::Boolean => SGDBRowValue::Boolean(decode(value)?),
            SGDBColumnType::Integer => SGDBRowValue::Integer(decode(value)?),
            SGDBColumnType::UInteger => SGDBRowValue::UInteger(decode(value)?),
            SGDBColumnType::Double => SGDBRowValue::Double(decode(value)?),
            SGDBColumnType::Decimal => SGDBRowValue::Decimal(decode(value)?),
            SGDBColumnType::DateTime => SGDBRowValue::DateTime(decode(value)?),
            SGDBColumnType::Binary => SGDBRowValue::Binary(decode(value)?),
            SGDBColumnType::Unknown => SGDBRowValue::Unknown {
                error: "Unknown column type".to_string(),
            },
        }
    })
}

fn map_column(col: &MySqlColumn) -> SGDBColumn {
    info!("{}", col.type_info().name());

    let r#type = match col.type_info().name() {
        "BOOLEAN" => SGDBColumnType::Boolean,
        "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "INT UNSIGNED" | "MEDIUMINT UNSIGNED"
        | "BIGINT UNSIGNED" => SGDBColumnType::UInteger,
        "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" | "BIGINT" => SGDBColumnType::Integer,
        "DECIMAL" => SGDBColumnType::Decimal,
        "FLOAT" | "DOUBLE" => SGDBColumnType::Double,
        "CHAR" | "VARCHAR" | "TEXT" => SGDBColumnType::Text,

        "DATE" | "DATETIME" | "TIMESTAMP" => SGDBColumnType::DateTime,

        _ => SGDBColumnType::Unknown,
    };

    SGDBColumn {
        name: col.name().to_string(),
        ordinal: col.ordinal(),
        r#type,
    }
}

#[async_trait]
impl SGDB for MySQL {
    async fn fetch_all(&self, query: &str, params: Option<Vec<String>>) -> Result<SGDBFetchResult> {
        let mut num_rows: usize = 0;
        let mut res = sqlx::query(query);

        if let Some(params) = params {
            for param in params {
                res = res.bind(param);
            }
        }

        let res = res.fetch_all(&self.pool)
            .await?
            .into_iter()
            .enumerate()
            .fold(IndexMap::new(), |mut map, (index, row)| {
                if index == 0 {
                    for col in row.columns() {
                        map.entry(map_column(col)).or_insert(Vec::new());
                    }
                }

                for (col, values) in map.iter_mut() {
                    values.push(map_column_value(col, &row).unwrap_or_else(|err| {
                        SGDBRowValue::Unknown {
                            error: format!("{}", err),
                        }
                    }));
                }

                num_rows += 1;

                map
            });

        Ok(SGDBFetchResult {
            data: res,
            num_rows,
        })
    }

    async fn list_tables(&self, schema: &str) -> Result<Vec<super::SGDBTable>> {
        let tables = sqlx::query(
            "SELECT table_name, table_type, engine, version, table_rows, create_time FROM INFORMATION_SCHEMA.TABLES WHERE table_schema = ?",
        )
        .bind(&schema)
        .map(|row: MySqlRow| {
            let table_name = row.get("table_name");
            let full_path = format!("{}.{}", &schema, table_name);
            SGDBTable {
                table_name,
                table_type: row.get("table_type"),
                full_path,
                schema: schema.to_string(),
                engine: row.get("engine"),
                table_rows: row.get("table_rows"),
                // create_time: row.get("create_time"),
            }
        })
        .fetch_all(&self.pool)
        .await?;

        Ok(tables)
    }
}
