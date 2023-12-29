use chrono::NaiveDateTime;
use dojo_macros::Model;
use uuid::Uuid;

#[test]
fn test_expand_model() {
    #[derive(Model)]
    #[dojo(name = "users")]
    struct User {
        id: Uuid,
        name: String,
        email: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    }
}
