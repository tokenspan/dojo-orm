use dojo_macros::EmbeddedModel;

fn test_embedded() {
    #[derive(EmbeddedModel)]
    struct Profile {
        age: i32,
        address: String,
    }
}
