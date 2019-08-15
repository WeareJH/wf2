#![recursion_limit = "128"]
extern crate inflector;
extern crate proc_macro;
use inflector::Inflector;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::ExprLit;

#[derive(Debug)]
enum Item {
    Valid { ident: String, value: String },
    Invalid { ident: String, span: Span },
}

#[derive(Debug, Clone)]
struct EnvItem {
    key: String,
    value: String,
}

///
/// ```
/// #[macro_use]
/// extern crate serde_derive;
/// extern crate serde;
/// use env_proc::*;
/// use std::collections::HashMap;
/// env_vars! {
///    NAME = "shane"
/// }
/// fn main() {
///     let defaults = HmEnv::default();
///     assert_eq!(
///        defaults.0.get(&EnvVarKeys::Name),
///        Some(&String::from("shane"))
///     );
/// }
/// ```
///
#[proc_macro]
pub fn env_vars(item: TokenStream) -> TokenStream {
    let as_tokens: Vec<_> = item.into_iter().collect();
    let items = as_tokens.chunks(3).map(|c| {
        let ident = &c[0]; // eg: HOST_UID
        let _punc = &c[1]; // eg: =
        let value = &c[2]; // eg: "501"

        let tstream: TokenStream = value.clone().into();
        match syn::parse::<ExprLit>(tstream) {
            Ok(v) => match v {
                ExprLit {
                    lit: syn::Lit::Str(..),
                    ..
                } => {
                    let stripped = value.to_string();
                    let stripped = if stripped.len() > 2 {
                        &stripped[1..stripped.len() - 1]
                    } else {
                        ""
                    };
                    Item::Valid {
                        ident: ident.to_string(),
                        value: stripped.to_string(),
                    }
                }
                ExprLit {
                    lit: syn::Lit::Bool(..),
                    ..
                } => Item::Valid {
                    ident: ident.to_string(),
                    value: value.to_string(),
                },
                _ => Item::Invalid {
                    ident: ident.to_string(),
                    span: Span::from(value.span()),
                },
            },
            Err(..) => Item::Invalid {
                ident: ident.to_string(),
                span: Span::from(value.span()),
            },
        }
    });

    let errors: Vec<Span> = items
        .clone()
        .filter_map(|item| match item {
            Item::Invalid { span, .. } => Some(span),
            _ => None,
        })
        .collect();

    let valid = items
        .filter_map(|item| match item {
            Item::Valid { ident, value } => Some(EnvItem { key: ident, value }),
            _ => None,
        })
        .collect::<Vec<EnvItem>>();

    if errors.len() > 0 {
        return syn::Error::new(errors[0], r#"Expected a bool or string, eg: "12" or true"#)
            .to_compile_error()
            .into();
    }

    let enum_names = valid.iter().map(|env_item| {
        let enum_member_name = env_item.key.to_pascal_case();
        syn::Ident::new(&enum_member_name, Span::call_site())
    });

    let pushes = valid.iter().map(|env_item| {
        let hm = syn::Ident::new("hm", Span::call_site());
        let env_name = syn::Ident::new("EnvVarKeys", Span::call_site());
        let name = syn::Ident::new(&env_item.key.to_pascal_case(), Span::call_site());
        let value = env_item.value.to_string();
        quote! { #hm.insert(#env_name::#name, String::from(#value)); }
    });

    let to_string_impl = valid.iter().map(|env_item| {
        let hm = syn::Ident::new("hm", Span::call_site());
        let env_name = syn::Ident::new("EnvVarKeys", Span::call_site());
        let name = syn::Ident::new(&env_item.key.to_pascal_case(), Span::call_site());
        let value = env_item.value.to_string();
        quote! { #env_name::#name => String::from(#value) }
    });

    let output_tokens = quote! {

        #[derive(Eq, Debug, Clone, Hash, PartialEq, Deserialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        enum EnvVarKeys {
            #(#enum_names),*
        }

        impl EnvVarKeys {
            pub fn to_string(&self) -> String {
                match self {
                    #(#to_string_impl),*
                }
            }
        }

        #[derive(Eq, Debug, Clone, PartialEq, Deserialize)]
        struct HmEnv(HashMap<EnvVarKeys, String>);

        impl Default for HmEnv {
            fn default() -> HmEnv {
                let mut hm: HashMap<EnvVarKeys, String> = HashMap::new();
                #(#pushes);*
                HmEnv(hm)
            }
        }

        impl HmEnv {
            fn merge(mut self, other: HmEnv) -> HmEnv {
                HmEnv(self.0
                    .into_iter()
                    .chain(other.0.into_iter())
                    .collect::<HashMap<EnvVarKeys, String>>())
            }
        }
    };

    TokenStream::from(output_tokens)
}
