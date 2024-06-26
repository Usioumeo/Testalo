use orchestrator::{
    prelude::*,
    test::{DefaultTest, DummyExercise},
    GenerateState,
};

use crate::Postgres;

#[tokio::test]
async fn test_memory() {
    GenerateState!(ExerciseResult, DummyExercise);
    let m = Box::new(
        Postgres::clean_init("postgresql://postgres:test@localhost:5432/thesis")
            .await
            .unwrap(),
    );
    let mut o: Orchestrator<State> = Orchestrator::new(16, m);

    let def = DefaultTest::new_default();
    o.add_plugin(def).await.unwrap();
    let _ = o.run().await;
}
