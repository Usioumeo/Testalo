use orchestrator::prelude::*;
use sqlx::{query, Pool};
use std::error::Error;

/*
CREATE TABLE IF NOT EXISTS test_results(
    test_results_id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    compiled TEXT NOT NULL,
    runned TEXT NOT NULL,
    points FLOAT NOT NULL,
    refers_to BIGINT,
    FOREIGN KEY (refers_to) REFERENCES Sumbissions(submission_id),
); */
pub async fn add_test_result(
    pool: &Pool<sqlx::Postgres>,
    name: String,
    result: TestResult,
    submission_id: i64,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let compiled = serde_json::to_string(&result.compiled)?;
    let runned = serde_json::to_string(&result.runned)?;
    query("INSERT INTO test_results(name, compiled, runned, points, refers_to) VALUES ($1, $2, $3, $4, $5)")
        .bind(name)
        .bind(compiled)
        .bind(runned)
        .bind(result.points_given)
        .bind(submission_id)
        .execute(pool)
        .await?;
    Ok(())
}
