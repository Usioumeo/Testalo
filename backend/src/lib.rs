use orchestrator::prelude::*;

use rocket::fs::{FileServer, NamedFile};
use rocket::response::status::NotFound;
use rocket::tokio::sync::Notify;
use rocket::*;

mod auth;
mod problems;
#[cfg(test)]
mod test;
/// Return the index file as a Rocket NamedFile
async fn get_index() -> Result<NamedFile, NotFound<String>> {
    NamedFile::open("../frontend/dist/index.html")
        .await
        .map_err(|e| NotFound(e.to_string()))
}

/// Return the index when the url is /
#[get("/")]
async fn index() -> Result<NamedFile, NotFound<String>> {
    get_index().await
}
/// this is a fallback. if the path is not known we return the index
#[get("/<_..>", rank = 11)]
async fn fallback() -> Result<NamedFile, NotFound<String>> {
    get_index().await
}

/// start the server
pub async fn run_server<S: ExecutorGlobalState>(o: OrchestratorReference<S>) {
    let state: Box<dyn ReferenceWithoutState> = Box::new(o);
    let _ = rocket::build()
        .manage(state)
        .mount("/", routes![index, fallback])
        .mount("/", auth::routes())
        .mount("/", problems::routes())
        .mount("/static", FileServer::from("../frontend/dist"))
        .launch()
        .await;
}

/// plugin
pub struct WebServer;

impl<S: ExecutorGlobalState> Plugin<S> for WebServer {
    fn name(&self) -> &str {
        "rocket webserver"
    }

    fn desctiption(&self) -> &str {
        "generate a rocket webserver, and use it to get request from the users.
        In addition to that it host a static wasm frontend. (Got from Frontend crate)
        "
    }
    /// run the plugin
    async fn run(self, o: OrchestratorReference<S>, should_stop: std::sync::Arc<Notify>) {
        let q = tokio::spawn(async { run_server(o).await });
        let _ = q.await;
        should_stop.notify_one();
        //todo!()
    }
}
