//! from (hopefully) rs file to multiple files with all that is needed to compile Vec<String>
//!
//!

use std::collections::HashMap;

use orchestrator::prelude::ExerciseDef;
use quote::{quote, ToTokens};

use syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    token::PathSep,
    visit::{visit_file, visit_item_mod, Visit},
};
use syn::{
    parse_str, Attribute, Expr, ExprLit, File, Generics, Ident, Item, ItemFn, ItemImpl, Lifetime,
    Lit, LitFloat, LitInt, LitStr, Meta, Path, PathSegment, Token, Type, TypePath,
};

use super::error::RustError;
use super::test_definition::SendableTestDefinition;
use super::test_definition::UnfinishedTestDefinition;
/// impl trait for type <in path> IGNORED GENERICS, not supported yet
///
/// the only way to implement on strange type is throught a trait?
///
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, Eq, Hash, Clone)]
pub struct ImplementationPath {
    generics: Generics,
    pub(crate) type_: Type,
    trait_: Option<Path>,
    pub path: Punctuated<PathSegment, Token![::]>,
}
impl ToTokens for ImplementationPath {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let generics = &self.generics;
        let trait_ = self.trait_.as_ref().map(|x| quote! {#x for});
        let type_ = &self.type_;
        let path = &self.path;
        let path = if !path.is_empty() {
            Some(quote! {in #path})
        } else {
            None
        };

        let t = quote! {impl #generics #trait_ #type_ #path};

        tokens.extend(t);
    }
}

impl ImplementationPath {
    pub fn from_impl(value: &ItemImpl, path: &Punctuated<PathSegment, PathSep>) -> Self {
        ImplementationPath {
            generics: value.generics.clone(),
            type_: *value.self_ty.clone(),
            trait_: value.trait_.as_ref().map(|(_, path, _)| path).cloned(),
            path: path.clone(),
        }
    }
    pub fn from_fn(value: &ItemFn, path: &Punctuated<PathSegment, PathSep>) -> Self {
        let mut type_ = Punctuated::new();
        type_.push(value.sig.ident.clone().into());
        let type_ = TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: type_,
            },
        }
        .into();
        ImplementationPath {
            generics: Generics::default(),
            type_,
            trait_: None,
            path: path.clone(),
        }
    }
}

impl Parse for ImplementationPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![impl]>()?;
        let mut type_ = input.parse::<Type>()?;
        let mut trait_ = None;

        let has_generics = input.peek(Token![<])
            && (input.peek2(Token![>])
                || input.peek2(Token![#])
                || (input.peek2(Ident) || input.peek2(Lifetime))
                    && (input.peek3(Token![:])
                        || input.peek3(Token![,])
                        || input.peek3(Token![>])
                        || input.peek3(Token![=]))
                || input.peek2(Token![const]));
        let generics: Generics = if has_generics {
            input.parse()?
        } else {
            Generics::default()
        };

        if input.peek(Token![for]) {
            input.parse::<Token![for]>()?;
            if let Type::Path(TypePath { qself: None, .. }) = &type_ {
                while let Type::Group(ty) = type_ {
                    type_ = *ty.elem;
                }
                if let Type::Path(TypePath { qself: None, path }) = type_ {
                    trait_ = Some(path);
                } else {
                    unreachable!();
                }
            } else {
                return Err(syn::Error::new_spanned(&type_, "expected trait path"));
            }
            type_ = input.parse::<Type>()?;
        }
        let path = if input.peek(Token![in]) {
            input.parse::<Token![in]>()?;
            input.parse::<Path>()?.segments
        } else {
            Punctuated::new()
        };

        Ok(ImplementationPath {
            generics,
            type_,
            trait_,
            path,
        })
    }
}
impl PartialEq for ImplementationPath {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.trait_ == other.trait_ && self.path == other.path
    }
}

#[derive(Default, Debug)]
pub struct Visiter {
    dependencies: Vec<String>,
    description: Option<String>,
    mod_path: Punctuated<PathSegment, Token![::]>,
    tests: Vec<UnfinishedTestDefinition>,
    /// store the default implementations. the key is PaUseTreeth, optional Trait.
    default_impls: HashMap<ImplementationPath, Item>,
}

#[derive(Clone, Default, Debug)]
pub struct RustExercise {
    original: String,
    pub dependencies: Vec<String>,
    description: String,
    pub tests: Vec<SendableTestDefinition>,
}

impl ExerciseDef for RustExercise {
    fn description(&self) -> &str {
        &self.description
    }

    fn get_generator_src(&self) -> &str {
        &self.original
    }

    fn list(&self) -> Vec<orchestrator::prelude::TestDefinition> {
        self.tests
            .iter()
            .map(|x| orchestrator::prelude::TestDefinition {
                name: x.name.clone(),
                description: x.description.clone(),
                points: x.points as f64,
                is_visible: true,
            })
            .collect()
    }
}

impl RustExercise {
    pub fn parse(s: &str) -> Result<Self, RustError> {
        let file = parse_str::<File>(s)?;
        let mut v = Visiter::default();
        v.visit_file(&file);
        let tests = v
            .tests
            .into_iter()
            .map(|x| x.finish(&v.default_impls).map(SendableTestDefinition::from))
            .collect::<Result<Vec<SendableTestDefinition>, RustError>>()?;
        Ok(RustExercise {
            original: s.to_string(),
            dependencies: v.dependencies,
            description: v.description.unwrap_or(String::new()),
            tests,
        })
    }
}

fn extract_documentation<'a>(value: impl Iterator<Item = &'a Attribute>) -> Option<String> {
    value
        .filter_map(|value| {
            //filter only named attributes
            let name_value = value.meta.require_name_value().ok()?;
            // compute example_comment
            let Meta::NameValue(example_comment) =
                &Attribute::parse_inner.parse_str("//!hello").unwrap()[0].meta
            else {
                panic!("something wrong while parsing with syn")
            };
            // ignore other attributes
            if name_value.path == example_comment.path
                && name_value.eq_token == example_comment.eq_token
            {
                let t = if let Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Str(s),
                }) = &name_value.value
                {
                    Some(s)
                } else {
                    None
                }?;

                Some(t.value())
            } else {
                None
            }
        })
        .reduce(|x, y| x + "\n" + &y)
}

fn extract_dependencies<'a>(value: impl Iterator<Item = &'a Attribute>) -> Vec<String> {
    value
        .filter_map(|x| {
            if !x.path().is_ident("dependency") {
                return None;
            }
            let v = x.parse_args::<LitStr>().ok()?;
            Some(v.value())
        })
        .collect()
}

pub fn extract_fn(func: &ItemFn) -> Option<UnfinishedTestDefinition> {
    let description = extract_documentation(func.attrs.iter()).unwrap_or(String::new());
    let points = func
        .attrs
        .iter()
        .filter_map(|attribute| {
            if !attribute.path().is_ident("runtest") {
                return None;
            }
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
        })
        .next()?;
    let to_overwrite: Vec<ImplementationPath> = func
        .attrs
        .iter()
        .filter_map(|attribute| {
            if !attribute.path().is_ident("overwrite") {
                return None;
            }
            attribute.parse_args().ok()
        })
        .collect();
    let mut test = func.clone();
    test.attrs
        .retain(|x| !x.path().is_ident("overwrite") && !x.path().is_ident("runtest"));
    Some(UnfinishedTestDefinition {
        to_overwrite,
        test,
        description,
        points,
    })
}

impl<'a> Visit<'a> for Visiter {
    fn visit_file(&mut self, node: &'a syn::File) {
        //extract doc as string
        self.description = extract_documentation(node.attrs.iter());
        self.dependencies = extract_dependencies(node.attrs.iter());
        //get tests
        self.tests = node
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Fn(item_fn) => extract_fn(item_fn),
                _ => None,
            })
            .collect();
        //visit inner
        visit_file(self, node);
    }
    fn visit_item_mod(&mut self, i: &'a syn::ItemMod) {
        // extract module name, and append to path
        let segment = PathSegment {
            ident: i.ident.clone(),
            arguments: syn::PathArguments::None,
        };
        self.mod_path.push(segment);

        // visit inner
        visit_item_mod(self, i);

        // pop name from mod_path
        self.mod_path.pop();
        self.mod_path.pop_punct();
    }
    /// get impl
    fn visit_item_impl(&mut self, i: &'a syn::ItemImpl) {
        if i.trait_.is_some() {
            // can't split
            let key = ImplementationPath::from_impl(i, &self.mod_path);
            self.default_impls.insert(key, i.clone().into());
        }
        for item in &i.items {
            let mut implementation_skelethon = i.clone();
            match item {
                syn::ImplItem::Const(impl_item_const) => {
                    let mut key = ImplementationPath::from_impl(i, &self.mod_path);
                    // assuming that only path inherent type can be overloaded
                    if let Type::Path(p) = &mut key.type_ {
                        p.path.segments.push(impl_item_const.ident.clone().into());
                        implementation_skelethon.items = vec![impl_item_const.clone().into()];
                        self.default_impls
                            .insert(key, implementation_skelethon.into());
                    }
                }
                syn::ImplItem::Fn(impl_item_fn) => {
                    let mut key = ImplementationPath::from_impl(i, &self.mod_path);
                    key.path = self.mod_path.clone();
                    // assuming that only path inherent type can be overloaded
                    if let Type::Path(p) = &mut key.type_ {
                        p.path.segments.push(impl_item_fn.sig.ident.clone().into());
                        implementation_skelethon.items = vec![impl_item_fn.clone().into()];
                        self.default_impls
                            .insert(key, implementation_skelethon.into());
                    }
                }
                _ => {}
            }
        }
    }
    ///for functions
    fn visit_item_fn(&mut self, i: &'a syn::ItemFn) {
        if extract_fn(i).is_some() {
            return;
        }
        let key = ImplementationPath::from_fn(i, &self.mod_path);
        self.default_impls.insert(key, i.clone().into());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{ImplementationPath, Visiter};
    use quote::{quote, ToTokens};
    use syn::parse_str;
    use syn::punctuated::Punctuated;
    use syn::{parse2, parse_file, visit::Visit, File, Generics, Path, Type, TypePath};
    #[test]
    fn test() {
        let q = quote! {
        //!hello
        //!hello2
        //!
        #![do_something]
        #[derive(Debug)]
        struct x;
        };
        let f = q.to_token_stream().to_string();
        println!("{}", f);
        let file = parse_file(&f).unwrap();
        Visiter::default().visit_file(&file);
    }
    #[test]
    fn test2() {
        let q = quote! {
            #[runtest]
            #[overwrite(impl Derive for a in b::c)]
            fn test(){

            }
        }
        .to_string();
        //let f = q.to_token_stream().to_string();
        //println!("{}", f);
        let file = parse_file(&q).unwrap();
        let mut v = Visiter::default();
        v.visit_file(&file);
        println!("{:?}", v);
    }
    #[test]
    fn test_general() {
        let q = quote! {
            //! test comment
            #![dependency("rand=0.1")]

            #[runtest()]
            #[overwrite(bigger)]
            /// comment
            fn test_1(){

            }
            fn bigger(a: i32, b:i32)->i32{
                if(a>b){
                    a
                }else{
                    b
                }
            }
        };
        let file = parse2::<File>(q).unwrap();
        let mut v = Visiter::default();
        v.visit_file(&file);

        assert_eq!(v.dependencies, vec!["rand=0.1".to_string()]);
        assert_eq!(v.description, Some(" test comment".to_string()));
        let punctuated: Path = parse2(quote! {bigger}).unwrap();
        let impl_path = ImplementationPath {
            generics: Generics::default(),
            type_: Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: punctuated.segments,
                },
            }),
            trait_: None,
            path: Punctuated::new(),
        };
        let function = syn::parse2(quote! {fn bigger(a: i32, b:i32)->i32{
            if(a>b){
                a
            }else{
                b
            }
        }
        })
        .unwrap();
        let mut default_impls = HashMap::new();
        default_impls.insert(impl_path, function);
        assert_eq!(v.default_impls, default_impls);
        assert_eq!(v.mod_path, Punctuated::new());
        assert!(v.tests.len() == 1);
        let test = v.tests[0].clone();
        assert_eq!(test.description, " comment");
        assert_eq!(test.points, 1.0);
        // TODO finish
        /*assert_eq!(test.test, parse_str("fn test_1(){

        }").unwrap());*/
    }
    #[test]
    fn test_implementation_path() {
        let t = "impl Derive for a in b :: c";
        let p = parse_str::<ImplementationPath>(t).unwrap();
        assert_eq!(p.to_token_stream().to_string(), t);

        let t = "impl a in b :: c";
        let p = parse_str::<ImplementationPath>(t).unwrap();
        assert_eq!(p.to_token_stream().to_string(), t);
        let t = "impl a";
        let p = parse_str::<ImplementationPath>(t).unwrap();
        assert_eq!(p.to_token_stream().to_string(), t);
    }
}
