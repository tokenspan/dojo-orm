use chrono::{NaiveDateTime, Utc};
use googletest::prelude::*;
use googletest::{assert_that, pat};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::*;
use dojo_macros::Model;
use dojo_orm::predicates::{and, eq as eq_pred, in_list};
use dojo_orm::Database;

mod common;

#[tokio::test]
async fn test_insert_1() -> anyhow::Result<()> {
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

    let input = User {
        id: Uuid::new_v4(),
        name: "linh12".to_string(),
        email: "linh12@gmail.com".to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    let user = db.insert(&input).await?;
    assert_that!(
        user,
        pat!(User {
            id: anything(),
            name: eq("linh12".to_string()),
            email: eq("linh12@gmail.com".to_string()),
            created_at: anything(),
            updated_at: anything(),
        })
    );

    let user = db
        .bind::<User>()
        .where_by(and(&[eq_pred("id", &user.id)]))
        .first()
        .await?;
    assert_that!(
        user,
        some(pat!(User {
            id: anything(),
            name: eq("linh12".to_string()),
            email: eq("linh12@gmail.com".to_string()),
            created_at: anything(),
            updated_at: anything(),
        }))
    );

    Ok(())
}

#[tokio::test]
async fn test_insert_many_2() -> anyhow::Result<()> {
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

    let input1 = User {
        id: Uuid::new_v4(),
        name: "linh12".to_string(),
        email: "linh12@gmail.com".to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    let input2 = User {
        id: Uuid::new_v4(),
        name: "linh13".to_string(),
        email: "linh13@gmail.com".to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    let users = db.insert_many(&[input1, input2]).await?;
    assert_that!(
        users,
        contains_each![
            pat!(User {
                id: anything(),
                name: eq("linh12".to_string()),
                email: eq("linh12@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            }),
            pat!(User {
                id: anything(),
                name: eq("linh13".to_string()),
                email: eq("linh13@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            })
        ]
    );

    let users = db
        .bind::<User>()
        .where_by(and(&[in_list("id", &[&users[0].id, &users[1].id])]))
        .limit(2)
        .await?;
    assert_that!(
        users,
        contains_each![
            pat!(User {
                id: anything(),
                name: eq("linh12".to_string()),
                email: eq("linh12@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            }),
            pat!(User {
                id: anything(),
                name: eq("linh13".to_string()),
                email: eq("linh13@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            })
        ]
    );

    Ok(())
}
