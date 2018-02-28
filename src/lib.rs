#![recursion_limit="256"]

/*!
 * Procedural macro to derive RTTI trait.
 *
 * See crate [`rtti`](https://crates.io/crates/rtti) for [documentation](https://docs.rs/rtti/).
 */

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::Meta::{List, NameValue, Word};
use syn::NestedMeta::Meta;

#[proc_macro_derive(RTTI, attributes(rtti))]
pub fn macro_rtti(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let gen = impl_rtti(&ast);
    gen.into()
}

fn impl_rtti(ast: &syn::DeriveInput) -> quote::Tokens {

    let ident = ast.ident;
    let name = ast.ident.to_string();
    let visibility = translate_visibility(&ast.vis);
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let dummy_const = syn::Ident::new(&format!("_IMPL_RTTI_FOR_{}", ident), Span::def_site());
    let dummy_type = dummy_type();

    let body = if let syn::Data::Struct(ref data) = ast.data {
        if let syn::Fields::Named(ref fields) = data.fields {

            // handle structs with named members

            let idents: Vec<_> = fields.named.iter().map(|field| field.ident.unwrap()).collect();
            let names: Vec<_> = idents.iter().map(|ident| ident.to_string()).collect();
            let visibilities: Vec<_> = fields.named.iter().map(|field| translate_visibility(&field.vis)).collect();

            let types: Vec<_> = fields.named.iter().map(|field| {
                if parse_attr_ignore(&field.attrs) {
                    &dummy_type
                } else {
                    &field.ty
                }
            }).collect();

            let hints: Vec<_> = fields.named.iter().map(|field| {
                parse_attr_hint(&field.attrs)
            }).collect();

            quote! {
                Type::Struct(Struct {
                    name: #name,
                    vis: Visibility::#visibility,
                    size: ::std::mem::size_of::<#ident #impl_generics>(),
                    fields: {
                        let mut fields = Vec::new();
                        let dummy: #ident #impl_generics = unsafe { ::std::mem::uninitialized() };
                        #(
                            fields.push((#names, Field {
                                vis: Visibility::#visibilities,
                                offset: {
                                    let dummy_ref = &dummy;
                                    let field_ref = &dummy.#idents;
                                    (field_ref as *const _ as usize) - (dummy_ref as *const _ as usize)
                                },
                                ty: <#types>::rtti(),
                                hints: {
                                    let mut hints = Vec::new();
                                    #(
                                        hints.push(#hints);
                                    )*
                                    hints
                                }
                            }));
                        )*
                        std::mem::forget(dummy);
                        fields
                    }
                })
            }

        } else if let syn::Fields::Unnamed(ref fields) = data.fields {

            // handle structs with unnamed members

            let visibilities: Vec<_> = fields.unnamed.iter().map(|field| translate_visibility(&field.vis)).collect();
            let indices: Vec<_> = (0..visibilities.len()).map(|x| syn::Index::from(x)).collect();

            let types: Vec<_> = fields.unnamed.iter().map(|field| {
                if parse_attr_ignore(&field.attrs) {
                    &dummy_type
                } else {
                    &field.ty
                }
            }).collect();

            let hints: Vec<_> = fields.unnamed.iter().map(|field| {
                parse_attr_hint(&field.attrs)
            }).collect();

            quote! {
                Type::Tuple(Tuple {
                    name: #name,
                    vis: Visibility::#visibility,
                    size: ::std::mem::size_of::<#ident #impl_generics>(),
                    fields: {
                        let mut fields = Vec::new();
                        let dummy: #ident #impl_generics = unsafe { ::std::mem::uninitialized() };
                        #(
                            fields.push(Field {
                                vis: Visibility::#visibilities,
                                offset: {
                                    let dummy_ref = &dummy;
                                    let field_ref = &(dummy.#indices);
                                    (field_ref as *const _ as usize) - (dummy_ref as *const _ as usize)
                                },
                                ty: <#types>::rtti(),
                                hints: {
                                    let mut hints = Vec::new();
                                    #(
                                        hints.push(#hints);
                                    )*
                                    hints
                                }
                            });
                        )*
                        std::mem::forget(dummy);
                        fields
                    }
                })
            }
        } else {
            panic!("#[derive(RTTI)] NYI unit struct.");
        }
    } else if let syn::Data::Enum(ref data) = ast.data {

        let variants = &data.variants;
        let idents: Vec<_> = variants.iter().map(|variant| variant.ident).collect();
        let names: Vec<_> = idents.iter().map(|ident| ident.to_string()).collect();

        let variant_hints: Vec<_> = variants.iter().map(|variant| {
            parse_attr_hint(&variant.attrs)
        }).collect();

        let field_types: Vec<Vec<_>> = variants.iter().map(|variant| {
            variant.fields.iter().map(|field| {
                if parse_attr_ignore(&field.attrs) {
                    &dummy_type
                } else {
                    &field.ty
                }
            }).collect()
        }).collect();

        let field_hints: Vec<Vec<_>> = variants.iter().map(|variant| {
            variant.fields.iter().map(|field| parse_attr_hint(&field.attrs)).collect()
        }).collect();

        quote! {
            Type::Enum(Enum {
                name: #name,
                vis: Visibility::#visibility,
                size: ::std::mem::size_of::<#ident #impl_generics>(),
                variants: {
                    let mut variants = Vec::new();
                    //let dummy: #ident #impl_generics = unsafe { ::std::mem::uninitialized() };
                    #(
                        variants.push((#names, Variant {
                            fields: {
                                let mut fields = Vec::new();
                                #(
                                    fields.push(Field {
                                        vis: Visibility::Public,
                                        offset: 0, /*{
                                            let dummy_ref = &dummy;
                                            let field_ref = &dummy.#idents;
                                            (field_ref as *const _ as usize) - (dummy_ref as *const _ as usize)
                                        },*/
                                        ty: <#field_types>::rtti(),
                                        hints: {
                                            let mut hints = Vec::new();
                                            #(
                                                hints.push(#field_hints);
                                            )*
                                            hints
                                        }
                                    });
                                )*
                                fields
                            },
                            hints: {
                                let mut hints = Vec::new();
                                #(
                                    hints.push(#variant_hints);
                                )*
                                hints
                            }
                        }));
                    )*
                    //std::mem::forget(dummy);
                    variants
                }
            })
        }
    } else {
        panic!("#[derive(RTTI)] NYI union");
    };

    quote! {
        #[allow(non_upper_case_globals,unused_mut)]
        const #dummy_const: () = {
            extern crate rtti;
            use rtti::*;
            impl #impl_generics RTTI for #ident #ty_generics #where_clause  {
                fn rtti() -> Type {
                    #body
                }
            }
        };
    }
}

fn translate_visibility(vis: &syn::Visibility) -> syn::Ident {
    #[allow(unreachable_patterns)]
    match vis {
        &syn::Visibility::Public(_) => syn::Ident::from("Public"),
        &syn::Visibility::Crate(_) => syn::Ident::from("Crate"),
        &syn::Visibility::Restricted(_) => syn::Ident::from("Restricted"),
        &syn::Visibility::Inherited => syn::Ident::from("Inherited"),
        _ => syn::Ident::from("Unknown"),
    }
}

fn parse_attr_hint(attrs: &Vec<syn::Attribute>) -> Vec<String> {
    let mut hints = Vec::new();
    for meta_items in attrs.iter().filter_map(filter_attr_rtti) {
        for meta_item in meta_items {
            match meta_item {
                // Parse `#[rtti(hint = "foo")]`
                Meta(NameValue(ref m)) if m.ident == "hint" => {
                    if let Some(s) = parse_lit(&m.lit) {
                        hints.push(s.value().to_string());
                    }
                },
                _ => {}
            }
        }
    }
    hints
}

fn parse_attr_ignore(attrs: &Vec<syn::Attribute>) -> bool {
    for meta_items in attrs.iter().filter_map(filter_attr_rtti) {
        for meta_item in meta_items {
            match meta_item {
                // Parse `#[rtti(ignore)]`
                Meta(Word(ref ident)) if ident == "ignore" => {
                    return true;
                },
                _ => {}
            }
        }
    }
    false
}

fn parse_lit(lit: &syn::Lit) -> Option<&syn::LitStr> {
    if let syn::Lit::Str(ref lit) = *lit {
        Some(lit)
    } else {
        None
    }
}

fn filter_attr_rtti(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "rtti" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                // TODO: produce an error
                None
            }
        }
    } else {
        None
    }
}

fn dummy_type() -> &'static syn::Type {
    // TODO: pull in lazy_static instead?
    static mut DUMMY: Option<syn::Type> = None;
    unsafe {
        if DUMMY.is_none() {
            DUMMY = Some(syn::Type::Path(syn::TypePath {
                qself: None,
                path: {
                    // there must be a shorter way than this.
                    let p1: syn::PathSegment = "rtti".into();
                    let p2: syn::PathSegment = "Ignored".into();
                    syn::Path {
                        leading_colon: None,
                        segments: {
                            let mut punc = syn::punctuated::Punctuated::new();
                            punc.push(p1);
                            punc.push(p2);
                            punc // rain dance ceremony completed
                        }
                    }
                }
            }));
        }
        DUMMY.as_ref().unwrap()
    }
}