use std::error::Error;
use std::fmt::Display;
use std::io::Read;
use std::ops::DerefMut;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bytes::BufMut;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use uuid::Uuid;

use dojo_macros::{EmbeddedModel, Model, UpdateModel};
use dojo_orm::database::Database;
use dojo_orm::ops::{and, eq};
use dojo_orm::{Model, UpdateModel};

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./tests/migrations");
}

#[tokio::test]
async fn test_simple() {
    let url = "postgresql://postgres:123456@localhost:5432/test";
    let manager = PostgresConnectionManager::new_from_stringlike(url, NoTls).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    let mut conn = pool.get().await.unwrap();

    let client = conn.deref_mut();
    embedded::migrations::runner()
        .run_async(client)
        .await
        .unwrap();

    #[derive(Debug, ToSql, FromSql)]
    #[postgres(name = "user_role", rename_all = "lowercase")]
    enum UserRole {
        Admin,
        User,
    }

    #[derive(Debug, Deserialize, Serialize, EmbeddedModel)]
    struct Profile {
        age: i32,
        address: String,
    }

    #[derive(Debug, Model)]
    #[dojo(name = "users")]
    struct User {
        id: Uuid,
        name: String,
        email: String,
        profile: Profile,
        role: UserRole,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    let input = User {
        id: Uuid::new_v4(),
        name: "linh1".to_string(),
        email: "linh1@gmail.com".to_string(),
        role: UserRole::Admin,
        profile: Profile {
            age: 20,
            address: "Tokyo".to_string(),
        },
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    #[derive(Debug, ToSql, FromSql)]
    struct UserUpdateInput {
        name: Option<String>,
        email: Option<String>,
        profile: Option<Profile>,
        role: Option<UserRole>,
    }

    #[derive(UpdateModel)]
    struct UpdateUser {
        name: Option<String>,
    }

    let db = Database::new(&pool);

    let user = db.insert(&input).await.unwrap();
    println!("user: {:?}", user);

    // let input = UpdateUser {
    //     name: Some("John1".to_string()),
    // };
    // let id = Uuid::parse_str("ae686215-9676-4657-b239-339699049f28").unwrap();
    // let row = db
    //     .update::<User, _>(&input)
    //     .where_by(and(vec![eq("id", &id)]))
    //     .execute()
    //     .await
    //     .unwrap();
    // println!("row: {:?}", row);

    // let id = Uuid::parse_str("c4cf875a-7861-4ae8-a9ff-21d040ed0d7b").unwrap();
    // let cursor = Cursor::new("created_at", 1);
    // db.bind::<User>().cursor(None, None).limit(1).build();

    // let user = db
    //     .bind::<User>()
    //     .where_by(eq("id", &id))
    //     .execute()
    //     .await
    //     .unwrap();
    // println!("user: {:?}", user);
}
