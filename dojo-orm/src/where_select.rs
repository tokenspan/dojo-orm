use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::execution;
use crate::execution::Execution;
use tracing::debug;

use crate::model::Model;
use crate::pagination::{Cursor, DefaultSortKeys};
use crate::pool::*;
use crate::predicates::{Expr, ExprValueType, Predicate};
use crate::query_builder::{QueryBuilder, QueryType};
use crate::types::ToSql;

pub struct WhereSelect<'a, T> {
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) params: Vec<&'a (dyn ToSql + Sync)>,
    pub(crate) columns: &'a [&'a str],
    pub(crate) predicates: Vec<Predicate<'a>>,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> WhereSelect<'a, T>
where
    T: Model + Debug,
{
    pub fn where_by(&'a mut self, predicate: Predicate<'a>) -> &'a mut Self {
        self.predicates.push(predicate);
        self
    }

    pub async fn cursor(
        &'a self,
        first: Option<i64>,
        after: Option<Cursor>,
        last: Option<i64>,
        before: Option<Cursor>,
    ) -> anyhow::Result<Vec<T>> {
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .default_keys(T::sort_keys())
            .columns(self.columns)
            .params(&self.params.as_slice())
            .predicates(&self.predicates.as_slice())
            .first(first)
            .after(&after)
            .last(last)
            .before(&before)
            .ty(QueryType::Paging)
            .build();

        let execution: Execution<T> = Execution::new(&self.pool, &qb);
        execution.all().await
    }

    pub async fn limit(&'a self, limit: i64) -> anyhow::Result<Vec<T>> {
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(self.columns)
            .params(&self.params.as_slice())
            .predicates(&self.predicates.as_slice())
            .ty(QueryType::Select)
            .limit(limit)
            .build();

        let execution: Execution<T> = Execution::new(&self.pool, &qb);
        execution.all().await
    }

    pub async fn first(&'a self) -> anyhow::Result<Option<T>> {
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(self.columns)
            .params(&self.params.as_slice())
            .predicates(&self.predicates.as_slice())
            .ty(QueryType::Select)
            .limit(1)
            .build();

        let execution: Execution<T> = Execution::new(&self.pool, &qb);
        execution.first().await
    }
}
