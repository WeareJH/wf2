#![allow(clippy::needless_doctest_main)]
#![recursion_limit = "128"]

extern crate inflector;
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

const BASE: &str = "https://docs.rs/wf2_core/latest";
//const BASE: &'static str = "file:///Users/shakyshane/sites/oss/wf2/target/doc";

#[proc_macro_attribute]
pub fn doc_link(attr: TokenStream, item: TokenStream) -> TokenStream {
    let i: ItemStruct = syn::parse::<ItemStruct>(item).expect("yep");

    let s = attr.to_string();
    let stripped = &s[1..s.len() - 1];
    let str_out = format!(
        "\nDocumentation: {base}/wf2_core{path}/index.html",
        base = BASE,
        path = stripped
    );
    let lit = quote! { #str_out };
    let ident = i.ident.clone();
    let tokens = quote! {

        #i

        impl #ident {
            const DOC_LINK: &'static str = #lit;
        }
    };
    TokenStream::from(tokens)
}
