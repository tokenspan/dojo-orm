use std::marker::PhantomData;

use postgres_types::ToSql;

use crate::model::Model;
use crate::ops::Op;
use crate::order_by::Order;
use crate::pagination::Cursor;

pub struct CursorLimitClause<'a, T>
where
    T: Model,
{
    pub(crate) params: &'a [&'a (dyn ToSql + Sync)],
    pub(crate) ops: &'a [Op<'a>],
    pub(crate) orders: &'a [(&'a str, Order)],
    pub(crate) after: Option<&'a Cursor<'a>>,
    pub(crate) limit: i32,
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

        println!("query: {}", query);
        println!("params: {:?}", params);
        (query, params)
    }
}
