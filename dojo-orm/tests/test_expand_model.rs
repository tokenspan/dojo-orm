use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use dojo_macros::{EmbeddedModel, Model};
use dojo_orm::Model;

#[test]
fn test_expand_model() {
    #[derive(Debug, Serialize, Deserialize, EmbeddedModel)]
    struct Profile {
        address: String,
    }

    #[derive(Model)]
    #[dojo(name = "users")]
    struct User {
        name: String,
        email: String,
        profile: Profile,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    }

    let user = User {
        name: "John Doe".to_string(),
        email: "linh@gmail.com".to_string(),
        profile: Profile {
            address: "Hanoi".to_string(),
        },
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    let profile = user.get_value("profile");
    println!("profile: {:?}", profile);
}
