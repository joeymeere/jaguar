use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(JaguarSerialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            Fields::Unnamed(fields) => fields.unnamed,
            Fields::Unit => return quote! {}.into(),
        },
        _ => return quote! {}.into(),
    };

    let field_serialize = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! {
            self.#field_name.serialize(ser)?;
        }
    });

    let expanded = quote! {
        impl #impl_generics jaguar::JaguarSerialize for #name #ty_generics #where_clause {
            fn serialize(&self, ser: &mut jaguar::JaguarSerializer) -> Result<(), jaguar::SerError> {
                #(#field_serialize)*
                Ok(())
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(JaguarDeserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            Fields::Unnamed(fields) => fields.unnamed,
            Fields::Unit => return quote! {}.into(),
        },
        _ => return quote! {}.into(),
    };

    let field_deserialize = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        quote! {
            let #field_name = <#field_type as jaguar::JaguarDeserialize>::deserialize(de)?;
        }
    });

    let field_names = fields.iter().map(|field| {
        field.ident.as_ref().unwrap()
    });

    let expanded = quote! {
        impl<'a> #impl_generics jaguar::JaguarDeserialize<'a> for #name #ty_generics #where_clause {
            fn deserialize(de: &mut jaguar::JaguarDeserializer<'a>) -> Result<Self, jaguar::SerError> {
                #(#field_deserialize)*
                Ok(Self {
                    #(#field_names,)*
                })
            }
        }
    };

    expanded.into()
}
