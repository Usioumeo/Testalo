//! Utility uses to perform load-test on the Backend crate.

use goose::prelude::*;
use goose_eggs::{validate_and_load_static_assets, Validate};
/// submit an exercise
async fn loadtest_index(user: &mut GooseUser) -> TransactionResult {
    let params = [
        ("problem", "es1"),
        (
            "source",
            "fn bigger(x: i32, y: i32)->i32{
        if x>y{
            x
        }else{
            y
        }
    }",
        ),
    ];
    let goose = user.post_form("/submit", &params).await?;
    let validate = &Validate::builder()
        .status(200)
        //.text("Gander")
        .build();

    validate_and_load_static_assets(user, goose, validate)
        .await
        .unwrap();
    Ok(())
}
/// make a login to the backend, in order to submit many exercises
async fn website_login(user: &mut GooseUser) -> TransactionResult {
    let id = user.weighted_users_index;
    let name = format!("goose_{id}");
    let params = [("username", name.as_str()), ("password", "mondo")];
    let _goose = user.post_form("/register", &params).await.unwrap();

    let params = [("username", name.as_str()), ("password", "mondo")];
    let _goose = user.post_form("/login", &params).await.unwrap();

    Ok(())
}

#[tokio::main]
/// Initialize and starts a GooseAttack
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .set_default(GooseDefault::Host, "http://localhost:8000")
        .unwrap()
        .set_default(GooseDefault::Users, 1000)
        .unwrap()
        //.set_default(GooseDefault::StartupTime, 5).unwrap()
        .register_scenario(
            scenario!("LoadtestTransactions")
                .register_transaction(transaction!(website_login).set_on_start())
                .register_transaction(transaction!(loadtest_index)),
        )
        .execute()
        .await?;

    Ok(())
}
