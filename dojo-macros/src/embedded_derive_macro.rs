use syn::{Data, Fields};

pub fn embedded_derive_macro_impl(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    // Parse the input tokens into a syntax tree
    let mut ast = syn::parse2::<syn::DeriveInput>(input)?;

    // Define impl variables
    let ident = &ast.ident;

    let fields: Fields = match ast.data.clone() {
        Data::Struct(data) => data.fields,
        _ => panic!("Table can only be derived for structs"),
    };

    // Get the field idents
    let field_idents = fields
        .clone()
        .into_iter()
        .filter_map(|f| f.ident)
        .collect::<Vec<_>>();

    let field_idents_str = field_idents
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>();

    // Define the output tokens
    let expanded = quote::quote! {
        impl<'a> postgres_types::FromSql<'a> for #ident {
            fn from_sql(ty: &postgres_types::Type, mut raw: &[u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
                if *ty == postgres_types::Type::JSONB {
                    let mut b = [0; 1];
                    raw.read_exact(&mut b)?;
                    // We only support version 1 of the jsonb binary format
                    if b[0] != 1 {
                        return Err("unsupported JSONB encoding version".into());
                    }
                }

                serde_json::from_slice(raw).map_err(Into::into)
            }

            postgres_types::accepts!(JSON, JSONB);
        }

        impl postgres_types::ToSql for #ident {
            fn to_sql(
                &self,
                ty: &postgres_types::Type,
                out: &mut bytes::BytesMut,
            ) -> Result<postgres_types::IsNull, Box<dyn Error + Sync + Send>> {
                if *ty == postgres_types::Type::JSONB {
                    bytes::BufMut::put_u8(out, 1);
                }
                use bytes::buf::BufMut;
                serde_json::to_writer(out.writer(), &self)?;
                Ok(postgres_types::IsNull::No)
            }

            postgres_types::accepts!(JSON, JSONB);
            postgres_types::to_sql_checked!();
        }
    };

    // Return the generated impl
    Ok(expanded)
}
