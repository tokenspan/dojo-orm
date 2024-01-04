use std::marker::PhantomData;

use async_graphql::futures_util::TryFutureExt;
use tracing::debug;

use crate::model::Model;
use crate::ops::Op;
use crate::order_by::Order;
use crate::pagination::{Cursor, CursorExt, Pagination};
use crate::pool::*;
use crate::types::ToSql;

pub struct CursorLimitClause<'a, T>
where
    T: Model,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) params: &'a [&'a (dyn ToSql + Sync)],
    pub(crate) ops: &'a [Op<'a>],
    pub(crate) orders: &'a [(&'a str, Order)],
    pub(crate) before: &'a Option<Cursor>,
    pub(crate) after: &'a Option<Cursor>,
    pub(crate) limit: i64,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> CursorLimitClause<'a, T>
where
    T: Model,
{
    pub fn build(&'a self) -> (String, Vec<&'a (dyn ToSql + Sync)>) {
        let mut params_index = 1;

        let mut params = self.params.to_vec();
        let mut query = "SELECT ".to_string();
        query.push_str(&T::COLUMNS.join(", "));
        query.push_str(" FROM ");
        query.push_str(T::NAME);

        let mut ands = vec![];
        for op in self.ops {
            let (q, p) = op.sql(&mut params_index);
            ands.push(q);
            params.extend_from_slice(&p);
        }
        if !ands.is_empty() {
            let and = ands.join(" AND ");
            query.push_str(" WHERE ");
            query.push_str(&and);
        }

        let limit = self.limit + if self.after.is_some() { 2 } else { 1 };
        query.push_str(format!(" LIMIT {}", limit).as_str());

        (query, params)
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
        let (query, params) = self.build();
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
            self.limit,
            count,
        ))
    }
}
