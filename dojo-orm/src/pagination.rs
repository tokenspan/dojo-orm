use std::error::Error;
use std::fmt::Display;
use std::marker::PhantomData;

use crate::types::{accepts, to_sql_checked, IsNull, ToSql, Type};
use crate::{Direction, Model, OrderValue};
use anyhow::Result;
use async_graphql::connection::{Connection, CursorType, Edge};
use async_graphql::{
    InputValueError, InputValueResult, OutputType, Scalar, ScalarType, SimpleObject, Value,
};
use base64ct::{Base64, Encoding};
use bytes::BytesMut;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub trait DefaultSortKeys {
    fn keys() -> Vec<String>;

    fn order_by_stmt(direction: Direction) -> String {
        let mut stmt = "".to_string();
        for (i, order) in Self::keys().iter().enumerate() {
            if i > 0 {
                stmt.push_str(", ");
            }
            stmt.push_str(&order);
            if i == 0 {
                if direction == Direction::Asc {
                    stmt.push_str(" ASC");
                } else {
                    stmt.push_str(" DESC");
                }
            } else {
                stmt.push_str(" ASC");
            }
        }

        stmt
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorValue {
    pub column: String,
    pub value: crate::model::Value,
}

impl CursorValue {
    pub fn new(column: String, value: crate::model::Value) -> Self {
        Self { column, value }
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub values: Vec<CursorValue>,
}

impl Cursor {
    pub fn to_where_stmt(&self, direction: Direction) -> (String, Vec<&(dyn ToSql + Sync)>) {
        let mut columns = vec![];
        let mut params: Vec<&(dyn ToSql + Sync)> = vec![];
        for value in &self.values {
            columns.push(value.column.clone());
            params.push(&value.value);
        }
        let mut stmt = "(".to_string();
        stmt.push_str(&columns.join(", "));
        stmt.push_str(") ");

        if direction == Direction::Asc {
            stmt.push_str(">");
        } else {
            stmt.push_str("<");
        }
        stmt.push_str(" (");
        stmt.push_str(
            &params
                .iter()
                .enumerate()
                .map(|(i, _)| format!("${}", i + 1))
                .collect::<Vec<_>>()
                .join(", "),
        );
        stmt.push_str(")");

        (stmt, params)
    }

    pub fn to_order_by_stmt(&self, direction: Direction) -> String {
        let keys = self.values.iter().map(|v| v.column.clone()).collect();
        Self::order_by_stmt_by_keys(&keys, direction)
    }

    pub fn order_by_stmt_by_keys(keys: &Vec<String>, direction: Direction) -> String {
        let mut stmt = "".to_string();
        if let Some(value) = keys.first() {
            stmt.push_str(value);
            if direction == Direction::Asc {
                stmt.push_str(" ASC");
            } else {
                stmt.push_str(" DESC");
            }
        }

        for value in keys.iter().skip(1) {
            stmt.push_str(", ");
            stmt.push_str(value);
            stmt.push_str(" ASC");
        }

        stmt
    }
}

impl Cursor {
    pub fn new(values: Vec<CursorValue>) -> Self {
        Self { values }
    }

    pub fn encode(&self) -> String {
        // it's safe, trust me bro.
        let buf = bincode::serialize(&self.values).unwrap();
        Base64::encode_string(buf.as_slice())
    }

    pub fn decode(encoded: &str) -> Result<Self> {
        let decoded = Base64::decode_vec(encoded).unwrap();
        let values: Vec<CursorValue> = bincode::deserialize(&decoded[..])?;
        Ok(Self { values })
    }
}

#[Scalar]
impl ScalarType for Cursor {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            let cursor =
                Cursor::decode(value).map_err(|e| InputValueError::custom(e.to_string()))?;
            Ok(cursor)
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        let value = self.encode();
        Value::String(value)
    }
}

impl CursorType for Cursor {
    type Error = anyhow::Error;

    fn decode_cursor(s: &str) -> std::result::Result<Self, Self::Error> {
        Self::decode(s)
    }

    fn encode_cursor(&self) -> String {
        self.encode()
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct AdditionalFields {
    pub total_nodes: i64,
}

#[derive(Debug)]
pub struct Pagination<T> {
    pub items: Vec<T>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
    pub limit: i64,
    pub total_nodes: i64,
}

impl<T> Pagination<T> {
    pub fn new(
        items: Vec<T>,
        before: Option<Cursor>,
        after: Option<Cursor>,
        limit: i64,
        total_nodes: i64,
    ) -> Self {
        Self {
            items,
            before,
            after,
            limit,
            total_nodes,
        }
    }
}

impl<T> From<Pagination<T>> for Connection<Cursor, T, AdditionalFields>
where
    T: OutputType + Model,
{
    fn from(value: Pagination<T>) -> Self {
        let has_previous = false;
        let has_next = false;
        let mut connection = Connection::with_additional_fields(
            has_previous,
            has_next,
            AdditionalFields {
                total_nodes: value.total_nodes,
            },
        );

        let edges = vec![];

        connection.edges.extend(edges);

        connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::{connection::Connection, SimpleObject};
    use chrono::NaiveDateTime;
    use dojo_macros::Model;
    use googletest::prelude::*;
    use uuid::Uuid;

    #[test]
    fn test_cursor_to_sql_with_1_key() -> anyhow::Result<()> {
        let created_at = NaiveDateTime::parse_from_str("2024-01-07 12:34:56", "%Y-%m-%d %H:%M:%S")?;
        let cursor_value = CursorValue {
            column: "created_at".to_string(),
            value: crate::model::Value::NaiveDateTime(created_at),
        };
        let cursor = Cursor::new(vec![cursor_value]);
        let (sql, params) = cursor.to_where_stmt(Direction::Asc);
        println!("sql: {}", sql);
        println!("params: {:?}", params);

        Ok(())
    }

    #[test]
    fn test_cursor_to_sql_with_2_key() -> anyhow::Result<()> {
        let created_at = NaiveDateTime::parse_from_str("2024-01-07 12:34:56", "%Y-%m-%d %H:%M:%S")?;
        let uuid = Uuid::parse_str("ce2087a7-bdbc-4453-9fb8-d4dff3584f3e")?;
        let cursor = Cursor::new(vec![
            CursorValue {
                column: "created_at".to_string(),
                value: crate::model::Value::NaiveDateTime(created_at),
            },
            CursorValue {
                column: "id".to_string(),
                value: crate::model::Value::Uuid(uuid),
            },
        ]);
        let (sql, params) = cursor.to_where_stmt(Direction::Asc);
        println!("sql: {}", sql);
        println!("params: {:?}", params);

        Ok(())
    }

    #[test]
    fn test_decode_cursor() -> anyhow::Result<()> {
        let created_at = NaiveDateTime::parse_from_str("2024-01-07 12:34:56", "%Y-%m-%d %H:%M:%S")?;
        let cursor_value = CursorValue {
            column: "created_at".to_string(),
            value: crate::model::Value::NaiveDateTime(created_at),
        };
        let cursor = Cursor::new(vec![cursor_value]);
        let encoded = cursor.encode();

        let decoded = Cursor::decode(&encoded).unwrap();
        assert_that!(
            decoded,
            pat!(Cursor {
                values: contains_each![pat!(CursorValue {
                    column: eq("created_at".to_string()),
                    value: eq(crate::model::Value::NaiveDateTime(created_at)),
                }),],
            })
        );

        Ok(())
    }
}
