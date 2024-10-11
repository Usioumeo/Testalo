use core::prelude;

use proc_macro::TokenStream;
use syn::{fold::{fold_file, fold_item_mod, Fold}, parse, parse_quote, parse_str, File, Ident, ItemMod};
use rust_default::prelude::*;
use quote::{format_ident, quote, ToTokens};

struct Sanitizer{

}
impl Fold for Sanitizer{
    fn fold_item_fn(&mut self, mut i: syn::ItemFn) -> syn::ItemFn {
        let ident: Ident = parse_quote!(main);
        if i.sig.ident==ident{
            i.attrs.push(parse_quote!(#[test]));
        }
        i
    }
    /*fn fold_file(&mut self, mut i: syn::File) -> syn::File {
        i.items.retain(|x| 
            if let syn::Item::ExternCrate(e) = x{
                let ident: Ident = parse_quote!(std);
                e.ident!=ident
        }else{
            true
        });
        fold_file(self, i)
    }
    fn fold_item_mod(&mut self, mut i: syn::ItemMod) -> syn::ItemMod {
        if let Some((_, items)) = &mut i.content{
            items.retain(|x| {
                //panic!("folding {}", x.to_token_stream());
                if let syn::Item::ExternCrate(e) = x{
                    
                    let ident: Ident = parse_quote!(std);
                    e.ident!=ident
            }else{
                true
            }});
        }
        fold_item_mod(self, i)
    }*/
}

#[proc_macro_attribute]
pub fn magic_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemMod = parse(item).unwrap();
    let input = input.content.map(|(_, a)| a).unwrap_or(Vec::new());
    let file = File{
        shebang: None,
        attrs: Vec::new(),
        items: input,
    };
    //let input = item.to_string();
    //panic!("{}", input);
    //println!("{}", input);
    //let mut prelude = parse_str::<File>(&prelude).unwrap().to_token_stream();
    //panic!("{}", item);
    let input = file.to_token_stream().to_string();
    let ex = RustExercise2::parse(&input).unwrap();
    let test = GeneratedFiles2::generate(ex, input).unwrap();
    let mut file = File{
        shebang: None,
        attrs: Vec::new(),
        items: Vec::new(),
    };
    
    for (name, (content, _)) in test.files{
        let name = format_ident!("{}", name);
        let content: File = parse_str(&content).unwrap();
        //println!("{} {}",name,  content);
        let module: ItemMod = parse_quote! {mod #name {
            #content
        }};
        //println!("{}", quote.to_token_stream());
        /*let module: ItemMod = parse2().map_err(|x| {
            let l = x.span().start();
            x.to_compile_error().to_string()+&format!("{} {}", l.column, l.line)
        }).unwrap();*/
        file.items.push(module.into());
        
    }
    /*let prelude = parse_quote! {
        extern crate std;

    };*/
    let mut s = Sanitizer{};
    let file = s.fold_file(file);
    //panic!("{}", prettyplease::unparse(&file));
    //panic!("{:?}", test);
    //item
    //prelude.extend();
    //prelude.into()
    file.into_token_stream().into()
}