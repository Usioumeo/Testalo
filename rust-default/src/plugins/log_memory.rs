use std::{sync::Arc, time::Duration};

use memory_stats::memory_stats;
use orchestrator::{
    executor::ExecutorGlobalState, orchestrator::OrchestratorReference, plugin::Plugin,
};
use tokio::sync::Notify;

pub struct LogMemory;
impl<S: ExecutorGlobalState> Plugin<S> for LogMemory {
    fn name(&self) -> &str {
        "Memory Logger"
    }

    fn desctiption(&self) -> &str {
        "Logs currently used memory by the application"
    }
    async fn run(self, _o: OrchestratorReference<S>, _should_stop: Arc<Notify>) {
        loop {
            if let Some(m) = memory_stats() {
                println!("{} {}", m.virtual_mem, m.physical_mem);
            }
            tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
        }
    }
}
