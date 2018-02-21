#![recursion_limit="256"]

/*!
 * Procedural macro to derive RTTI trait. See crate rtti.
 *
 * **very early, probably best to stay away for now**
 */

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
use proc_macro::TokenStream;
use proc_macro2::Span;

#[proc_macro_derive(RTTI, attributes(HelloWorldName))]
pub fn macro_rtti(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let gen = impl_rtti(&ast);
    gen.into()
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

fn impl_rtti(ast: &syn::DeriveInput) -> quote::Tokens {

    let ident = ast.ident;
    let name = ast.ident.to_string();
    let visibility = translate_visibility(&ast.vis);
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let dummy_const = syn::Ident::new(&format!("_IMPL_RTTI_FOR_{}", ident), Span::def_site());

    let body = if let syn::Data::Struct(ref data) = ast.data {
        if let syn::Fields::Named(ref fields) = data.fields {

            // handle structs with named members

            let idents: Vec<_> = fields.named.iter().map(|field| field.ident.unwrap()).collect();
            let names: Vec<_> = idents.iter().map(|field| field.to_string()).collect();
            let visibilities: Vec<_> = fields.named.iter().map(|field| translate_visibility(&field.vis)).collect();

            let types: Vec<_> = fields.named.iter().map(|field| {
                //TODO: field.attrs, check for ignored types. use Option<Box<Type>>
                &field.ty
            }).collect();

            quote! {
                Type::Struct(Struct {
                    name: #name.to_string(),
                    vis: Visibility::#visibility,
                    fields: {
                        let mut fields = Vec::new();
                        let dummy: #ident #impl_generics = unsafe { ::std::mem::uninitialized() };
                        #(
                            fields.push((#names.to_string(), Field {
                                vis: Visibility::#visibilities,
                                offset: {
                                    let dummy_ref = &dummy;
                                    let field_ref = &dummy.#idents;
                                    (field_ref as *const _ as usize) - (dummy_ref as *const _ as usize)
                                },
                                ty: Box::new(<#types>::rtti())
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
                //TODO: field.attrs, check for ignored types. use Option<Box<Type>>
                &field.ty
            }).collect();

            quote! {
                Type::Tuple(Tuple {
                    name: #name.to_string(),
                    vis: Visibility::#visibility,
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
                                ty: Box::new(<#types>::rtti())
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
    } else {
        panic!("#[derive(RTTI)] NYI non-struct..");
    };

    let tmp = quote! {
        #[allow(non_upper_case_globals)]
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
    ;tmp
    //;panic!(tmp.to_string());
}