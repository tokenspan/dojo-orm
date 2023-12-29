use crate::common::derive_get_fields_with_tys;
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(rabbit))]
struct UpdateModelFieldAttributes {
    #[deluxe(default = false)]
    nullable: bool,
}

fn extract_update_model_field_attributes(
    ast: &mut DeriveInput,
) -> deluxe::Result<HashMap<String, UpdateModelFieldAttributes>> {
    let mut field_attrs = HashMap::new();

    if let Data::Struct(s) = &mut ast.data {
        for field in s.fields.iter_mut() {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let attrs = deluxe::extract_attributes(field)?;
            field_attrs.insert(field_name, attrs);
        }
    }

    Ok(field_attrs)
}

pub fn update_model_derive_macro_impl(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse the input tokens into a syntax tree
    let mut ast = syn::parse2::<syn::DeriveInput>(input)?;

    let field_attrs = extract_update_model_field_attributes(&mut ast)?;

    // Define impl variables
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let fields: Fields = match ast.data.clone() {
        Data::Struct(data) => data.fields,
        _ => panic!("Table can only be derived for structs"),
    };

    let params = derive_get_fields_with_tys(fields)
        .into_iter()
        .map(|(ident, outer_ty, _inner_ty)| {
            let nullable = field_attrs
                .get(&ident.to_string())
                .map(|attrs| attrs.nullable)
                .unwrap_or(false);

            if outer_ty == "Option" && !nullable {
                quote::quote! {
                    if let Some(value) = &self.#ident {
                        params.push(value);
                    }
                }
            } else {
                quote::quote! {
                    params.push(&self.#ident);
                }
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote::quote! {
        impl #impl_generics dojo_orm::UpdateModel for #ident #ty_generics #where_clause {
            const COLUMNS: &'static [&'static str] = &["name"];

            fn params(&self) -> Vec<&(dyn postgres_types::ToSql + Sync)> {
                let mut params: Vec<&(dyn postgres_types::ToSql + Sync)> = Vec::new();

                #(#params)*

                params
            }
        }
    };

    Ok(expanded)
}