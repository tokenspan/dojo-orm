use std::fmt::Display;
use tracing::Level;

use uuid::Uuid;

use dojo_macros::Model;
use dojo_orm::ops::{and, in_list};
use dojo_orm::pagination::{Cursor, CursorExt};
use dojo_orm::Database;

#[tokio::test]
async fn test_in_list() {
    tracing_subscriber::fmt().init();
    let url = "postgresql://postgres:123456@localhost:5432/tokenspan";
    let db = Database::new(url).await.unwrap();

    #[derive(Debug, Model)]
    #[dojo(name = "users")]
    struct User {
        id: Uuid,
        username: String,
        email: String,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    impl CursorExt<Cursor> for User {
        fn cursor(&self) -> Cursor {
            Cursor::new("created_at".to_string(), self.created_at.timestamp_micros())
        }
    }

    let ids = &[Uuid::parse_str("e5fe58fc-89f9-416a-a97f-97820fbbd8dd").unwrap()];
    let users = db
        .bind::<User>()
        .where_by(and(&[in_list("id", ids)]))
        .all()
        .await
        .unwrap();
    println!("{:?}", users)
}
