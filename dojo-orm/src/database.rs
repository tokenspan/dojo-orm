use crate::model::{Model, UpdateModel};
use crate::where_select::WhereSelectClause;
use crate::where_update::WhereUpdateClause;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use std::marker::PhantomData;
use tokio_postgres::{NoTls, Row};

pub struct Database<'a> {
    pool: &'a Pool<PostgresConnectionManager<NoTls>>,
}

impl<'a> Database<'a> {
    pub fn new(pool: &'a Pool<PostgresConnectionManager<NoTls>>) -> Self {
        Self { pool }
    }

    pub fn bind<T: Model>(&'a self) -> WhereSelectClause<'a, T> {
        WhereSelectClause {
            pool: &self.pool,
            params: vec![],
            ops: vec![],
            _t: PhantomData,
        }
    }

    pub async fn insert<T: Model + Clone + From<Row>>(&'a self, data: &'a T) -> anyhow::Result<T> {
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

        let mut conn = self.pool.get().await?;
        let row = conn.query_one(query.as_str(), &params).await?.into();

        Ok(row)
    }

    pub fn update<T: Model, U: UpdateModel>(&'a self, data: &'a U) -> WhereUpdateClause<'a, T, U> {
        let params = data.params();
        WhereUpdateClause {
            pool: &self.pool,
            params: params.clone(),
            ops: vec![],
            _t: PhantomData,
            _u: PhantomData,
        }
    }
}
