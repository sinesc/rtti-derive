#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
use quote::{Tokens, ToTokens};
use proc_macro2::{TokenTree};
use proc_macro::TokenStream;

#[proc_macro_derive(RTTI, attributes(HelloWorldName))]
pub fn macro_rtti(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let gen = impl_rtti(&ast);
    gen.into()
}

fn noescape(string: &str) -> TokenTree {
    use proc_macro2::{TokenNode, TokenTree, Span, Term};
    TokenTree {
        span: Span::def_site(),
        kind: TokenNode::Term(Term::intern(string)),
    }
}

fn translate_visibility(vis: &syn::Visibility) -> TokenTree {
    match vis {
        &syn::Visibility::Public(_) => noescape("Visibility::Public"),
        &syn::Visibility::Crate(_) => noescape("Visibility::Crate"),
        &syn::Visibility::Restricted(_) => noescape("Visibility::Restricted"),
        &syn::Visibility::Inherited => noescape("Visibility::Inherited"),
        _ => noescape("Visibility::Unknown"),
    }
}

fn impl_rtti(ast: &syn::DeriveInput) -> quote::Tokens {
    let ident = ast.ident;
    let name = ast.ident.to_string();
    let visibility = translate_visibility(&ast.vis);

    if let syn::Data::Struct(ref data) = ast.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            let names: Vec<_> = fields.named.iter().map(|field| field.ident.unwrap().to_string()).collect();
            let visibilities: Vec<_> = fields.named.iter().map(|field| translate_visibility(&field.vis)).collect();

            let types: Vec<_> = fields.named.iter().map(|field| {
                //TODO: field.attrs, check for ignored types. use Option<Box<Type>>
                &field.ty

            }).collect();

            let result = quote! {
                impl RTTI for #ident {
                    fn rtti() -> Type {
                        Type::Struct(Struct {
                            name: #name.to_string(),
                            vis: #visibility,
                            fields: {
                                let mut fields = Vec::new();
                                #(
                                    fields.push(Field {
                                        name: #names.to_string(),
                                        vis: #visibilities,
                                        ty: Box::new(#types::rtti())
                                    });
                                )*
                                fields
                            }
                        })
                    }
                }
            }
            ;result
            //;panic!(result.to_string())
        } else {
            panic!("#[derive(RTTI)] is only defined for structs.");
        }
    } else {
        panic!("#[derive(RTTI)] is only defined for structs.");
    }
}