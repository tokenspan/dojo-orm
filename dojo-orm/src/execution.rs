use std::fmt::Debug;
use std::marker::PhantomData;

use anyhow::Result;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use tracing::{debug, info};

use crate::query_builder::QueryBuilder;
use crate::Model;

pub struct Execution<'a, T> {
    pub(crate) pool: &'a Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) qb: &'a QueryBuilder<'a>,
    pub(crate) _t: PhantomData<T>,
}

impl<'a, T> Execution<'a, T>
where
    T: Model + Debug,
{
    pub fn new(pool: &'a Pool<PostgresConnectionManager<NoTls>>, qb: &'a QueryBuilder<'a>) -> Self {
        Self {
            pool,
            qb,
            _t: PhantomData,
        }
    }

    pub async fn first_or_throw(&self) -> Result<T> {
        let conn = self.pool.get().await?;
        let (stmt, params) = self.qb.build_sql()?;
        let row = conn.query_one(&stmt, &params).await?;
        let record = T::from_row(row)?;

        info!(stmt);
        info!(?record);

        Ok(record)
    }

    pub async fn first(&self) -> Result<Option<T>> {
        let conn = self.pool.get().await?;
        let (stmt, params) = self.qb.build_sql()?;
        let record = conn.query_opt(&stmt, &params).await?.map(T::from_row);
        let record = record.transpose()?;

        info!(stmt);
        info!(?record);

        Ok(record)
    }

    pub async fn all(&self) -> Result<Vec<T>> {
        let conn = self.pool.get().await?;
        let (stmt, params) = self.qb.build_sql()?;
        let rows = conn.query(&stmt, &params).await?;
        let mut records = vec![];
        for row in rows {
            records.push(T::from_row(row)?);
        }

        info!(stmt);
        info!(?records);

        Ok(records)
    }
}
