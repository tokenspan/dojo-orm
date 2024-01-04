use std::marker::PhantomData;
use tracing::debug;

use crate::model::{Model, UpdateModel};
use crate::pagination::{Cursor, CursorExt};
use crate::pool::*;
use crate::where_select::WhereSelectClause;
use crate::where_update::WhereUpdateClause;

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

    pub fn bind<T>(&self) -> WhereSelectClause<T>
    where
        T: Model + CursorExt<Cursor> + Sync + Send,
    {
        WhereSelectClause {
            pool: &self.pool,
            is_delete: false,
            params: vec![],
            ops: vec![],
            _t: PhantomData,
        }
    }

    pub async fn insert<T: Model>(&self, data: &T) -> anyhow::Result<T> {
        let mut query = "INSERT INTO ".to_string();
        query.push_str(T::NAME);
        query.push_str(" (");
        query.push_str(T::COLUMNS.join(", ").as_str());
        query.push_str(") VALUES (");

        let mut params_index = 1;
        let mut params = vec![];
        let mut values = vec![];
        for param in data.params() {
            values.push(format!("${}", params_index));
            params.push(param);
            params_index += 1;
        }
        query.push_str(values.join(", ").as_str());
        query.push_str(") RETURNING ");
        query.push_str(T::COLUMNS.join(", ").as_str());

        let conn = self.pool.get().await?;
        debug!("query: {}, params: {:?}", query, params);
        let row = conn.query_one(query.as_str(), &params).await?;

        Ok(T::from_row(row)?)
    }

    pub fn update<'a, T: Model, U: UpdateModel>(
        &'a self,
        data: &'a U,
    ) -> WhereUpdateClause<'a, T, U> {
        let params = data.params();
        let columns = data.columns();
        WhereUpdateClause {
            pool: &self.pool,
            columns: columns.clone(),
            params: params.clone(),
            ops: vec![],
            _t: PhantomData,
            _u: PhantomData,
        }
    }

    pub fn delete<T: Model>(&self) -> WhereSelectClause<T> {
        WhereSelectClause {
            pool: &self.pool,
            is_delete: true,
            params: vec![],
            ops: vec![],
            _t: PhantomData,
        }
    }
}
