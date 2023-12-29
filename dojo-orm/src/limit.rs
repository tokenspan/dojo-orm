use std::collections::HashMap;
use std::marker::PhantomData;

use postgres_types::ToSql;

use crate::model::Model;
use crate::ops::Op;
use crate::order_by::Order;

pub struct LimitClause<'a, T>
where
    T: Model,
{
    pub(crate) params: &'a [&'a (dyn ToSql + Sync)],
    pub(crate) ops: &'a [Op<'a>],
    pub(crate) orders: Option<&'a HashMap<&'a str, Order>>,
    pub(crate) limit: i32,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> LimitClause<'a, T> where T: Model {}
