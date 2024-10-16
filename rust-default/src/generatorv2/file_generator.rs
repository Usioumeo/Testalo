use std::collections::HashMap;
use std::iter::once;

use quote::quote;
use syn::fold::{fold_file, fold_item_mod, Fold};
use syn::punctuated::Punctuated;
use syn::token::{Brace, PathSep};
use syn::{parse2, parse_quote, parse_str, File, Ident, Item, ItemMod, Path, PathSegment, Token, Type};

use super::test_definition::TestDefinition;

use super::error::RustError;
use super::parser::{extract_fn, ImplementationPath, RustExercise};

#[derive(Clone, Default, Debug)]
pub struct GeneratedFiles {
    pub files: HashMap<String, (String, f32)>,
    pub(crate) dependencies: Vec<String>,
}

impl GeneratedFiles {
    pub fn generate(def: RustExercise, user: String) -> Result<Self, RustError> {
        let user: File = parse_str(&user)?;

        let tests: Vec<TestDefinition> = def
            .tests
            .into_iter()
            .map( TestDefinition::try_from)
            .collect::<Result<Vec<TestDefinition>, RustError>>()?;
        let files: HashMap<String, (String, f32)> = tests
            .into_iter()
            .map(|test| {
                let points = test.points;
                let mut s = Substitute::new(test);
                let solution = s.fold_file(user.clone());
                let file = prettyplease::unparse(&solution);
                (s.name, (file, points ))
            })
            .collect();
        //let def = TestDefinition::try_from(def)?;

        Ok(GeneratedFiles { files, dependencies: def.dependencies})
    }
}

struct Substitute {
    name: String,
    def: TestDefinition,
    mod_path: Punctuated<PathSegment, Token![::]>,
}
impl Substitute {
    fn new(def: TestDefinition) -> Self {
        Self {
            name: String::new(),
            def,
            mod_path: Punctuated::new(),
        }
    }
}

fn is_sub_module(
    root: &Punctuated<PathSegment, PathSep>,
    bigger: &(ImplementationPath, Item),
) -> Option<(ImplementationPath, Item)> {
    if root.iter().zip(bigger.0.path.iter()).all(|(a, b)| a == b) {
        let mut bigger = bigger.clone();
        bigger.0.path = bigger.0.path.into_iter().skip(root.len()).collect();
        Some(bigger)
    } else {
        None
    }
}

fn build_submodules(mut v: Vec<(ImplementationPath, Item)>) -> Result<Vec<Item>, RustError> {
    let mut ret: Vec<Item> = v
        .iter()
        .filter_map(|(path, imp)| {
            if path.path.is_empty() {
                Some(imp)
            } else {
                None
            }
        })
        .cloned()
        .collect();
    v.retain(|(p, _)| !p.path.is_empty());
    while !v.is_empty() {
        // safe to unwrap because we know there is at least one element
        let (key, _) = v.first().unwrap();
        // it is ok to unwrap because we already filtered out the empty elements
        let t = key.path.first().unwrap().clone();
        let mut new_mod = parse2::<ItemMod>(quote!(mod #t {  }))?;
        let sub_path: Punctuated<PathSegment, PathSep> = once(t).collect();
        let next_v: Vec<_> = v
            .iter()
            .filter_map(|key| is_sub_module(&sub_path, key))
            .collect();
        v.retain(|key| is_sub_module(&sub_path, key).is_none());
        let t = build_submodules(next_v)?;
        new_mod.content = Some((Brace::default(), t));
        ret.push(new_mod.into());
    }
    //let t: Vec<ImplementationPath> = v.into_iter().filter_map(|(key, _)| is_sub_itermodule(&self.mod_path, key)).collect();
    Ok(ret)
}

impl Fold for Substitute {
    fn fold_item_mod(&mut self, i: syn::ItemMod) -> syn::ItemMod {
        self.mod_path.push(i.ident.clone().into());
        let mut module = fold_item_mod(self, i);
        // build submodules
        let v: Vec<(ImplementationPath, Item)> = self
            .def
            .to_overwrite
            .iter()
            .filter_map(|(a, b)| is_sub_module(&self.mod_path, &(a.clone(), b.clone())))
            .collect();
        self.def
            .to_overwrite
            .retain(|k, v| is_sub_module(&self.mod_path, &(k.clone(), v.clone())).is_none());
        let t = build_submodules(v).unwrap();
        if t.is_empty() {
            return module;
        }
        if let Some((_, b)) = &mut module.content {
            b.extend(t);
        } else {
            module.content = Some((Brace::default(), t));
        }
        self.mod_path.pop();
        self.mod_path.pop_punct();
        module
    }

    fn fold_item_fn(&mut self, i: syn::ItemFn) -> syn::ItemFn {
        let key = ImplementationPath::from_fn(&i, &self.mod_path);
        self.def.to_overwrite.remove(&key).and_then(|item| if let Item::Fn(f) = item{
            Some(f)
        }else{
            None
        }).unwrap_or(i)
    }
    fn fold_file(&mut self, mut i: syn::File) -> syn::File {
        i.attrs.retain(|x| !x.path().is_ident("dependency"));
        let path: Path = parse_quote!(procedural::magic_macro);
        i.attrs.retain(|x| *x.path()!=path);
        // removed the attributed function
        i.items.retain(|x| {
            if let Item::Fn(x) = &x {
                extract_fn(x).is_none()
            } else {
                true
            }
        });

        //recursively explore file
        let mut file = fold_file(self, i);
        // add the thing we havent fount nowere
        let v: Vec<(ImplementationPath, Item)> = self
            .def
            .to_overwrite
            .iter()
            .filter_map(|(a, b)| is_sub_module(&self.mod_path, &(a.clone(), b.clone())))
            .collect();
        self.def
            .to_overwrite
            .retain(|k, v| is_sub_module(&self.mod_path, &(k.clone(), v.clone())).is_none());
        let tmp = build_submodules(v).unwrap();
        file.items.extend(tmp);
        // add test
        self.name = self.def.test.sig.ident.to_string();
        self.def.test.sig.ident = parse_str::<Ident>("main").unwrap();
        file.items.push(self.def.test.clone().into());
        file
    }

    fn fold_item_impl(&mut self, mut i: syn::ItemImpl) -> syn::ItemImpl {
        if i.trait_.is_some() {
            let key = ImplementationPath::from_impl(&i, &self.mod_path);

            self.def
                .to_overwrite
                .remove(&key)
                .and_then(|x| {
                    if let syn::Item::Impl(x) = x {
                        Some(x.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or(i)
        } else {
            let key = ImplementationPath::from_impl(&i, &self.mod_path);
            i.items.retain(|elem| {
                let mut key = key.clone();
                match elem {
                    syn::ImplItem::Const(impl_item_const) => {
                        // assuming that only path inherent type can be overloadeditems
                        if let Type::Path(p) = &mut key.type_ {
                            p.path.segments.push(impl_item_const.ident.clone().into());
                            !self.def.to_overwrite.contains_key(&key)
                        } else {
                            false
                        }
                    }
                    syn::ImplItem::Fn(impl_item_fn) => {
                        key.path = self.mod_path.clone();
                        // assuming that only path inherent type can be overloaded
                        if let Type::Path(p) = &mut key.type_ {
                            p.path.segments.push(impl_item_fn.sig.ident.clone().into());
                            !self.def.to_overwrite.contains_key(&key)
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            });
            i
        }
    }
}

#[cfg(test)]
mod tests{
    use std::collections::HashMap;

    use syn::{parse_quote, File};

    use crate::prelude::{GeneratedFiles2, RustExercise2};

    #[test]
    fn check_file_generator(){
        use quote::quote;
        let q = quote! {
            //! test comment
            #![dependency("rand=0.1")]
            

            #[runtest()]
            #[overwrite(impl point_me in hidden::hidden2)]
            /// comment
            fn test_1(){
                use rand;
                ///magic test
                struct lol;
            }

            mod hidden{
                mod hidden2{
                    fn point_me(){}
                }
                struct Dummy;
                impl Dummy{
                    fn print(){}
                }
            }

            #[runtest(1.0)]
            #[overwrite(impl Dummy::print in hidden)]
            fn test_2(){

            }
        };
        let q = q.to_string();
        let t = RustExercise2::parse(&q).unwrap();
        let res = GeneratedFiles2::generate(t, "".to_string()).unwrap();
        let mut h = HashMap:: new();
        let test_1: File = parse_quote!{
            mod hidden {
                mod hidden2 {
                    fn point_me() {}
                }
            }
            /// comment
            fn main() {
                use rand;
                ///magic test
                struct lol;
            }
        };
        let test_2: File = parse_quote!{
            mod hidden {
                impl Dummy {
                    fn print() {}
                }
            }
            fn main() {}
        };
        h.insert("test_1".to_string(), 
        (prettyplease::unparse(&test_1), 1.0));
        h.insert("test_2".to_string(), (prettyplease::unparse(&test_2), 1.0));
        assert_eq!(h, res.files);
    }
}