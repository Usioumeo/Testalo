use proc_macro::TokenStream;
use syn::{fold::Fold, parse, parse_quote, parse_str, File, Ident, ItemMod};
use rust_default::prelude::*;
use quote::{format_ident, ToTokens};

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

        let module: ItemMod = parse_quote! {mod #name {
            #content
        }};
        file.items.push(module.into());
        
    }

    let mut s = Sanitizer{};
    let file = s.fold_file(file);

    file.into_token_stream().into()
}