//! from (hopefully) rs file to multiple files with all that is needed to compile Vec<String>
//!
//!
use std::collections::HashMap;
use orchestrator::prelude::ExerciseDef;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::visit::visit_item_mod;
use syn::{
    parse_str, ExprLit, File, Generics, Ident, Item, ItemFn, ItemImpl, Lifetime, LitFloat, LitInt, LitStr, Path, PathSegment, Token, Type, TypePath
};

use syn::{
    parse::Parser,
    visit::{visit_file, Visit},
    Attribute, Expr, Lit, Meta,
};

use super::error::RustError;
/// impl trait for type <in path> IGNORED GENERICS, not supported yet
///
/// the only way to implement on strange type is throught a trait?
///
#[derive(Debug, Eq, Hash, Clone)]
pub struct ImplementationPath {
    generics: Generics,
    type_: Type,
    trait_: Option<Path>,
    pub path: Option<Punctuated<PathSegment, Token![::]>>,
}
#[test]
fn test_sync(){
    fn sync<T: Sync>(){}
    //sync::<Type>();
}

impl From<&ItemImpl> for ImplementationPath {
    fn from(value: &ItemImpl) -> Self {
        ImplementationPath {
            generics: value.generics.clone(),
            type_: *value.self_ty.clone(),
            trait_: value.trait_.as_ref().map(|(_, path, _)| path).cloned(),
            path: None,
        }
    }
}

impl Parse for ImplementationPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![impl]>()?;
        let mut type_ = input.parse::<Type>()?;
        let mut trait_ = None;
        let mut path = None;

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
        if input.peek(Token![in]) {
            input.parse::<Token![in]>()?;
            path = Some(input.parse::<Path>()?.segments);
        }

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
#[derive(Debug, Clone)]
pub struct UnfinishedTestDefinition {
    to_overwrite: Vec<ImplementationPath>,
    test: ItemFn,
    description: String,
    points: f32,
}
impl UnfinishedTestDefinition{
    pub fn finish(self, default_impl: &HashMap<ImplementationPath, Item> )->Result<TestDefinition, RustError>{
        let to_overwrite = self.to_overwrite.into_iter().map(|o| {
            default_impl.get(&o).map(|x| (o.clone(), x.clone())).ok_or(RustError::MatchNotFound(format!("{:?}", o)))
        }).collect::<Result<_, RustError>>()?;
        Ok(TestDefinition{
            to_overwrite,
            test: self.test,
            description: self.description,
            points: self.points,
        })
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
#[derive(Clone)]
pub struct TestDefinition{
    to_overwrite: HashMap<ImplementationPath, Item>,
    test: ItemFn,
    description: String,
    points: f32,
}
#[derive(Clone, Default)]
pub struct RustExercise{
    dependencies: Vec<String>,
    description: String,
    tests: Vec<TestDefinition>,
}
impl RustExercise{
    pub fn parse(s: &str)->Result<Self, RustError>{
        let file = parse_str::<File>(s)?;
        let mut v = Visiter::default();
        v.visit_file(&file);
        let tests = v.tests.into_iter().map(|x| x.finish(&v.default_impls)).collect::<Result<Vec<TestDefinition>, RustError>>()?;
        Ok(RustExercise{
            dependencies: v.dependencies,
            description: v.description.unwrap_or(String::new()),
            tests,
        })
    }
}
impl ExerciseDef for RustExercise{
    fn description(&self) -> &str {
        todo!()
    }

    fn get_generator_src(&self) -> &str {
        todo!()
    }

    fn list(&self) -> Vec<orchestrator::prelude::TestDefinition> {
        todo!()
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

fn extract_dependencies<'a>(value: impl Iterator<Item = &'a Attribute>) -> Vec<String>{
    value.filter_map(|x| {
        if !x.path().is_ident("dependency"){
            return None;
        }
        let v = x.parse_args::<LitStr>().ok()?;
        Some(v.value())
    }).collect()
}

fn extract_fn<'a>(func: &ItemFn) -> Option<UnfinishedTestDefinition> {
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
            if let Some(path) = attribute.parse_args().ok() {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    Some(UnfinishedTestDefinition {
        to_overwrite,
        test: func.clone(),
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
                Item::Fn(item_fn) => extract_fn(&item_fn),
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
        visit_item_mod(self, &i);

        // pop name from mod_path
        self.mod_path.pop();
    }
    /// get impl
    fn visit_item_impl(&mut self, i: &'a syn::ItemImpl) {
        if i.trait_.is_some() {
            // can't split
            let mut key: ImplementationPath = i.into();
            key.path = Some(self.mod_path.clone());
            self.default_impls.insert(key, i.clone().into());
        }
        for item in &i.items {
            let mut implementation_skelethon = i.clone();
            match item {
                syn::ImplItem::Const(impl_item_const) => {
                    let mut key: ImplementationPath = i.into();
                    key.path = Some(self.mod_path.clone());
                    // assuming that only path inherent type can be overloaded
                    if let Type::Path(p) = &mut key.type_ {
                        p.path.segments.push(impl_item_const.ident.clone().into());
                        implementation_skelethon.items = vec![impl_item_const.clone().into()];
                        self.default_impls
                            .insert(key, implementation_skelethon.into());
                    }
                }
                syn::ImplItem::Fn(impl_item_fn) => {
                    let mut key: ImplementationPath = i.into();
                    key.path = Some(self.mod_path.clone());
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
        if extract_fn(&i).is_some(){
            return;
        }
        let mut  type_ = Punctuated::new();
        type_.push(i.sig.ident.clone().into());
        let type_ = TypePath{
            qself: None,
            path: Path{
                leading_colon: None,
                segments: type_,
            },
        }.into();
        let path = if self.mod_path.len()>0{
            Some(self.mod_path.clone())
        }else{
            None
        };
        let key = ImplementationPath{
            generics: Generics::default(),
            type_,
            trait_: None,
            path,
        };
        self.default_impls.insert(key, i.clone().into());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use quote::{quote, ToTokens};
    use syn::punctuated::Punctuated;
    use syn::{parse2, parse_file, visit::Visit, File, Generics, Path, Type, TypePath};
    use super::{ImplementationPath, Visiter};
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
        let mut v =Visiter::default();
        v.visit_file(&file);
        
        assert_eq!(v.dependencies, vec!["rand=0.1".to_string()]);
        assert_eq!(v.description, Some(" test comment".to_string()));
        let punctuated: Path = parse2(quote! {bigger}).unwrap();
        let impl_path = ImplementationPath{
            generics: Generics::default(),
            type_: Type::Path(TypePath{qself: None, path: Path{leading_colon: None, segments: punctuated.segments}}),
            trait_: None ,
            path: None,
        };
        let function = syn::parse2(quote! {fn bigger(a: i32, b:i32)->i32{
            if(a>b){
                a
            }else{
                b
            }
        }
        }).unwrap();
        let mut default_impls = HashMap::new();
        default_impls.insert(impl_path, function);
        assert_eq!(v.default_impls, default_impls);
        assert_eq!(v.mod_path, Punctuated::new());
        assert!(v.tests.len()==1);
        let test = v.tests[0].clone();
        assert_eq!(test.description, " comment");
        assert_eq!(test.points, 1.0);
        // TODO finish
        /*assert_eq!(test.test, parse_str("fn test_1(){

            }").unwrap());*/
    }
}
