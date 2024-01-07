use async_graphql::futures_util::TryFutureExt;
use std::marker::PhantomData;
use tracing::debug;

use crate::model::Model;
use crate::ops::{Op, OpValue, OpValueType};
use crate::order_by::Order;
use crate::pagination::{Cursor, CursorExt, Pagination};
use crate::pool::*;
use crate::types::ToSql;

pub struct CursorOrderByClause<'a, T>
where
    T: Model,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) params: &'a [&'a (dyn ToSql + Sync)],
    pub(crate) ops: &'a [Op<'a>],
    pub(crate) orders: Vec<(&'a str, Order)>,
    pub(crate) before: &'a Option<Cursor>,
    pub(crate) after: &'a Option<Cursor>,
    pub(crate) first: Option<i64>,
    pub(crate) last: Option<i64>,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> CursorOrderByClause<'a, T>
where
    T: Model,
{
    pub fn order_by(&'a mut self, order: &'a [(&'a str, Order)]) -> &'a mut Self {
        self.orders.extend_from_slice(order);
        self
    }

    pub fn build(&'a self) -> (String, i64, Vec<&'a (dyn ToSql + Sync)>) {
        let mut params_index = 1;

        let mut params = self.params.to_vec();
        let mut query = "SELECT ".to_string();
        query.push_str(&T::COLUMNS.join(", "));
        query.push_str(" FROM ");
        query.push_str(T::NAME);

        let mut ops = self.ops.to_vec();
        let mut orders = self.orders.clone();
        let mut limit = 20;
        if let Some(first) = self.first {
            limit = first;

            if let Some(after) = self.after {
                ops.push(Op::Value(OpValue {
                    ty: OpValueType::Value,
                    column: after.field.clone().into(),
                    op: ">=",
                    value: after,
                }));
                orders.push(("created_at", Order::Asc));
            } else if let Some(before) = self.before {
                ops.push(Op::Value(OpValue {
                    ty: OpValueType::Value,
                    column: before.field.clone().into(),
                    op: "<=",
                    value: before,
                }));
                orders.push(("created_at", Order::Desc));
            } else {
                orders.push(("created_at", Order::Asc));
            }
        }

        if let Some(last) = self.last {
            limit = last;

            if let Some(after) = self.after {
                ops.push(Op::Value(OpValue {
                    ty: OpValueType::Value,
                    column: after.field.clone().into(),
                    op: "<=",
                    value: after,
                }));
                orders.push(("created_at", Order::Desc));
            } else if let Some(before) = self.before {
                ops.push(Op::Value(OpValue {
                    ty: OpValueType::Value,
                    column: before.field.clone().into(),
                    op: ">=",
                    value: before,
                }));
                orders.push(("created_at", Order::Asc));
            } else {
                orders.push(("created_at", Order::Desc));
            }
        }

        let mut ands = vec![];
        for op in ops {
            let (q, p) = op.sql(&mut params_index);
            if let Some(q) = q {
                ands.push(q);
                params.extend_from_slice(&p);
            }
        }
        debug!("ands: {:?}", ands);
        if !ands.is_empty() {
            let and = ands.join(" AND ");
            query.push_str(" WHERE ");
            query.push_str(&and);
        }

        query.push_str(" ORDER BY ");
        let mut orders_str = vec![];
        for (column, order) in orders {
            orders_str.push(format!("{} {}", column, order.to_string()));
        }
        let order = orders_str.join(", ");
        query.push_str(&order);

        let delta = if self.after.is_some() || self.before.is_some() {
            2
        } else {
            1
        };
        query.push_str(format!(" LIMIT {}", limit + delta).as_str());

        (query, limit, params)
    }

    async fn count(&'a self) -> anyhow::Result<i64> {
        let query = format!("SELECT COUNT(*) FROM {}", T::NAME);
        let conn = self.pool.get().await?;
        let row = conn.query_one(query.as_str(), &[]).await?;

        row.try_get(0)
            .map_err(|e| anyhow::anyhow!("failed to get count: {}", e))
    }

    pub async fn all<T1>(&'a self) -> anyhow::Result<Pagination<Cursor, T1>>
    where
        T1: CursorExt<Cursor> + Model + Sync + Send,
    {
        let (query, limit, params) = self.build();
        debug!("query: {}, params: {:?}", query, params);
        let conn = self.pool.get().await?;

        let query_fut = conn
            .query(query.as_str(), &params)
            .map_err(|e| anyhow::anyhow!("failed to query: {}", e));
        let (rows, count) = tokio::try_join!(query_fut, self.count())?;

        let mut items = vec![];
        for row in rows {
            items.push(T1::from_row(row)?);
        }

        Ok(Pagination::new(
            items,
            self.before.clone(),
            self.after.clone(),
            limit,
            count,
        ))
    }
}
