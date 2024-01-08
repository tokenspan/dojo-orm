use crate::execution::Execution;
use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::debug;

use crate::model::{Model, UpdateModel};
use crate::pagination::Cursor;
use crate::pool::*;
use crate::predicates::eq;
use crate::query_builder::{QueryBuilder, QueryType};
use crate::where_delete::WhereDelete;
use crate::where_select::WhereSelect;
use crate::where_update::WhereUpdate;

#[derive(Clone)]
pub struct Database {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Database {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let manager = PostgresConnectionManager::new_from_stringlike(url, NoTls)?;
        let pool = Pool::builder().build(manager).await?;

        Ok(Self { pool })
    }

    pub async fn get(&self) -> anyhow::Result<PooledConnection<PostgresConnectionManager<NoTls>>> {
        Ok(self.pool.get().await?)
    }

    pub fn bind<T>(&self) -> WhereSelect<T>
    where
        T: Model + Debug,
    {
        WhereSelect {
            pool: &self.pool,
            columns: T::COLUMNS,
            params: vec![],
            predicates: vec![],
            _t: PhantomData::<T>,
        }
    }

    pub async fn insert<T>(&self, data: &T) -> anyhow::Result<T>
    where
        T: Model + Debug,
    {
        let params = data.params();
        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(T::COLUMNS)
            .params(&params.as_slice())
            .ty(QueryType::Insert)
            .is_returning(true)
            .build();

        let execution: Execution<T> = Execution::new(&self.pool, &qb);
        execution.first_or_throw().await
    }

    pub async fn insert_many<T>(&self, data: &[T]) -> anyhow::Result<Vec<T>>
    where
        T: Model + Debug,
    {
        let mut params = vec![];
        for d in data {
            params.extend(d.params());
        }

        let qb = QueryBuilder::builder()
            .table_name(T::NAME)
            .columns(T::COLUMNS)
            .params(&params.as_slice())
            .ty(QueryType::Insert)
            .is_returning(true)
            .build();

        let execution: Execution<T> = Execution::new(&self.pool, &qb);
        execution.all().await
    }

    pub fn update<'a, T, U>(&'a self, data: &'a U) -> WhereUpdate<'a, T>
    where
        T: Model + Debug,
        U: UpdateModel + Debug,
    {
        WhereUpdate {
            pool: &self.pool,
            params: data.params(),
            predicates: vec![],
            _t: PhantomData,
        }
    }

    pub fn delete<T>(&self) -> WhereDelete<T>
    where
        T: Model + Debug,
    {
        WhereDelete {
            pool: &self.pool,
            predicates: vec![],
            _t: PhantomData,
        }
    }
}
