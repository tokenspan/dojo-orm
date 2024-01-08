use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::*;
use dojo_macros::Model;
use dojo_orm::Database;

mod common;

macro_rules! create_user {
    ($db: ident, names = $($name:literal),+) => {
        $db.insert_many(&[
            $(User {
                id: Uuid::new_v4(),
                name: $name.to_string(),
                email: concat!($name, "@gmail.com").to_string(),
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            }),+
        ]).await?;
    };
}

macro_rules! create_paging_args {
    (first = $first: literal) => {
        (Some($first as i64), None, None, None)
    };
    (first = $first: literal, after = $after: literal) => {
        (Some($first as i64), Some($after), None, None)
    };
    (last = $last: literal) => {
        (None, None, Some($last as i64), None)
    };
    (last = $last: literal, before = $before: literal) => {
        (None, None, Some($last as i64), Some($before))
    };
}

#[tokio::test]
async fn test_paging_forward() -> anyhow::Result<()> {
    let db: Database;
    setup!(db);

    #[derive(Serialize, Deserialize, Debug, Model)]
    #[dojo(name = "users", sort_keys = ["created_at", "id"])]
    struct User {
        id: Uuid,
        name: String,
        email: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    }

    create_user!(db, names = "linh1", "linh2", "linh3", "linh4", "linh5");

    let (first, after, last, before) = create_paging_args!(first = 1);
    db.bind::<User>().cursor(first, after, last, before).await?;

    Ok(())
}
