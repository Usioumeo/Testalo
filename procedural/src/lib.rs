use proc_macro::TokenStream;
use syn::{fold::Fold, parse, parse_quote, parse_str, token::Brace, File, Ident, ItemMod};
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
    let mut ret = input.clone();
    let input_content = input.content.map(|(_, a)| a).unwrap_or(Vec::new());
    let file = File{
        shebang: None,
        attrs: input.attrs,
        items: input_content,
    };

    let input = file.to_token_stream().to_string();
    let ex = RustExercise2::parse(&input).unwrap();
    let test = GeneratedFiles2::generate(ex, input).unwrap();
    /*let mut file = File{
        shebang: None,
        attrs: Vec::new(),
        items: Vec::new(),
    };*/
    if ret.content.is_none(){
        ret.content = Some((Brace::default(), Vec::new()));
    }else{
        ret.content.as_mut().unwrap().1.clear();
    }
    let ret_item = ret.content.as_mut().map(|(_, a)| a).unwrap();
    for (name, (content, _)) in test.files{
        let name = format_ident!("{}", name);
        let content: File = parse_str(&content).unwrap();

        let module: ItemMod = parse_quote! {mod #name {
            #content
        }};
        ret_item.push(module.into());
        
    }
    ret.attrs.clear();

    let mut s = Sanitizer{};
    let ret = s.fold_item_mod(ret);

    ret.into_token_stream().into()
}