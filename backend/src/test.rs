use orchestrator::{
    default_memory::DefaultMemory,
    prelude::*,
    test::TestInterface,
    GenerateState,
};
use reqwest::Client;
use rocket::async_test;
use std::error::Error;

use crate::WebServer;
struct BackendTest {
    url: String,
    client: Client,
}
impl BackendTest {
    fn new(url: &str) -> Self {
        let client = Client::builder().cookie_store(true).build().unwrap();
        Self {
            url: url.to_string(),
            client,
        }
    }
}

#[async_trait]
impl TestInterface for BackendTest {
    async fn register(&mut self, username: &str, password: &str) {
        let params = [("username", username), ("password", password)];
        let res = self
            .client
            .post(&format!("{}/register", self.url))
            .form(&params)
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success())
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
        let params = [("username", username), ("password", password)];
        let res = self
            .client
            .post(&format!("{}/login", self.url))
            .form(&params)
            .send()
            .await
            .unwrap();
        res.error_for_status_ref()?;

        Ok(())
    }

    async fn submit(
        &mut self,
        problem: String,
        source: String,
    ) -> Result<ExerciseResult, Box<dyn Error + Send + Sync + 'static>> {
        let params = [("problem", problem), ("source", source)];
        let res = self
            .client
            .post(&format!("{}/submit", self.url))
            .form(&params)
            .send()
            .await
            .unwrap();
        res.error_for_status_ref()?;
        let ret = res.text().await?;
        let ex: ExerciseResult = serde_json::from_str(&ret)?;
        Ok(ex)
    }
    async fn list_exercise(&mut self) -> Result<Vec<String>, Box<dyn Error + 'static>> {
        let body = reqwest::get(&format!("{}/list_problems", self.url))
            .await?
            .text()
            .await?;
        let deserialized: Vec<String> = serde_json::from_str(&body).unwrap();
        Ok(deserialized)
    }
}
#[async_test]
async fn test_backend() {
    GenerateState!(ExerciseResult, DummyExercise);
    let m = DefaultMemory::init();
    let mut o: Orchestrator<State> = Orchestrator::new(16, m);
    let test = BackendTest::new("http://localhost:8000");
    let def = DefaultTest::new(test);
    o.add_plugin(def).await.unwrap();
    o.add_plugin(WebServer).await.unwrap();
    let _ = o.run().await;
}
