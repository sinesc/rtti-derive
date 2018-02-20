#![recursion_limit="256"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
use proc_macro::TokenStream;

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

    if let syn::Data::Struct(ref data) = ast.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            let idents: Vec<_> = fields.named.iter().map(|field| field.ident.unwrap()).collect();
            let names: Vec<_> = idents.iter().map(|field| field.to_string()).collect();
            let visibilities: Vec<_> = fields.named.iter().map(|field| translate_visibility(&field.vis)).collect();

            let types: Vec<_> = fields.named.iter().map(|field| {
                //TODO: field.attrs, check for ignored types. use Option<Box<Type>>
                &field.ty

            }).collect();

            let result = quote! {
                impl #impl_generics RTTI for #ident #ty_generics #where_clause  {
                    fn rtti() -> Type {
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
                                        ty: Box::new(#types::rtti())
                                    }));
                                )*
                                std::mem::forget(dummy);
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