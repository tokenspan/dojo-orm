## Dojo ORM

### Installation
```toml
[dependencies]
dojo-orm = { git = "https://github.com/tokenspan/dojo-orm" }
```

### Usage
```rust
#[tokio::main]
async fn main() {
    #[derive(Debug, Type)]
    #[postgres(name = "user_role", rename_all = "lowercase")]
    enum UserRole {
        Admin,
        User,
    }

    #[derive(Debug, EmbeddedModel)]
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
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    }

    let user = db.insert(&input).await.unwrap();
}
```
