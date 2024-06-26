/*! This module contains the default rust exercise generator (v 0.2)

The code is subdivided in 4 modules:
 * In the current module we define some common struct, and general behaviour
 * Parser: Some parser behaviour, and how to generate the exercise
 * compile: We should take the source from the user and comile it
 * run: executing and collecting the results
 */

use std::{collections::HashMap, string::FromUtf8Error};

use orchestrator::prelude::{ExerciseDef, TestDefinition};
use quote::quote;
use syn::{
    fold::Fold, parse_file, parse_str, spanned::Spanned, Attribute, Ident, Item, ItemFn,
    PathSegment,
};

use crate::generator::parser::{is_refers_to, is_run_test};

use self::{
    parser::{Parser, ParserError},
    run::RunError,
};

pub(crate) mod compile;
pub(crate) mod parser;
pub(crate) mod run;
pub use compile::RustCompiled;
pub use compile::RustGeneratedFiles;
#[derive(thiserror::Error, Debug)]
/// Error for each RustError variant
pub enum RustError {
    #[error("RunError")]
    RunError(#[from] RunError),

    #[error("Parse error")]
    ParserError(#[from] ParserError),

    #[error("Not a valid utf-8 file {0}")]
    UTF8Error(#[from] FromUtf8Error),

    #[error("IO Error {0}")]
    IOError(#[from] std::io::Error),

    #[error("Not a file")]
    NotAFile,

    #[error("File {path} not found")]
    FileNotFound { path: String },

    #[error("Parsing Error while parsing file: {:?} {}", &.0.span().start(), .0)]
    ParsingError(#[from] syn::Error),
}

pub type ItemPathSend = Vec<String>;

/// A test to execute
#[derive(Clone, Debug)]
pub struct RustRunTest {
    pub func: String,
    pub to_replace: HashMap<ItemPathSend, String>,
    pub desc: TestDefinition,
}

/// An exercise. It can be viewed as a collection of tests
#[derive(Clone, Debug)]
pub struct RustExercise {
    generator: String,
    pub description: String,
    pub run_tests: Vec<RustRunTest>,
}
impl Default for RustExercise {
    fn default() -> Self {
        let file = quote!(
            fn nothing() {}
            #[runtest(0, nothing)]
            /// checking for nothing
            fn test_nothing() {
                nothing();
            }
        )
        .to_string();
        Self::parse(&file).unwrap()
    }
}

impl ExerciseDef for RustExercise {
    fn description(&self) -> &str {
        &self.description
    }
    fn list(&self) -> Vec<TestDefinition> {
        self.run_tests
            .iter()
            .map(|x| x.desc.clone())
            .collect::<Vec<_>>()
    }

    fn get_generator_src(&self) -> &str {
        &self.generator
    }
}
impl Fold for RustRunTest {
    fn fold_item_impl(&mut self, mut node: syn::ItemImpl) -> syn::ItemImpl {
        let path = match node.self_ty.as_ref() {
            syn::Type::Path(x) => x.clone(),
            _ => todo!(),
        };
        node.items.retain(|x| {
            match x {
                syn::ImplItem::Fn(f) => {
                    let mut p = path.path.segments.clone();
                    p.push(PathSegment {
                        ident: f.sig.ident.clone(),
                        arguments: syn::PathArguments::None,
                    });
                    let v: Vec<String> = p.into_iter().map(|x| x.ident.to_string()).collect();
                    !self.to_replace.contains_key(&v)
                }
                //keep all that we don't handle
                _ => true,
            }
        });
        //node.items.retain(f);
        node
    }
    fn fold_file(&mut self, mut node: syn::File) -> syn::File {
        //maybe a retain would be more efficient, but I think with the ownership it gets clearer
        node.items = node
            .items
            .into_iter()
            .filter_map(|x| {
                let filter = |x: &[Attribute]| {
                    x.iter()
                        .all(|x| is_run_test(x).is_none() && is_refers_to(x).is_none())
                };
                match x {
                    syn::Item::Fn(f) => {
                        if filter(&f.attrs) {
                            Some(syn::Item::Fn(f))
                        } else {
                            None
                        }
                    }
                    syn::Item::Impl(imp) => {
                        let imp = self.fold_item_impl(imp);
                        if !imp.items.is_empty() {
                            Some(syn::Item::Impl(imp))
                        } else {
                            None
                        }
                    }
                    // if not interest in, keep it
                    x => Some(x),
                }
            })
            .collect();
        let to_add = self
            .to_replace
            .values()
            .map(|x| parse_str::<Item>(x).expect("shouldn't be possible").clone());
        node.items.extend(to_add);
        node
    }
}

impl RustExercise {
    /// Parse the following file in a valid exercise definition
    pub fn parse(file: &str) -> Result<Self, RustError> {
        let p = Parser::new(file)?;
        Ok(p.finish()?)
    }

    /// Load directly from file.
    /// Path must be a valid path pointing to a local file
    /*pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self, RustError> {
        //extract file name (it will become the exercise name)
        let name = path
            .as_ref()
            .file_name()
            .and_then(|x| x.to_str())
            .ok_or(RustError::NotAFile)?
            .to_string();

        // read and parse
        let file = tokio::fs::read(path).await?;
        let file = String::from_utf8(file)?;
        Self::parse(&file)
    }*/

    /// it generates file, getting reading for compilation
    pub async fn generate_files(self, solution: String) -> Result<RustGeneratedFiles, RustError> {
        let source = parse_file(&solution)?;
        let mut files = HashMap::new();

        //very fast, it doesn't need async
        for mut run_test in self.run_tests {
            let source = run_test.fold_file(source.clone());
            let prelude = quote!(#![allow(dead_code)]).to_string(); //TODO this as to come from the user
            let mut test_fn: ItemFn = parse_str(&run_test.func)?;
            let ident = test_fn.sig.ident.clone();
            let span = test_fn.sig.span();
            test_fn.sig.ident = Ident::new("main", span);
            let mut source_cur = source.clone();
            source_cur.items.push(syn::Item::Fn(test_fn));
            let t = prelude + "\n" + &prettyplease::unparse(&source_cur);
            files.insert(ident.to_string(), (t, run_test.desc.points));
        }
        Ok(RustGeneratedFiles { files })
    }
}

#[cfg(test)]
mod tests {
    use orchestrator::prelude::{CompilationResult, RunResult, TestResult};
    use quote::quote;

    use crate::generator::RustError;

    use super::RustExercise;

    #[test]
    fn check_error() {
        fn is_valid<T: Send + Sync>() {}
        is_valid::<RustError>()
    }

    #[tokio::test]
    async fn test_impl_owerride() {
        let template = quote!(
            struct Dummy;
            impl Dummy{
                fn print()->&'static str{
                    "ciao"
                }
            }
            #[refers_to(Dummy::print)]
            #[runtest]
            fn test_print(){
                assert_eq!(Dummy::print(), "ciao");
            }
        );
        let source = quote!(
            struct Dummy;
        );
        let ex = RustExercise::parse(&template.to_string()).unwrap();
        let t = ex.generate_files(source.to_string()).await.unwrap();
        let t = t.compile(None).await.unwrap();
        let t = t.run().await.unwrap();
        let v = vec![(
            "test_print".to_string(),
            TestResult {
                compiled: CompilationResult::Built,
                runned: RunResult::Ok,
                points_given: 1.0,
            },
        )];
        assert_eq!(t.tests, v.into_iter().collect());
        //panic!("{:?}", t.tests);
    }
}
