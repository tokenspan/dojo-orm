use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::DerefMut;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use postgres_types::ToSql;
use tokio_postgres::NoTls;

use crate::cursor::order_by::CursorOrderByClause;
use crate::model::Model;
use crate::ops::{Op, OpValue};
use crate::order_by::{Order, OrderByClause};
use crate::pagination::Cursor;

pub struct WhereSelectClause<'a, T> {
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) params: Vec<&'a (dyn ToSql + Sync)>,
    pub(crate) ops: Vec<Op<'a>>,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> WhereSelectClause<'a, T>
where
    T: Model,
{
    pub fn order_by(&'a self, order: (&'a str, Order)) -> OrderByClause<'a, T> {
        OrderByClause {
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
        before: Option<&'a Cursor>,
        after: Option<&'a Cursor>,
    ) -> CursorOrderByClause<'a, T> {
        if let Some(value) = before {
            self.ops.push(Op::Value(OpValue {
                column: value.field.clone().into(),
                op: "<=",
                value,
            }));
        }

        if let Some(value) = after {
            self.ops.push(Op::Value(OpValue {
                column: value.field.clone().into(),
                op: ">=",
                value,
            }));
        }

        CursorOrderByClause {
            params: &self.params,
            ops: &self.ops,
            orders: vec![],
            after,
            _t: PhantomData,
        }
    }

    pub fn build(&'a self) -> (String, Vec<&'a (dyn ToSql + Sync)>) {
        let mut params_index = 1;

        let mut params = self.params.clone();
        let mut query = "SELECT ".to_string();
        query.push_str(&T::COLUMNS.join(", "));
        query.push_str(" FROM ");
        query.push_str(T::NAME);

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

        (query, params)
    }

    pub async fn execute(&'a mut self) -> anyhow::Result<Option<T>> {
        let (query, params) = self.build();
        println!("query: {}", query);
        println!("params: {:?}", params);
        let mut conn = self.pool.get().await?;
        let client = conn.deref_mut();

        client
            .query_opt(query.as_str(), &params)
            .await?
            .map(T::from_row)
            .transpose()
    }
}
