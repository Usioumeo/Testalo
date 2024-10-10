use std::collections::HashMap;

use quote::ToTokens;
use syn::{parse_str, Item, ItemFn};

use super::{error::RustError, parser::ImplementationPath};

#[derive(Clone)]
pub struct TestDefinition {
    pub(crate) to_overwrite: HashMap<ImplementationPath, Item>,
    pub(crate) test: ItemFn,
    pub(crate) description: String,
    pub(crate) points: f32,
}

#[derive(Clone, Debug)]
pub struct SendableTestDefinition {
    pub(crate) name: String,
    pub(crate) to_overwrite: HashMap<String, String>,
    pub(crate) test: String,
    pub(crate) description: String,
    pub(crate) points: f32,
}

impl From<TestDefinition> for SendableTestDefinition {
    fn from(value: TestDefinition) -> Self {
        let name = value.test.sig.ident.to_string();
        Self {
            name,
            to_overwrite: value
                .to_overwrite
                .into_iter()
                .map(|(a, b)| {
                    (
                        a.to_token_stream().to_string(),
                        b.to_token_stream().to_string(),
                    )
                })
                .collect(),
            test: value.test.to_token_stream().to_string(),
            description: value.description,
            points: value.points,
        }
    }
}
impl TryFrom<SendableTestDefinition> for TestDefinition {
    fn try_from(value: SendableTestDefinition) -> Result<Self, RustError> {
        Ok(Self {
            to_overwrite: value
                .to_overwrite
                .into_iter()
                .map(|(a, b)| Ok((parse_str(&a)?, parse_str(&b)?)))
                .collect::<Result<HashMap<ImplementationPath, Item>, RustError>>()?,
            test: parse_str::<ItemFn>(&value.test)?,
            description: value.description,
            points: value.points,
        })
    }

    type Error = RustError;
}

#[derive(Debug, Clone)]
pub struct UnfinishedTestDefinition {
    pub(crate) to_overwrite: Vec<ImplementationPath>,
    pub(crate) test: ItemFn,
    pub(crate) description: String,
    pub(crate) points: f32,
}
impl UnfinishedTestDefinition {
    pub fn finish(
        self,
        default_impl: &HashMap<ImplementationPath, Item>,
    ) -> Result<TestDefinition, RustError> {
        let all = default_impl.keys().map(|x| x.to_token_stream().to_string()).fold(String::new(), |a, b| a+"\n"+&b);
        let to_overwrite = self
            .to_overwrite
            .into_iter()
            .map(|o| {
                default_impl
                    .get(&o)
                    .map(|x| (o.clone(), x.clone()))
                    .ok_or(RustError::MatchNotFound(format!("Not found: {}\n but instead found:\n {}", o.to_token_stream(), all)))
            })
            .collect::<Result<_, RustError>>()?;
        Ok(TestDefinition {
            to_overwrite,
            test: self.test,
            description: self.description,
            points: self.points,
        })
    }
}
