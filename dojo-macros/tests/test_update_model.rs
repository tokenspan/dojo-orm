use dojo_macros::UpdateModel;

#[test]
fn test_update_model() {
    #[derive(UpdateModel)]
    struct UpdateUser {
        name: Option<String>,
    }
}
