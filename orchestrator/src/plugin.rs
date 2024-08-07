//! This module contains the definition of a Plugin, and how to wrap it inside the Orchestrator
//!
use std::{error::Error, future::Future, marker::PhantomData, mem, sync::Arc};

use async_trait::async_trait;

use crate::prelude::*;

/// Plugin interface:
///
/// contains name and description methods, and they are needed for visualizzation purpose.
///
/// The fondamental method is run, that takes an OrchestratorReference.
/// It can gracefully shutdown all program by notifing "should_stop"
///
/// Then there is on_add, and is executed while the plugin is getting add.
pub trait Plugin<S: ExecutorGlobalState>: Sized + Send + Sync + 'static {
    /// Return the name of the Plugin, used for Error creation and Debug
    fn name(&self) -> &str;

    /// Return a descriptionn of the Plugin. It should contain a list of the registered and activated Executors
    fn desctiption(&self) -> &str;

    /// Function called when the Plugin is runned (at the end of the init phase).
    /// it takes an OrchestratorReference, which permit almost complete control over the orchestrator, and a shared Notify.
    /// This notify should get called when a Plugin request an orchestrator shutdown.
    fn run(
        self,
        o: OrchestratorReference<S>,
        should_stop: Arc<Notify>,
    ) -> impl Future<Output = ()> + Send + 'static {
        async {
            let _o = o;
            let _s = should_stop;
        }
    }
    /// When the plugin gets added to the Orchestrator, this function gets called.
    /// It gives complete control on the unrunned executor.
    /// It should be used to register/activate executor, add other Plugins, add Exercise (discouraged)...
    fn on_add<'a>(
        &'a mut self,
        o: &'a mut Orchestrator<S>,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync + 'static>>> + Send + 'a {
        async {
            let _o = o;
            Ok(())
        }
    }
}

/// inner plugin storage, is not exposed outside this crate
///
/// it's needed because it simplify some extension for the user
#[allow(dead_code)] // TODO REMOVE DEADCODE check, needed because some fields are never accessed
pub(crate) struct PluginStorage<T: Plugin<S>, S: ExecutorGlobalState> {
    pub name: String,
    pub description: String,
    pub inner: Option<T>,
    pub has_run: bool,
    ph: PhantomData<S>,
}

/// Wraps the future In a Pinned Box. necessary to render it Typesafe
#[async_trait]
pub(crate) trait InnerPlugin<S: ExecutorGlobalState>: Send + Sync {
    async fn run(
        &mut self,
        o: OrchestratorReference<S>,
        should_stop: Arc<Notify>,
    ) -> Result<(), Box<dyn Error>>;
}

/// how should run be wrapped
#[async_trait]
impl<T: Plugin<S> + 'static, S: ExecutorGlobalState> InnerPlugin<S> for PluginStorage<T, S> {
    async fn run(
        &mut self,
        o: OrchestratorReference<S>,
        should_stop: Arc<Notify>,
    ) -> Result<(), Box<dyn Error>> {
        let mut data = None;
        mem::swap(&mut self.inner, &mut data);
        let data = data.ok_or("cannot find function")?;
        data.run(o, should_stop).await;
        Ok(())
    }
}

impl<T: Plugin<S>, S: ExecutorGlobalState> PluginStorage<T, S> {
    /// create a new plugin storage
    pub fn new(inner: T) -> Self {
        let name = inner.name().to_string();
        let description = inner.desctiption().to_string();
        Self {
            name,
            description,
            inner: Some(inner),
            has_run: false,
            ph: PhantomData,
        }
    }
}
