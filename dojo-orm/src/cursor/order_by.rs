use std::marker::PhantomData;

use crate::cursor::limit::CursorLimitClause;
use postgres_types::ToSql;

use crate::model::Model;
use crate::ops::Op;
use crate::order_by::Order;
use crate::pagination::Cursor;

pub struct CursorOrderByClause<'a, T>
where
    T: Model,
{
    pub(crate) params: &'a [&'a (dyn ToSql + Sync)],
    pub(crate) ops: &'a [Op<'a>],
    pub(crate) orders: Vec<(&'a str, Order)>,
    pub(crate) after: Option<&'a Cursor<'a>>,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> CursorOrderByClause<'a, T>
where
    T: Model,
{
    pub fn limit(&'a mut self, limit: i32) -> CursorLimitClause<'a, T> {
        CursorLimitClause {
            params: &self.params,
            ops: &self.ops,
            orders: &self.orders,
            limit,
            after: self.after,
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
            ands.push(q);
            params.extend_from_slice(&p);
        }
        if !query.is_empty() {
            let and = ands.join(" AND ");
            query.push_str(" WHERE ");
            query.push_str(&and);
        }

        println!("query: {}", query);
        println!("params: {:?}", params);
        (query, params)
    }
}
