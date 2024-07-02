// macro that creates "serialize/deserialize action enum"
// create_action(X, Y)
// #[derive(Serialize, Deserialize)]
// enum ActionStates{
//    source
//  x, (implies struct X)
//  y,
//}

use std::{
    any::{Any, TypeId},
    error::Error as StdError,
    pin::Pin,
};

use std::future::Future;

use crate::prelude::{
    serde::{Deserialize, Serialize},
    *,
};

/// This trait is implemented for a State managable by the orchestrator.
/// I Strongly advise in using the provided macro to generate it
pub trait ExecutorGlobalState: Clone + TryInto<ExerciseResult> + Send + Sync + 'static {
    fn serialize_variant(&self) -> String;
    fn deserialize_variant(s: &str) -> Result<TypeId, Box<dyn StdError + Send + Sync + 'static>>;
}

/// Generates an Enum that implements ExecutorGlobalState.
///
/// Simply pass the types that you want to manage in your state:
#[macro_export]
macro_rules! GenerateState {
    ($($cur:ident),+) => {
        use std::any::TypeId;
        use orchestrator::prelude::*;
        use serde_json;

        #[derive(Clone)]
        enum State{
            $($cur($cur)),+
        }
        /*fn check<S: ExecutorState>(){

        }

        #[test]
        fn test_if_all_are_implementing_executor_state(){

            $(check::<$cur>();)+
        }*/
        $(
        #[allow(irrefutable_let_patterns)]
        impl TryFrom<State> for $cur{
            type Error=();
            fn try_from(value: State) -> Result<Self, Self::Error> {
                if let State::$cur(s) = value{
                    Ok(s)
                }else{
                    Err(())
                }
            }
        }
        impl From<$cur> for State{
            fn from(value: $cur) -> Self {
                Self::$cur(value)
            }
        }
        )+
        #[derive(Serialize, Deserialize)]
        #[serde(crate = "orchestrator::prelude::serde")]
        enum SerDeState{
            $($cur),+
        }

        impl ExecutorGlobalState for State{
            fn serialize_variant(&self)->String{
                match self{
                    //it's tested, it should NEVER panic
                    $(State::$cur(_) =>  serde_json::to_string(&SerDeState::$cur).unwrap()),+
                }
            }
            fn deserialize_variant(input: &str)->Result<TypeId, Box<dyn std::error::Error + Send + Sync + 'static >>{
                let s: SerDeState =serde_json::from_str(input)?;
                match s {
                    $(SerDeState::$cur => Ok(TypeId::of::<$cur>())),+
                }
            }
        }
    };
}

/// it's an auto trait. It it has all the requirements it will be used
pub trait ExecutorState: Clone + Send + Sync + AsyncDefault + 'static + Any {}
/// automatic implementation for autotrait
impl<S: Clone + Send + Sync + 'static + AsyncDefault + Any> ExecutorState for S {}

// TODO remove this trait
pub trait AddExecutor<Input: ExecutorState, Out: ExecutorState> {
    /// method used to register an executor:
    ///
    /// it also checks if the executor does work.
    /// In order to do that it create the default Input, and then call's the function. If it works adds it to the executor register.
    fn add_executor<F, E, Data>(
        &mut self,
        f: fn(Input, Data) -> F,
        data: Data,
    ) -> impl Future<Output = Result<(), Box<dyn StdError + Send + Sync + 'static>>>
    where
        F: Future<Output = Result<Out, E>> + 'static + Send + Sync,
        E: Into<Box<dyn StdError + Send + Sync>>,
        Data: Serialize + for<'a> Deserialize<'a> + 'static;

    fn enable_executor_typed<Data: Serialize>(
        &mut self,
        i: &Input,
        o: &Out,
        data: Data,
    ) -> impl Future<Output = Result<(), Error>>;
}

pub type ExecutorFuture<S> = Pin<
    Box<dyn Send + Sync + Future<Output = Result<S, Box<dyn StdError + Send + Sync + 'static>>>>,
>;
pub type Executor<S> = Box<dyn Send + Sync + Fn(S, String) -> ExecutorFuture<S>>;

impl<S, Input, Output> AddExecutor<Input, Output> for Orchestrator<S>
where
    S: ExecutorGlobalState + Sized,
    Input: TryFrom<S> + Into<S> + ExecutorState + Any,
    Output: Into<S> + ExecutorState,
{
    /// method used to register an executor:
    ///
    /// it also checks if the executor does work.
    /// In order to do that it create the default Input, and then call's the function. If it works adds it to the executor register.
    async fn add_executor<F, E, Data>(
        &mut self,
        func: fn(Input, Data) -> F,
        data: Data,
    ) -> Result<(), Box<dyn StdError + Send + Sync + 'static>>
    where
        F: Future<Output = Result<Output, E>> + 'static + Send + Sync,
        E: Into<Box<dyn StdError + Send + Sync + 'static>> ,
        Data: Serialize + for<'a> Deserialize<'a> + 'static,
    {
        // wrap in a generic function
        let f = move |s: S, data: String| {
            let t: ExecutorFuture<S> = Box::pin(async move {
                let data = serde_json::from_str(&data)?;
                let res = func(
                    <S as TryInto<Input>>::try_into(s).map_err(|_| Error::NotFound)?,
                    data,
                )
                .await
                .map_err(|x| x.into())?;
                let t = Into::<S>::into(res);
                Ok::<S, Box<dyn StdError + Send + Sync + 'static>>(t)
            });
            t
        };
        let serialized_data = serde_json::to_string(&data)?;
        // check if it is working
        if let Err(err) = f(Into::into(Input::async_default().await), serialized_data).await {
            Err(format!("Execution Failed with error: {}", err).as_str())?
        } else {
            self.executors
                .insert((TypeId::of::<Input>(), TypeId::of::<Output>()), Box::new(f));
        }
        Ok(())
    }

    async fn enable_executor_typed<Data: Serialize>(
        &mut self,
        i: &Input,
        o: &Output,
        data: Data,
    ) -> Result<(), Error> {
        let i: S = i.clone().into();
        let o: S = o.clone().into();
        if !self
            .executors
            .contains_key(&(TypeId::of::<Input>(), TypeId::of::<Output>()))
        {
            return Err(Error::UnregisteredExecutor);
        }
        let data_string = serde_json::to_string(&data)?;
        self.memory()
            .enable_executor(&i, &o, data_string)
            .await
            .map_err(|_| Error::CycleDetected)?;
        Ok(())
    }
}

#[allow(opaque_hidden_inferred_bound)]
pub trait AsyncDefault {
    fn async_default() -> impl Future<Output = Self>;
}
impl<T: Default> AsyncDefault for T {
    async fn async_default() -> Self {
        Default::default()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic Error")]
    Generic(#[from] Box<dyn StdError + Send + Sync + 'static>),
    #[error("Implementation not found")]
    NotFound,
    #[error("cycle detected")]
    CycleDetected,
    #[error("Not a registered executor")]
    UnregisteredExecutor,
    #[error("Json serialize Error")]
    Json(#[from] serde_json::Error),
}
#[cfg(test)]
mod test {
    //use serde::{Deserialize, Serialize};
    use crate as orchestrator;

    GenerateState!(ExerciseResult);

    #[tokio::test]
    async fn try_test() {
        let mut v: Vec<State> = Vec::new();
        for _ in 0..10 {
            v.push(<ExerciseResult as Default>::default().into());
        }
    }
}
