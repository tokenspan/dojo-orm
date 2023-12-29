use std::marker::PhantomData;
use std::ops::DerefMut;

use crate::pool::*;
use crate::types::ToSql;

use crate::limit::LimitClause;
use crate::model::{Model, UpdateModel};
use crate::ops::Op;

pub struct WhereUpdateClause<'a, T, U>
where
    T: Model,
    U: UpdateModel,
{
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) params: Vec<&'a (dyn ToSql + Sync)>,
    pub(crate) ops: Vec<Op<'a>>,
    pub(crate) _t: PhantomData<T>,
    pub(crate) _u: PhantomData<U>,
}

impl<'a, T, U> WhereUpdateClause<'a, T, U>
where
    T: Model,
    U: UpdateModel,
{
    fn limit(&'a self, limit: i32) -> LimitClause<'a, T> {
        LimitClause {
            params: &self.params,
            ops: &self.ops,
            orders: None,
            limit,
            _t: PhantomData,
        }
    }

    pub fn where_by(&'a mut self, op: Op<'a>) -> &'a mut Self {
        self.ops.push(op);
        self
    }

    fn build(&'a self) -> (String, Vec<&'a (dyn ToSql + Sync)>) {
        let mut params_index = 1;

        let mut params = self.params.clone();
        let mut query = "UPDATE ".to_string();
        query.push_str(T::NAME);
        query.push_str(" SET ");

        let mut sets = vec![];
        for column in U::COLUMNS {
            sets.push(format!("{} = ${}", column, params_index));
            params_index += 1;
        }
        query.push_str(&sets.join(", "));

        let mut ands = vec![];
        for op in &self.ops {
            let (q, p) = op.sql(&mut params_index);
            ands.push(q);
            params.extend_from_slice(&p);
        }
        if !query.is_empty() {
            let and = ands.join(" AND ");
            query.push_str(" WHERE ");
            query.push_str(&and);
        }

        query.push_str(" RETURNING ");
        query.push_str(&T::COLUMNS.join(", "));

        (query, params)
    }

    pub async fn first(&'a self) -> anyhow::Result<Option<T>> {
        let (query, params) = self.build();
        let mut conn = self.pool.get().await?;
        let client = conn.deref_mut();

        client
            .query_opt(query.as_str(), &params)
            .await?
            .map(T::from_row)
            .transpose()
    }
}
