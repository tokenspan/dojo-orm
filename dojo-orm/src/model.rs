use crate::types::ToSql;
use anyhow::Result;
use async_graphql::Enum;
use chrono::NaiveDateTime;
use postgres_types::{accepts, to_sql_checked};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::{Display, EnumString};
use uuid::Uuid;

macro_rules! impl_value {
    ($ty: ty, $variant: ident) => {
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Value::$variant(value)
            }
        }

        impl From<&$ty> for Value {
            fn from(value: &$ty) -> Self {
                Value::$variant(value.clone())
            }
        }

        impl From<Option<$ty>> for Value {
            fn from(_: Option<$ty>) -> Self {
                Value::Null
            }
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Value {
    Uuid(Uuid),
    Int32(i32),
    Int64(i64),
    String(String),
    NaiveDateTime(NaiveDateTime),
    Null,
}

impl_value!(Uuid, Uuid);
impl_value!(i32, Int32);
impl_value!(i64, Int64);
impl_value!(String, String);
impl_value!(NaiveDateTime, NaiveDateTime);

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, EnumString, Serialize, Deserialize)]
pub enum Direction {
    #[graphql(name = "asc")]
    #[strum(serialize = "asc", serialize = "ASC")]
    Asc,
    #[graphql(name = "desc")]
    #[strum(serialize = "desc", serialize = "DESC")]
    Desc,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Asc => write!(f, "ASC"),
            Direction::Desc => write!(f, "DESC"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct OrderValue {
    pub column: String,
    pub direction: Direction,
}

impl From<OrderValue> for String {
    fn from(value: OrderValue) -> Self {
        format!("{} {}", value.column, value.direction)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Orders(pub Vec<OrderValue>);

impl Orders {
    pub fn new(values: Vec<OrderValue>) -> Self {
        Self(values)
    }

    pub fn to_sql(&self) -> String {
        let mut stmt = "".to_string();
        if let Some(value) = self.0.first() {
            stmt.push_str(value.column.as_str());
            stmt.push_str(" ");
            stmt.push_str(value.direction.to_string().as_str());
        }

        for value in self.0.iter().skip(1) {
            stmt.push_str(", ");
            stmt.push_str(value.column.as_str());
            stmt.push_str(" ");
            stmt.push_str(Direction::Asc.to_string().as_str());
        }

        stmt
    }
}

impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &crate::types::Type,
        w: &mut bytes::BytesMut,
    ) -> std::result::Result<crate::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            Value::Uuid(t) => t.to_sql(ty, w),
            Value::Int32(t) => t.to_sql(ty, w),
            Value::Int64(t) => t.to_sql(ty, w),
            Value::String(t) => t.to_sql(ty, w),
            Value::NaiveDateTime(t) => t.to_sql(ty, w),
            Value::Null => Ok(crate::types::IsNull::Yes),
        }
    }

    accepts!(UUID, INT4, INT8, TEXT, TIMESTAMP);
    to_sql_checked!();
}

pub trait Model {
    const NAME: &'static str;
    const COLUMNS: &'static [&'static str];
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
    fn from_row(row: tokio_postgres::Row) -> Result<Self>
    where
        Self: Sized;

    fn get_value(&self, column: &str) -> Option<Value>;
    fn sort_keys() -> Vec<String>;
}

pub trait UpdateModel {
    const COLUMNS: &'static [&'static str];
    fn columns(&self) -> Vec<&'static str>;
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}
