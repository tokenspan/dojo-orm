use crate::types::ToSql;

pub trait Model {
    const NAME: &'static str;
    const COLUMNS: &'static [&'static str];
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
    fn from_row(row: tokio_postgres::Row) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub trait UpdateModel {
    const COLUMNS: &'static [&'static str];
    fn columns(&self) -> Vec<&'static str>;
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}
