use std::collections::HashMap;
use std::marker::PhantomData;

use crate::limit::LimitClause;
use postgres_types::ToSql;

use crate::model::Model;
use crate::ops::Op;

#[derive(Debug)]
pub enum Order {
    Asc,
    Desc,
}

pub struct OrderByClause<'a, T>
where
    T: Model,
{
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
}
