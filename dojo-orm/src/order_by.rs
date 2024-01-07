use async_graphql::Enum;
use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;
use tracing::debug;

use crate::limit::LimitClause;
use crate::model::Model;
use crate::ops::Op;
use crate::pool::*;
use crate::types::ToSql;

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Order {
    #[graphql(name = "asc")]
    Asc,
    #[graphql(name = "desc")]
    Desc,
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::Asc => write!(f, "ASC"),
            Order::Desc => write!(f, "DESC"),
        }
    }
}

pub struct OrderByClause<'a, T>
where
    T: Model,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) params: &'a [&'a (dyn ToSql + Sync)],
    pub(crate) ops: &'a [Op<'a>],
    pub(crate) orders: HashMap<&'a str, Order>,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> OrderByClause<'a, T>
where
    T: Model,
{
    pub fn limit(&'a mut self, limit: i32) -> LimitClause<'a, T> {
        LimitClause {
            params: &self.params,
            ops: &self.ops,
            orders: Some(&self.orders),
            limit,
            _t: PhantomData,
        }
    }

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
            if let Some(q) = q {
                ands.push(q);
                params.extend_from_slice(&p);
            }
        }
        if !query.is_empty() {
            let and = ands.join(" AND ");
            query.push_str(" WHERE ");
            query.push_str(&and);
        }

        let mut orders = vec![];
        for (column, order) in &self.orders {
            orders.push(format!("{} {}", column, order.to_string()));
        }
        if !orders.is_empty() {
            let order = orders.join(", ");
            query.push_str(" ORDER BY ");
            query.push_str(&order);
        }

        query.push_str(" LIMIT 1");

        (query, params)
    }

    pub async fn first(&'a mut self) -> anyhow::Result<Option<T>> {
        let (query, params) = self.build();
        debug!("query: {}, params: {:?}", query, params);
        let conn = self.pool.get().await?;

        conn.query_opt(query.as_str(), &params)
            .await?
            .map(T::from_row)
            .transpose()
    }
}
