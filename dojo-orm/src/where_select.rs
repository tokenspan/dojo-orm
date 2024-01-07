use std::collections::HashMap;
use std::marker::PhantomData;

use tracing::debug;

use crate::cursor::order_by::CursorOrderByClause;
use crate::model::Model;
use crate::ops::{Op, OpValue, OpValueType};
use crate::order_by::{Order, OrderByClause};
use crate::pagination::Cursor;
use crate::pool::*;
use crate::types::ToSql;

pub struct WhereSelectClause<'a, T>
where
    T: Model,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) params: Vec<&'a (dyn ToSql + Sync)>,
    pub(crate) ops: Vec<Op<'a>>,
    pub(crate) is_delete: bool,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> WhereSelectClause<'a, T>
where
    T: Model,
{
    pub fn order_by(&'a self, order: (&'a str, Order)) -> OrderByClause<'a, T> {
        OrderByClause {
            pool: &self.pool,
            params: &self.params,
            ops: &self.ops,
            orders: HashMap::from([order]),
            _t: PhantomData,
        }
    }

    pub fn where_by(&'a mut self, op: Op<'a>) -> &'a mut Self {
        self.ops.push(op);
        self
    }

    pub fn cursor(
        &'a mut self,
        first: Option<i64>,
        last: Option<i64>,
        before: &'a Option<Cursor>,
        after: &'a Option<Cursor>,
    ) -> CursorOrderByClause<'a, T> {
        CursorOrderByClause {
            pool: &self.pool,
            params: &self.params,
            ops: &self.ops,
            orders: vec![],
            first,
            last,
            before,
            after,
            _t: PhantomData,
        }
    }

    pub fn build(&'a self) -> (String, Vec<&'a (dyn ToSql + Sync)>) {
        let mut params_index = 1;

        let mut params = self.params.clone();
        let mut query = if self.is_delete {
            format!("DELETE FROM {}", T::NAME)
        } else {
            let mut query = "SELECT ".to_string();
            query.push_str(&T::COLUMNS.join(", "));
            query.push_str(" FROM ");
            query.push_str(T::NAME);

            query
        };

        let mut ands = vec![];
        for op in &self.ops {
            let (q, p) = op.sql(&mut params_index);
            if let Some(q) = q {
                ands.push(q);
                params.extend_from_slice(&p);
            }
        }
        if !ands.is_empty() {
            let and = ands.join(" AND ");
            query.push_str(" WHERE ");
            query.push_str(&and);
        }

        if self.is_delete {
            query.push_str(" RETURNING ");
            query.push_str(&T::COLUMNS.join(", "));
        }

        (query, params)
    }

    pub async fn first(&'a mut self) -> anyhow::Result<Option<T>> {
        let (query, params) = self.build();
        let query = if self.is_delete {
            query
        } else {
            format!("{} LIMIT 1", query)
        };
        debug!("query: {}, params: {:?}", query, params);
        let conn = self.pool.get().await?;

        conn.query_opt(query.as_str(), &params)
            .await?
            .map(T::from_row)
            .transpose()
    }

    pub async fn all(&'a mut self) -> anyhow::Result<Vec<T>> {
        let (query, params) = self.build();
        debug!("query: {}, params: {:?}", query, params);
        let conn = self.pool.get().await?;

        let mut rows = vec![];
        for row in conn.query(query.as_str(), &params).await? {
            rows.push(T::from_row(row)?);
        }

        Ok(rows)
    }
}
