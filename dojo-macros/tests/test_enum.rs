use dojo_macros::Type;

#[test]
fn test_enum() {
    #[derive(Debug, Type)]
    #[dojo(name = "user_role", rename_all = "lowercase")]
    enum UserRole {
        Admin,
        User,
    }
}
