use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{Attribute, Data, DataStruct, DeriveInput, Meta, Result};

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(SCGetter, attributes(get, get_ref, get_deref, get_as, get_try_from))]
pub fn add_scgetter(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if let Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let generated = fields
            .iter()
            .filter_map(|field| {
                if has_tag(field.attrs.iter(), "get_deref") {
                    let field_name = field.clone().ident.unwrap();
                    let ty = field.ty.clone();
                    let fn_name = Ident::new(&format!("{}", field_name), Span::call_site());
                    let doc = field.attrs.iter().filter(|v| {
                        v.parse_meta()
                            .map(|meta| meta.path().is_ident("doc"))
                            .unwrap_or(false)
                    });
                    Some(quote! {
                        #(#doc)*
                        #[inline(always)]
                        pub fn #fn_name(&self) -> &<#ty as std::ops::Deref>::Target {
                            &self.#field_name
                        }
                    })
                } else if has_tag(field.attrs.iter(), "get_ref") {
                    let field_name = field.clone().ident.unwrap();
                    let ty = field.ty.clone();
                    let fn_name = Ident::new(&format!("{}", field_name), Span::call_site());
                    let doc = field.attrs.iter().filter(|v| {
                        v.parse_meta()
                            .map(|meta| meta.path().is_ident("doc"))
                            .unwrap_or(false)
                    });
                    Some(quote! {
                        #(#doc)*
                        #[inline(always)]
                        pub fn #fn_name(&self) -> &#ty {
                            &self.#field_name
                        }
                    })
                } else if has_tag(field.attrs.iter(), "get") {
                    let field_name = field.clone().ident.unwrap();
                    let ty = field.ty.clone();
                    let fn_name = Ident::new(&format!("{}", field_name), Span::call_site());
                    let doc = field.attrs.iter().filter(|v| {
                        v.parse_meta()
                            .map(|meta| meta.path().is_ident("doc"))
                            .unwrap_or(false)
                    });
                    Some(quote! {
                        #(#doc)*
                        #[inline(always)]
                        pub fn #fn_name(&self) -> #ty {
                            self.#field_name
                        }
                    })
                } else if has_tag(field.attrs.iter(), "get_as") {
                    let attr = get_tag(field.attrs.iter(), "get_as");
                    let ty_ident = get_tag_attr(attr).unwrap();

                    let field_name = field.clone().ident.unwrap();
                    let fn_name = Ident::new(&format!("{}", field_name), Span::call_site());
                    let doc = field.attrs.iter().filter(|v| {
                        v.parse_meta()
                            .map(|meta| meta.path().is_ident("doc"))
                            .unwrap_or(false)
                    });
                    Some(quote! {
                        #(#doc)*
                        #[inline(always)]
                        pub fn #fn_name(&self) -> #ty_ident {
                            self.#field_name as #ty_ident
                        }
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                #(#generated)*
            }
        }
        .into()
    } else {
        panic!("Nope");
    }
}

fn has_tag<'a, T: Iterator<Item = &'a Attribute>>(mut attribs: T, tag_name: &str) -> bool {
    attribs
        .find_map(|v| {
            let meta = v.parse_meta().expect("failed to parse attr meta data");
            if meta.path().is_ident(tag_name) {
                Some(meta)
            } else {
                None
            }
        })
        .is_some()
}

fn get_tag<'a, T: Iterator<Item = &'a Attribute>>(mut attribs: T, tag_name: &str) -> &'a Attribute {
    attribs
        .find_map(|v| {
            let meta = v.parse_meta().expect("failed to parse attr meta data");
            if meta.path().is_ident(tag_name) {
                Some(v)
            } else {
                None
            }
        })
        .unwrap()
}

fn get_tag_attr(attr: &Attribute) -> Result<Ident> {
    let list: Meta = attr.parse_args()?;
    let ident = list.path().get_ident().expect("missing ident");
    Ok(Ident::new(&format!("{}", ident), Span::call_site()))
}
