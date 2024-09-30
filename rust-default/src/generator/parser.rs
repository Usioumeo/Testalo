/*!
syntax tree analysis:
rules:
 * #[runtest<(Optional_points)>]
 * #[default_impl] used to decorate default impl that should be used in tests
 * #[refers_to(paths)] which default impl should I use (at least one path should be provided)
    In a correct exercise only the overridden impl should be deleted

    Because of that there are two passes:
    take file input and Parse it obtaining a Parser struct (visit)
    Transform it in an RustExercise, composed of multiple RustTests which describes how to compute an exercise (which impl to remove and which to add).
 */

use std::collections::HashMap;

use orchestrator::prelude::TestDefinition;
use quote::ToTokens;
use syn::{
    parse_file,
    visit::{visit_file, Visit},
    Attribute, Expr, ImplItem, Item, ItemFn, Lit, LitFloat, LitInt, Meta, MetaNameValue,
    PathSegment, TypePath,
};

use super::{RustError, RustExercise, RustRunTest};

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Path {path} not found")]
    PathNotFound { path: String },
}

struct PotentialRunTest {
    func: ItemFn,
    name: String,
    description: String,
    default_impls: Vec<TypePath>,
    points: f64,
}

/// temporary struct for keep track of the intermediate results
pub(super) struct Parser {
    ///contains potential run_tests (Fn, default_impl, points)
    run_tests: Vec<PotentialRunTest>,
    description: String,
    default_impls: HashMap<TypePath, Item>,
    generator: String,
}

/// function used to extract #[runtest(..)] attribute, if found it returns how much point it is worth
pub fn is_run_test(attribute: &Attribute) -> Option<f64> {
    if attribute.path().is_ident("runtest") {
        if let Some(f) = attribute
            .parse_args()
            .ok()
            .and_then(|f: LitFloat| f.base10_parse().ok())
        {
            // is a float?
            return Some(f);
        }
        if let Some(f) = attribute
            .parse_args()
            .ok()
            .and_then(|f: LitInt| f.base10_parse().ok())
        {
            // is it an int?
            return Some(f);
        }
        //let's set 1 as default
        Some(1.0)
    } else {
        None
    }
}

/// function used to extract #[refers_to(..)] attribute
pub fn is_refers_to(attribute: &Attribute) -> Option<TypePath> {
    if !attribute.path().is_ident("refers_to") {
        return None;
    }
    attribute.parse_args::<TypePath>().ok()
}

/// function used to extract doc_comments as description
fn extract_docs(attributes: &[Attribute]) -> Option<String> {
    attributes
        .iter()
        .filter_map(|x| {
            if let Meta::NameValue(MetaNameValue {
                path,
                eq_token: _,
                value:
                    Expr::Lit(syn::ExprLit {
                        attrs: _,
                        lit: Lit::Str(str_value),
                    }),
            }) = &x.meta
            {
                if path.is_ident("doc") {
                    Some(str_value.value())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .fold(None, |x: Option<String>, y| match x {
            Some(x) => Some(x + "\n" + &y),
            None => Some(y),
        })
}
impl<'ast> Visit<'ast> for Parser {
    fn visit_file(&mut self, node: &'ast syn::File) {
        //extract doc description
        self.description = extract_docs(&node.attrs).unwrap_or_default();
        //visit inner
        visit_file(self, node);
    }
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let impl_block = node.clone();
        let type_path = match node.self_ty.as_ref() {
            syn::Type::Path(path) => path.clone(),
            _ => todo!(),
        };
        for cur in &node.items {
            //for now it supports only default methods/associated functions
            match cur {
                ImplItem::Fn(f) => {
                    let ident = f.sig.ident.clone();
                    let mut type_path = type_path.clone();

                    type_path.path.segments.push(PathSegment {
                        ident,
                        arguments: syn::PathArguments::None,
                    });

                    let mut impl_block = impl_block.clone();
                    impl_block.items = vec![ImplItem::Fn(f.clone())];
                    //let path = impl_block.self_ty.to_token_stream().to_string();
                    //println!("{}", type_path.to_token_stream());
                    self.default_impls.insert(type_path, Item::Impl(impl_block));
                }
                x => {
                    println!("Warning: {} ignored", x.to_token_stream())
                }
            }
        }
    }
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let points = if let Some(x) = node.attrs.iter().filter_map(is_run_test).next() {
            x
        } else {
            return;
        };
        //extract docs
        let description = extract_docs(node.attrs.as_slice()).unwrap_or_default();
        let name = node.sig.ident.to_string();
        let default_impls = node.attrs.iter().filter_map(is_refers_to).collect();
        let mut func = node.clone();
        func.attrs
            .retain(|x| is_run_test(x).is_none() && is_refers_to(x).is_none());

        let pot = PotentialRunTest {
            func,
            name,
            description,
            default_impls,
            points,
        };
        self.run_tests.push(pot);
    }
}
impl Parser {
    pub fn new(s: &str) -> Result<Self, RustError> {
        let mut ret = Self {
            run_tests: Vec::default(),
            description: String::new(),
            default_impls: HashMap::new(),
            generator: s.to_string(),
        };
        let f = parse_file(s)?;
        ret.visit_file(&f);
        Ok(ret)
    }
    /// convert the intermediate Parser rappresentation to a real exercise
    pub fn finish(self) -> Result<RustExercise, ParserError> {
        let mut ret = RustExercise {
            description: self.description,
            run_tests: Vec::new(),
            generator: self.generator,
        };
        for pot in self.run_tests {
            let to_replace = pot
                .default_impls
                .into_iter()
                .map(|path| {
                    let cur = self
                        .default_impls
                        .get(&path)
                        .ok_or(ParserError::PathNotFound {
                            path: format!("not found: {}\n", path.to_token_stream()),
                        })?;
                    let path: Vec<String> = path
                        .path
                        .segments
                        .into_iter()
                        .map(|x| x.ident.to_string())
                        .collect();
                    Ok((path, cur.to_token_stream().to_string()))
                })
                .collect::<Result<_, ParserError>>()?;
            let cur_test_definition: RustRunTest = RustRunTest {
                func: pot.func.to_token_stream().to_string(),
                to_replace,
                desc: TestDefinition {
                    name: pot.name,
                    description: pot.description,
                    points: pot.points,
                    is_visible: true,
                },
            };
            ret.run_tests.push(cur_test_definition);
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod test {

    use quote::{quote, ToTokens};
    use syn::{parse_str, Item, TypePath};

    use crate::generator::parser::Parser;

    #[test]
    fn test_impl() {
        let file = quote!(
            struct Foo;
            impl<T> Foo {
                fn watch_me() {}
                fn me_too() {}
            }

            #[runtest]
            #[refers_to(Foo::watch_me)]
            fn test() {}
        );
        let parser = Parser::new(&file.to_string()).unwrap();

        let v = vec![
            (
                parse_str::<TypePath>("Foo :: me_too").unwrap(),
                parse_str::<Item>("impl < T > Foo { fn me_too () { } }").unwrap(),
            ),
            (
                parse_str::<TypePath>("Foo :: watch_me").unwrap(),
                parse_str::<Item>("impl < T > Foo { fn watch_me () { } }").unwrap(),
            ),
        ];
        assert_eq!(parser.default_impls, v.clone().into_iter().collect());
        let mut res = parser.finish().unwrap();
        let test = res.run_tests.remove(0);
        let v = v
            .into_iter()
            .filter_map(|(ty, item)| {
                if ty != parse_str::<TypePath>("Foo :: watch_me").unwrap() {
                    return None;
                }
                Some((
                    ty.path
                        .segments
                        .into_iter()
                        .map(|x| x.ident.to_string())
                        .collect(),
                    item.to_token_stream().to_string(),
                ))
            })
            .collect();
        assert_eq!(test.to_replace, v);
        //todo!()
    }
}
